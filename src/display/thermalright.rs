//! Thermalright frame encoding, re-implemented natively in Rust.
//!
//! Protocol behaviour (header layout, RGB565 column-major mapping, chunking)
//! was studied from open-source reference projects (Apache-2.0 and GPL-3.0);
//! no reference code is copied here — header bytes are factual wire values.
//!
//! 320x240 HID layout (observed on VID 0x0416 / PID 0x5302):
//! * 22-byte frame header, then 320*240 RGB565-LE pixels = 153600 bytes.
//! * rows laid out column-major after vertical flip; last pixel of each
//!   column transmitted as 0.
//! * whole frame (22 + 153600 = 153622 bytes) split into 512-byte reports,
//!   each report prefixed with a 0x00 report id and zero-padded at the end.

use super::{DisplayError, Result};

/// Static description of one Thermalright panel variant.
#[derive(Debug, Clone)]
pub struct FrameParams {
    pub width: u32,
    pub height: u32,
    /// USB report payload size (excluding the report-id byte).
    pub chunk: usize,
    pub report_id: u8,
    pub header: Vec<u8>,
}

/// Frozen Warframe 320x240 over HID (VID 0x0416 / PID 0x5302).
pub fn frozen_warframe_320x240() -> FrameParams {
    let mut header = Vec::with_capacity(22);
    header.extend_from_slice(&[0xDA, 0xDB, 0xDC, 0xDD]);
    for v in [2u16, 1, 320, 240, 2, 0] {
        header.extend_from_slice(&v.to_le_bytes());
    }
    header.extend_from_slice(&153600u32.to_le_bytes()); // payload size
    header.extend_from_slice(&0u16.to_le_bytes());
    debug_assert_eq!(header.len(), 22);
    FrameParams {
        width: 320,
        height: 240,
        chunk: 512,
        report_id: 0x00,
        header,
    }
}

/// RGB888 row-major → RGB565-LE column-major (vertical flip, last pixel of
/// each column forced to 0). Returns exactly w*h*2 bytes.
pub fn encode_rgb565_le_columns(frame: &[u8], w: u32, h: u32) -> Result<Vec<u8>> {
    let (w, h) = (w as usize, h as usize);
    if frame.len() < w * h * 3 {
        return Err(DisplayError::new(format!(
            "frame buffer too small: {} < {}",
            frame.len(),
            w * h * 3
        )));
    }
    let mut out = vec![0u8; w * h * 2];
    for x in 0..w {
        for y in 0..h {
            let sy = h - 1 - y; // vertical flip
            let si = (sy * w + x) * 3;
            let r = frame[si];
            let g = frame[si + 1];
            let b = frame[si + 2];
            let mut v = ((r as u16 & 0xF8) << 8) | ((g as u16 & 0xFC) << 3) | (b as u16 >> 3);
            if y == h - 1 {
                v = 0; // last pixel of each column is zeroed on the wire
            }
            let di = (x * h + y) * 2;
            out[di] = (v & 0xFF) as u8;
            out[di + 1] = (v >> 8) as u8;
        }
    }
    Ok(out)
}

/// Encode + prepend header → full wire frame (header + payload).
pub fn build_frame(params: &FrameParams, frame: &[u8]) -> Result<Vec<u8>> {
    let mut wire = Vec::with_capacity(params.header.len() + (params.width * params.height * 2) as usize);
    wire.extend_from_slice(&params.header);
    wire.extend_from_slice(&encode_rgb565_le_columns(frame, params.width, params.height)?);
    Ok(wire)
}

/// Split the wire frame into report chunks of `params.chunk` bytes,
/// zero-padding the final chunk.
pub fn chunk_reports(params: &FrameParams, wire: &[u8]) -> Vec<Vec<u8>> {
    wire.chunks(params.chunk)
        .map(|c| {
            let mut report = vec![params.report_id];
            report.extend_from_slice(c);
            report.resize(params.chunk + 1, 0);
            report
        })
        .collect()
}
