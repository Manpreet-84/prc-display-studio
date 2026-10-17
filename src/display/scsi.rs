//! SCSI/Bulk variant placeholder — this unit (VID 0416 / PID 5302) speaks
//! HID, so the SCSI path is a stub until a SCSI-based Frozen Warframe unit
//! is in hand to verify against. The interface is fixed today so M5/M6 code
//! can be written against the trait without caring about transport.

use super::{DisplayDevice, DisplayError, Result};

pub struct ThermalrightScsiDisplay;

impl ThermalrightScsiDisplay {
    pub fn open(_vid: u16, _pid: u16) -> Result<Self> {
        Err(DisplayError::new(
            "SCSI transport not implemented: no verified SCSI unit available",
        ))
    }
}

impl DisplayDevice for ThermalrightScsiDisplay {
    fn width(&self) -> u32 {
        0
    }

    fn height(&self) -> u32 {
        0
    }

    fn set_brightness(&mut self, _value: u8) -> Result<()> {
        Err(DisplayError::new("SCSI display not connected"))
    }

    fn send_frame(&mut self, _frame: &[u8]) -> Result<()> {
        Err(DisplayError::new("SCSI display not connected"))
    }
}
