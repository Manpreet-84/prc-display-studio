//! CPU metrics.
//!
//! * usage: our own probe based on kernel `GetSystemTimes` (always available,
//!   works even when the perf-counter registry is broken — a case where
//!   `sysinfo::global_cpu_usage` would report a stuck 100 % because PDH
//!   returns 0 for failed counters).
//! * frequency: sysinfo max per-core MHz.
//! * temperature/power: supplied separately by optional hardware provider.
//! No ACPI thermal-zone values are presented as CPU package temperatures.

use sysinfo::System;
use windows_sys::Win32::Foundation::FILETIME;
use windows_sys::Win32::System::Threading::GetSystemTimes;

use super::CpuStats;

/// One `GetSystemTimes` sample (100 ns units, ALL cores).
#[derive(Clone, Copy, Default)]
struct Times {
    idle: u64,
    kernel: u64,
    user: u64,
}

impl Times {
    fn read() -> Option<Times> {
        // SAFETY: GetSystemTimes takes three valid out-pointers and writes
        // 64-bit values. Zeroed FILETIMEs are always safe to pass.
        unsafe {
            let mut idle: FILETIME = std::mem::zeroed();
            let mut kernel: FILETIME = std::mem::zeroed();
            let mut user: FILETIME = std::mem::zeroed();
            if GetSystemTimes(&mut idle, &mut kernel, &mut user) == 0 {
                return None;
            }
            let ft = |t: FILETIME| ((t.dwHighDateTime as u64) << 32) | t.dwLowDateTime as u64;
            Some(Times {
                idle: ft(idle),
                kernel: ft(kernel),
                user: ft(user),
            })
        }
    }

    /// Usage since the previous sample. `busy = (kernel + user) - idle`.
    fn usage_since(&self, prev: &Times) -> Option<f32> {
        let idle_d = self.idle.saturating_sub(prev.idle) as f64;
        let busy_d = (self.kernel.saturating_sub(prev.kernel) as f64
            + self.user.saturating_sub(prev.user) as f64)
            - idle_d;
        let total = idle_d.max(0.0) + busy_d.max(0.0);
        if total <= 0.0 {
            None
        } else {
            Some((busy_d.max(0.0) / total * 100.0).clamp(0.0, 100.0) as f32)
        }
    }
}

/// CPU load uses kernel deltas; temperature needs a CPU hardware provider.
pub struct CpuProbe {
    prev_times: Option<Times>,
}
impl Default for CpuProbe {
    fn default() -> Self {
        Self::new()
    }
}
impl CpuProbe {
    pub fn new() -> Self {
        Self {
            prev_times: Times::read(),
        }
    }
    fn usage(&mut self) -> Option<f32> {
        let now = Times::read()?;
        let prev = self.prev_times.replace(now)?;
        now.usage_since(&prev)
    }
}
pub fn collect(sys: &System, probe: &mut CpuProbe) -> CpuStats {
    let freq = sys.cpus().iter().map(|c| c.frequency()).max();
    CpuStats {
        temp_c: None, // ACPI thermal zones are not CPU package sensors.
        usage_pct: probe.usage(),
        freq_mhz: freq.map(|f| f as f32),
        power_w: None, // no standard Windows source yet
    }
}
