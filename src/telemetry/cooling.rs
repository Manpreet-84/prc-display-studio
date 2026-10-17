//! Cooling: CPU fan / GPU fan / pump RPM.
//!
//! Windows exposes no standard fan-tach API. These values require a
//! hardware provider (LibreHardwareMonitor-compatible / motherboard SMBus)
//! which arrives in a later milestone. Until then every value is `None`
//! and themes correctly render "--" — values are never invented.

use super::CoolingStats;

pub fn collect() -> CoolingStats {
    CoolingStats::default()
}
