//! Hardware abstraction: the renderer only knows pixels.
//! Implemented by `ThermalrightHidDisplay` (M4) and later `ThermalrightScsiDisplay`.

pub mod bulk;
pub mod device_detection;
pub mod hid;
pub mod scsi;
pub mod thermalright;

use std::fmt;

/// USB transport used by a concrete panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    Hid,
    Bulk,
    Scsi,
}

#[derive(Debug)]
pub struct DisplayError(pub String);

impl DisplayError {
    pub fn new(msg: impl Into<String>) -> Self {
        DisplayError(msg.into())
    }
}

impl fmt::Display for DisplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "display error: {}", self.0)
    }
}

impl std::error::Error for DisplayError {}

pub type Result<T> = std::result::Result<T, DisplayError>;

/// Target panel contract. Implemented in M4 by
/// `ThermalrightHidDisplay` / `ThermalrightScsiDisplay`.
pub trait DisplayDevice {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn set_brightness(&mut self, value: u8) -> Result<()>;
    /// `frame`: width*height*3 RGB888 bytes, row-major, top-left origin.
    /// Implementations encode + chunk + skip-if-identical.
    fn send_frame(&mut self, frame: &[u8]) -> Result<()>;
}
