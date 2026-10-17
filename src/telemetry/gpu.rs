//! GPU metrics via NVML (NVIDIA only). On AMD/Intel or missing driver,
//! `Nvml::init()` fails and every GPU value gracefully becomes "--".
//!
//! Fan RPM uses NVML's explicit RPM API when supported. Percent fan speed
//! is never converted to RPM; unsupported hardware returns `None`.

use nvml_wrapper::enum_wrappers::device::{Clock, TemperatureSensor};
use nvml_wrapper::Nvml;

use super::GpuStats;

pub struct GpuProbe {
    nvml: Option<Nvml>,
    pub nvml_ok: bool,
}

impl Default for GpuProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuProbe {
    pub fn new() -> Self {
        let nvml = Nvml::init().ok();
        Self {
            nvml_ok: nvml.is_some(),
            nvml,
        }
    }
}

pub fn collect(probe: &GpuProbe) -> GpuStats {
    let Some(nvml) = &probe.nvml else {
        return GpuStats::default();
    };
    let Ok(dev) = nvml.device_by_index(0) else {
        return GpuStats::default();
    };
    let mem = dev.memory_info().ok();
    GpuStats {
        temp_c: dev
            .temperature(TemperatureSensor::Gpu)
            .ok()
            .map(|t| t as f32),
        usage_pct: dev.utilization_rates().ok().map(|u| u.gpu as f32),
        core_mhz: dev.clock_info(Clock::Graphics).ok().map(|c| c as f32),
        power_w: dev.power_usage().ok().map(|mw| mw as f32 / 1000.0),
        vram_used_mb: mem.as_ref().map(|m| (m.used / (1024 * 1024)) as u32),
        vram_total_mb: mem.as_ref().map(|m| (m.total / (1024 * 1024)) as u32),
        fan_rpm: dev.fan_speed_rpm(0).ok(),
    }
}
