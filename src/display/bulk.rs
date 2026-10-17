//! USB **bulk** transport for the Frozen Warframe panel (0416:5302).
//!
//! Wire behavior re-implemented natively from protocol study of open-source
//! reference projects (Apache-2.0) plus decompilation of the vendor's own
//! .NET writer (`USBLCDNEW`/`TRCC`, local analysis for interoperability):
//!
//! 1. Set config 1, claim the interface owning bulk EP 0x02 (OUT) / 0x01 (IN).
//! 2. Handshake: sleep 50ms, write 512-byte DEV_INFO request
//!    (`DA DB DC DD` … `[12]=1`), sleep 200ms, read 512, require
//!    `[0..4]=magic && [12]==1 && [16]==16` (serial length at `[20..36]`).
//! 3. Frames: rotate the 320×240 canvas 90° CW → 240×320, JPEG (q95,
//!    vendor retries lower if ≥450 000 B), prepend the 20-byte picture
//!    header (`DA DB DC DD | 02 00 00 00 | 240 | 320 | 02 00 00 00 | len`),
//!    round the total up to a multiple of 512, bulk-write EP 0x02.
//! 4. Brightness is baked into pixels (the vendor does the same).
//!    Rotate direction follows the config `orientation` (0 = 90° CW).

use std::time::Duration;

use rusb::{DeviceHandle, GlobalContext};

use super::DisplayDevice;
use super::{DisplayError, Result};

const VID: u16 = 0x0416;
const PID: u16 = 0x5302;
const EP_OUT: u8 = 0x02;
const EP_IN: u8 = 0x01;
const HANDSHAKE_LEN: usize = 512;
const MAX_JPEG: usize = 450_000; // vendor cap; quality steps down below this
const JPEG_QUALITY: u8 = 95;

pub struct ThermalrightBulkDisplay {
    handle: DeviceHandle<GlobalContext>,
    brightness: u8,          // 0..100, baked into pixels
    orientation: u32,        // 0..3 → orientation steps (vendor directionB)
    last_hash: Option<u64>,  // frame dedup: skip unchanged USB traffic
    frames_sent: u64,
    frames_skipped: u64,
    pub serial: String,
    pub pm: u8,
}


impl ThermalrightBulkDisplay {
    pub fn open(orientation: u32) -> Result<Self> {
        let handle = rusb::open_device_with_vid_pid(VID, PID)
            .ok_or_else(|| DisplayError::new(format!("device {VID:04x}:{PID:04x} not found")))?;

        // Config 1 (vendor: SetConfiguration(1)).
        let _ = handle.set_active_configuration(1);

        // Find the interface that owns the bulk OUT endpoint (0x02).
        let config = handle
            .device()
            .active_config_descriptor()
            .map_err(|e| DisplayError::new(format!("get config: {e}")))?;
        let mut bulk_iface: Option<u8> = None;
        for iface in config.interfaces() {
            for desc in iface.descriptors() {
                for ep in desc.endpoint_descriptors() {
                    if ep.address() == EP_OUT && ep.transfer_type() == rusb::TransferType::Bulk {
                        bulk_iface = Some(desc.interface_number());
                    }
                }
            }
        }
        let iface = bulk_iface.ok_or_else(|| {
            // Diagnostic: dump everything we can see.
            let mut seen = String::new();
            for i in config.interfaces() {
                for d in i.descriptors() {
                    for ep in d.endpoint_descriptors() {
                        seen.push_str(&format!(
                            " [if{} {:02x} {:?} {:?}]",
                            d.interface_number(),
                            ep.address(),
                            ep.transfer_type(),
                            ep.direction()
                        ));
                    }
                }
            }
            DisplayError::new(format!("no bulk OUT 0x02 interface; seen:{seen}"))
        })?;
        handle
            .claim_interface(iface)
            .map_err(|e| DisplayError::new(format!("claim interface {iface}: {e}")))?;

        let mut dev = Self {
            handle,
            brightness: 100,
            orientation: orientation % 4,
            last_hash: None,
            frames_sent: 0,
            frames_skipped: 0,
            serial: String::new(),
            pm: 0,
        };
        dev.handshake()?;
        Ok(dev)
    }

    /// DEV_INFO handshake, vendor-faithful timing (50ms → write → 200ms → read).
    fn handshake(&mut self) -> Result<()> {
        std::thread::sleep(Duration::from_millis(50));
        let mut hs = vec![0u8; HANDSHAKE_LEN];
        hs[0..4].copy_from_slice(&[0xDA, 0xDB, 0xDC, 0xDD]);
        hs[12] = 1; // SSCRM_CMD_TYPE_DEV_INFO
        self.handle
            .write_bulk(EP_OUT, &hs, Duration::from_millis(1000))
            .map_err(|e| DisplayError::new(format!("handshake write: {e}")))?;
        std::thread::sleep(Duration::from_millis(200));
        let mut resp = [0u8; HANDSHAKE_LEN];
        let n = self
            .handle
            .read_bulk(EP_IN, &mut resp, Duration::from_millis(1000))
            .map_err(|e| DisplayError::new(format!("handshake read: {e}")))?;
        if n >= 20 && resp[0..4] == [0xDA, 0xDB, 0xDC, 0xDD] && resp[12] == 1 && resp[16] == 16 {
            let serial_bytes = &resp[20..36];
            self.serial = serial_bytes.iter().map(|b| format!("{b:02X}")).collect();
            self.pm = resp[5]; // observed PM=51 on this unit
            println!(
                "panel: handshake OK ({} bytes), PM={}, serial={}",
                n, self.pm, self.serial
            );
            Ok(())
        } else {
            Err(DisplayError::new(format!(
                "handshake response invalid: n={n} head={:02X?}",
                &resp[..n.min(20)]
            )))
        }
    }

