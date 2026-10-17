//! flash-static — flash ONE static image to the Thermalright LCD over the
//! reverse-engineered HID protocol, with no dependency on the stock app.
//!
//! Wire protocol (reused from `prc_lcd::display`):
//!   1. Type-2 handshake: 0x00 report, `[0xDA 0xDB 0xDC 0xDD | 8 zero | 01 00 00 00]`,
//!      padded to 512 bytes. Read-only — only reads the panel identity.
//!   2. Frame: 22-byte header + 320x240 RGB565-LE column-major (vertical flip,
//!      last pixel of each column zeroed), chunked into 512-byte reports.
//!
//! Usage:
//!   flash-static <image.png>
//!   flash-static --probe <image.png>   # handshake only (read-only), then exit
//!   flash-static --hold N <image.png>  # keep the frame on screen N seconds,
//!                                      # re-reading the PNG each second so a
//!                                      # generator script can repaint it live
//!
//! Exit codes: 0 success / 1 device or frame error / 2 usage.

use std::env;
use std::process::ExitCode;

use prc_lcd::display::{
    hid, DisplayDevice,
};

fn usage() -> ExitCode {
    eprintln!(
        "usage: flash-static [--probe] <image.png>\n\n\
         --probe   type-2 handshake only (read-only); do not send a frame\n\
         image.png 320x240 RGB PNG to flash"
    );
    ExitCode::from(2)
}

fn fnv1a(frame: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in frame {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn read_image(path: &str) -> Result<Vec<u8>, String> {
    let img = image::open(path).map_err(|e| format!("open {path}: {e}"))?;
    let rgb = img.to_rgb8();
    let (w, h) = (rgb.width(), rgb.height());
    if w != 320 || h != 240 {
        return Err(format!("image must be 320x240, got {}x{}", w, h));
    }
    Ok(rgb.into_vec())
}

fn probe_only(_img_path: &str) -> ExitCode {
    let api = match hidapi::HidApi::new() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("hid init failed: {e}");
            return ExitCode::from(1);
        }
    };
    let ident = hid::HidIdentity {
        vid: 0x0416,
        pid: 0x5302,
        serial: None,
    };
    let dev = match api.open(ident.vid, ident.pid) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("hid open 0416:5302 failed: {e}");
            return ExitCode::from(1);
        }
    };
    let _ = dev.set_blocking_mode(true);
    hid::send_t2_handshake(&dev);
    println!("probe: panel answered the type-2 handshake");
    ExitCode::from(0)
}

fn static_cmd(img_path: &str) -> ExitCode {
    // 1) Open the panel.
    let api = match hidapi::HidApi::new() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("hid init failed: {e}");
            return ExitCode::from(1);
        }
    };
    let ident = hid::HidIdentity {
        vid: 0x0416,
        pid: 0x5302,
        serial: None,
    };
    let mut dev = match hid::open_frozen_warframe(&api, &ident) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("open device 0416:5302: {e}");
            return ExitCode::from(1);
        }
    };

    // 2) Send the read-only type-2 handshake and drain the panel answer.
    println!("probe: sending type-2 handshake...");
    hid::send_t2_handshake(dev.raw());

    // 3) Load the 320x240 RGB PNG.
    let frame = match read_image(img_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("image: {e}");
            return ExitCode::from(1);
        }
    };

    // 4) Encode + chunk + send the single frame (brightness baked in at 80).
    dev.set_brightness(80).unwrap_or(());
    let sent = match dev.send_frame(&frame) {
        Ok(()) => dev.stats().0,
        Err(e) => {
            eprintln!("send_frame failed: {e}");
            return ExitCode::from(1);
        }
    };

    // 5) Report.
    let h = fnv1a(&frame);
    println!(
        "SENT image={}  fnv1a={:016x}  frames_sent={}",
        img_path, h, sent
    );
    ExitCode::from(0)
}

