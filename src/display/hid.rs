//! HID transport for Thermalright panels (hidapi).
//!
//! The display interface (MI_00) exposes a vendor-defined usage page
//! (UP 0xFF06 / U 0x0001) which hidapi can open while the Thermalright
//! service is stopped. Frames arrive as 0x00-report-id + 512-byte chunks.

use super::thermalright::{build_frame, chunk_reports, frozen_warframe_320x240, FrameParams};
use super::{DisplayDevice, DisplayError, Result};

/// Panel identity supplied by `device_detection`.
#[derive(Debug, Clone)]
pub struct HidIdentity {
    pub vid: u16,
    pub pid: u16,
    pub serial: Option<String>,
}

pub struct ThermalrightHidDisplay {
    dev: hidapi::HidDevice,
    params: FrameParams,
    brightness: u8,
    /// FNV-1a hash of the last transmitted RGB frame — identical frames
    /// are not re-sent (spec: skip USB transfer when unchanged).
    last_hash: Option<u64>,
    frames_sent: u64,
    frames_skipped: u64,
}

impl ThermalrightHidDisplay {
    /// Borrow the raw handle (handshakes, reads) without breaking the
    /// frame-dedup state.
    pub fn raw(&self) -> &hidapi::HidDevice {
        &self.dev
    }
}

/// Frozen Warframe 320x240 panel on interface MI_00.
pub fn open_frozen_warframe(api: &hidapi::HidApi, ident: &HidIdentity) -> Result<ThermalrightHidDisplay> {
    let dev = match &ident.serial {
        Some(s) if !s.is_empty() => api.open_serial(ident.vid, ident.pid, s),
        _ => api.open(ident.vid, ident.pid),
    }
    .map_err(|e| DisplayError::new(format!("hid open {:04x}:{:04x} failed: {e}", ident.vid, ident.pid)))?;
    // generous but bounded blocking writes; never an infinite stall
    let _ = dev.set_blocking_mode(true);
    Ok(ThermalrightHidDisplay {
        dev,
        params: frozen_warframe_320x240(),
        brightness: 80,
        last_hash: None,
        frames_sent: 0,
        frames_skipped: 0,
    })
}

/// Type-2 identify handshake: [00] DA DB DC DD + 8 zero + 01 00 00 00,
/// padded to one 512-byte payload + report id.
///
/// Best effort — failures are only logged; some firmware variants answer
/// slowly or not at all yet still accept frames.
pub fn send_t2_handshake(dev: &hidapi::HidDevice) {
    let mut init = vec![0x00u8];
    init.extend_from_slice(&[0xDA, 0xDB, 0xDC, 0xDD]);
    init.extend_from_slice(&[0u8; 8]);
    init.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
    init.resize(513, 0);
    match dev.write(&init) {
        Ok(n) => println!("panel: T2 handshake sent ({} bytes)", n),
        Err(e) => println!("panel: T2 handshake write failed: {e}"),
    }
    // Drain the PM/SUB/FBL answer so it can't be mistaken for frame ACKs.
    let mut resp = [0u8; 512];
    match dev.read_timeout(&mut resp, 2000) {
        Ok(0) => println!("panel: T2 answer: none"),
        Ok(n) => println!(
            "panel: T2 answer: {} bytes, PM={} SUB={}",
            n,
            resp.get(5).copied().unwrap_or(0),
            resp.get(4).copied().unwrap_or(0)
        ),
        Err(e) => println!("panel: T2 answer read failed: {e}"),
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

impl DisplayDevice for ThermalrightHidDisplay {
    fn width(&self) -> u32 {
        self.params.width
    }

    fn height(&self) -> u32 {
        self.params.height
    }

    fn set_brightness(&mut self, value: u8) -> Result<()> {
        // No brightness command is documented for this panel; apply in the
        // framebuffer (software scaling) at encode time. Storing here keeps
        // the setting consistent across reconnects.
        self.brightness = value.min(100);
        Ok(())
    }

    fn send_frame(&mut self, frame: &[u8]) -> Result<()> {
        let expected = (self.params.width * self.params.height * 3) as usize;
        if frame.len() < expected {
            return Err(DisplayError::new(format!(
                "frame too small: {} < {}",
                frame.len(),
                expected
            )));
        }
        let h = fnv1a(&frame[..expected]);
        if self.last_hash == Some(h) {
            self.frames_skipped += 1;
            return Ok(()); // unchanged — no USB traffic
        }

        let scaled;
        let rgb: &[u8] = if self.brightness >= 100 {
            &frame[..expected]
        } else {
            // Software brightness: per-channel scale (integer math, no float).
            let k = self.brightness as u32;
            scaled = frame[..expected].iter().map(|b| (*b as u32 * k / 100) as u8).collect::<Vec<u8>>();
            &scaled
        };

        let wire = build_frame(&self.params, rgb)?;
        for report in chunk_reports(&self.params, &wire) {
            self.dev
                .write(&report)
                .map_err(|e| DisplayError::new(format!("hid write failed: {e}")))?;
        }
        self.last_hash = Some(h);
        self.frames_sent += 1;
        Ok(())
    }
}

impl ThermalrightHidDisplay {
    pub fn stats(&self) -> (u64, u64) {
        (self.frames_sent, self.frames_skipped)
    }
}