    /// Rotate W×H RGB 90° clockwise → H×W.
    fn rotate_cw90(src: &[u8], w: u32, h: u32) -> Vec<u8> {
        let (w, h) = (w as usize, h as usize);
        let mut dst = vec![0u8; w * h * 3];
        for sy in 0..h {
            for sx in 0..w {
                let si = (sy * w + sx) * 3;
                let di = ((sx * h) + (h - 1 - sy)) * 3;
                dst[di..di + 3].copy_from_slice(&src[si..si + 3]);
            }
        }
        dst
    }

    /// Rotate W×H RGB 90° counter-clockwise → H×W.
    fn rotate_ccw90(src: &[u8], w: u32, h: u32) -> Vec<u8> {
        let (w, h) = (w as usize, h as usize);
        let mut dst = vec![0u8; w * h * 3];
        for sy in 0..h {
            for sx in 0..w {
                let si = (sy * w + sx) * 3;
                let di = (((h - 1 - sx) * h) + sy) * 3;
                dst[di..di + 3].copy_from_slice(&src[si..si + 3]);
            }
        }
        dst
    }

    pub fn stats(&self) -> (u64, u64) {
        (self.frames_sent, self.frames_skipped)
    }
}
fn fnv1a(frame: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in frame {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

impl DisplayDevice for ThermalrightBulkDisplay {
    fn width(&self) -> u32 {
        320 // logical landscape canvas
    }

    fn height(&self) -> u32 {
        240
    }

    fn set_brightness(&mut self, value: u8) -> Result<()> {
        // Vendor bakes brightness into the rendered pixels; same here.
        self.brightness = value.min(100);
        Ok(())
    }

    fn send_frame(&mut self, frame: &[u8]) -> Result<()> {
        const W: usize = 320;
        const H: usize = 240;
        if frame.len() < W * H * 3 {
            return Err(DisplayError::new(format!(
                "frame too small: {} < {}",
                frame.len(),
                W * H * 3
            )));
        }
        let hash = fnv1a(&frame[..W * H * 3]);
        if self.last_hash == Some(hash) {
            self.frames_skipped += 1;
            return Ok(()); // unchanged frame → no USB traffic
        }

        // 1) brightness baked in.
        let rgb: Vec<u8> = if self.brightness >= 100 {
            frame[..W * H * 3].to_vec()
        } else {
            let k = self.brightness as u32;
            frame[..W * H * 3].iter().map(|b| (*b as u32 * k / 100) as u8).collect()
        };

        // 2) orientation (vendor directionB=0 → RotateImg(90° CW) → 240×320).
        let portrait = match self.orientation {
            0 | 2 => Self::rotate_cw90(&rgb, W as u32, H as u32),
            _ => Self::rotate_ccw90(&rgb, W as u32, H as u32),
        };
        let (pw, ph) = (240u32, 320u32); // wire header dims (vendor literal)

        // 3) JPEG with the vendor quality ladder (start 95, -5 while ≥450KB).
        let mut quality = JPEG_QUALITY;
        let jpeg = loop {
            let mut out = Vec::new();
            let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, quality);
            enc.encode(&portrait, pw, ph, image::ColorType::Rgb8.into())
                .map_err(|e| DisplayError::new(format!("jpeg: {e}")))?;
            if out.len() < MAX_JPEG || quality <= 60 {
                break out;
            }
            quality -= 5;
        };

        // 4) 20-byte picture header + payload, padded to a 512 multiple.
        let mut wire = Vec::with_capacity(20 + jpeg.len() + 512);
        wire.extend_from_slice(&[0xDA, 0xDB, 0xDC, 0xDD]);
        wire.extend_from_slice(&2u32.to_le_bytes()); // SSCRM_CMD_TYPE_PICTURE
        wire.extend_from_slice(&(pw as u16).to_le_bytes()); // vendor: 240
        wire.extend_from_slice(&(ph as u16).to_le_bytes()); // vendor: 320
        wire.extend_from_slice(&2u32.to_le_bytes());
        wire.extend_from_slice(&(jpeg.len() as u32).to_le_bytes());
        wire.extend_from_slice(&jpeg);
        let pad = (512 - wire.len() % 512) % 512;
        wire.resize(wire.len() + pad, 0);

        self.handle
            .write_bulk(EP_OUT, &wire, Duration::from_millis(1000))
            .map_err(|e| DisplayError::new(format!("bulk write: {e}")))?;
        std::thread::sleep(Duration::from_millis(1)); // vendor pacing

        self.last_hash = Some(hash);
        self.frames_sent += 1;
        Ok(())
    }
}