/// Hold an image on the panel for `hold_secs` seconds.
///
/// HID only — this unit has no bulk endpoints. Re-sends the FULL frame every
/// 600ms using raw report writes, deliberately bypassing `send_frame`'s
/// identical-frame dedup: the panel reverts to its boot logo when the OUT
/// pipe goes quiet for ~1s, so repeats must actually reach the wire.
/// The PNG is re-read each pass so a generator script can repaint it live
/// (atomic replace avoids torn reads; on failure we keep the last good frame).
fn hold_frame(img_path: &str, hold_secs: u64, cadence_ms: Option<u64>) -> ExitCode {
    use prc_lcd::display::thermalright::{build_frame, chunk_reports, frozen_warframe_320x240};
    // Cycle = sleep + ~90ms encode/transfer. Keep total comfortably under the
    // ~1s watchdog; 600ms default ≈ 690ms cycle, long enough for the panel to
    // commit, short enough to not fall back to logo.
    let sleep = cadence_ms.unwrap_or(600);

    let api = match hidapi::HidApi::new() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("hid init failed: {e}");
            return ExitCode::from(1);
        }
    };
    let ident = hid::HidIdentity {
        vid: 0x0416,
        pid: 0x5302,
        serial: None,
    };
    let dev = match hid::open_frozen_warframe(&api, &ident) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("open device 0416:5302: {e}");
            return ExitCode::from(1);
        }
    };

    println!("hold: sending type-2 handshake...");
    hid::send_t2_handshake(dev.raw());

    let mut frame = match read_image(img_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("image: {e}");
            return ExitCode::from(1);
        }
    };
    let params = frozen_warframe_320x240();
    let start = std::time::Instant::now();
    let (mut frames, mut reports, mut acks, mut last_logged) = (0u64, 0u64, 0u64, u64::MAX);
    println!(
        "hold: HID raw resend every {sleep}ms for {hold_secs}s, watching {img_path}, fnv1a={:016x}",
        fnv1a(&frame)
    );
    loop {
        if start.elapsed().as_secs() >= hold_secs {
            break;
        }
        if let Ok(f) = read_image(img_path) {
            frame = f;
        }
        let wire = match build_frame(&params, &frame) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("hold: encode failed: {e}");
                return ExitCode::from(1);
            }
        };
        for report in chunk_reports(&params, &wire) {
            if let Err(e) = dev.raw().write(&report) {
                eprintln!("hold: hid write failed (frame {frames}): {e}");
                return ExitCode::from(1);
            }
            reports += 1;
        }
        frames += 1;
        // Drain the panel's 8-byte status/ACK reports on EP 0x81 (non-blocking).
        // Unread ACKs back up the IN queue and can stall the firmware.
        let mut buf = [0u8; 64];
        loop {
            match dev.raw().read_timeout(&mut buf, 0) {
                Ok(0) => break,
                Ok(_n) => acks += 1,
                Err(_) => break,
            }
        }
        let secs = start.elapsed().as_secs();
        if secs != last_logged && (secs % 5 == 0 || secs + 1 >= hold_secs) {
            println!(
                "hold: [{secs}/{hold_secs}s] frames={frames} reports={reports} acks={acks} fnv1a={:016x}",
                fnv1a(&frame)
            );
            last_logged = secs;
        }
        std::thread::sleep(std::time::Duration::from_millis(sleep));
    }
    println!("hold: done, frames={frames} reports={reports} acks={acks}, kept image {hold_secs}s");
    ExitCode::from(0)
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return usage();
    }
    let mut do_probe = false;
    let mut img_path = None;
    let mut hold_secs: Option<u64> = None;
    let mut cadence_ms: Option<u64> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--probe" => do_probe = true,
            "--cadence" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--cadence requires milliseconds");
                    std::process::exit(2);
                }
                cadence_ms = Some(
                    args[i]
                        .parse()
                        .unwrap_or_else(|_| {
                            eprintln!("--cadence requires milliseconds");
                            std::process::exit(2);
                        }),
                );
            }
            "--hold" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--hold requires a number of seconds");
                    std::process::exit(2);
                }
                hold_secs = Some(
                    args[i]
                        .parse()
                        .unwrap_or_else(|_| {
                            eprintln!("--hold requires a number of seconds");
                            std::process::exit(2);
                        }),
                );
            }
            _ => img_path = Some(args[i].clone()),
        }
        i += 1;
    }
    let img_path = match img_path {
        Some(p) => p,
        None => {
            eprintln!("error: missing image path");
            return usage();
        }
    };
    if do_probe {
        return probe_only(&img_path);
    }
    if let Some(seconds) = hold_secs {
        return hold_frame(&img_path, seconds, cadence_ms);
    }
    static_cmd(&img_path)
}
