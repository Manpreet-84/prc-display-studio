//! Normalized telemetry snapshot. Every metric is `Option<T>` so
//! themes render "--" for unavailable sensors instead of crashing.

#[cfg(feature = "live-telemetry")]
pub mod collector;
#[cfg(feature = "live-telemetry")]
pub mod cpu;
pub mod fake;
#[cfg(feature = "live-telemetry")]
pub mod gpu;
pub mod history;
#[cfg(feature = "live-telemetry")]
pub mod memory;
#[cfg(feature = "live-telemetry")]
pub mod network;
#[cfg(feature = "live-telemetry")]
pub mod storage;
#[cfg(feature = "live-telemetry")]
pub mod system;
pub mod time_util;
#[cfg(feature = "live-telemetry")]
pub mod cooling;
#[cfg(feature = "live-telemetry")]
pub mod media;

#[cfg(feature = "live-telemetry")]
pub use collector::Collector;
pub use fake::{fake_alert_telemetry, fake_media_telemetry, fake_telemetry};
pub use history::{History, Series};

#[derive(Debug, Clone, Default)]
pub struct CpuStats {
    pub temp_c: Option<f32>,
    pub usage_pct: Option<f32>,
    pub freq_mhz: Option<f32>,
    pub power_w: Option<f32>,
}

#[derive(Debug, Clone, Default)]
pub struct GpuStats {
    pub temp_c: Option<f32>,
    pub usage_pct: Option<f32>,
    pub core_mhz: Option<f32>,
    pub power_w: Option<f32>,
    pub vram_used_mb: Option<u32>,
    pub vram_total_mb: Option<u32>,
    pub fan_rpm: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub used_mb: Option<u64>,
    pub total_mb: Option<u64>,
}

impl MemoryStats {
    pub fn pct(&self) -> Option<f32> {
        match (self.used_mb, self.total_mb) {
            (Some(u), Some(t)) if t > 0 => Some(u as f32 / t as f32 * 100.0),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StorageStats {
    pub label: String,
    pub used_gb: Option<u64>,
    pub total_gb: Option<u64>,
    pub temp_c: Option<f32>,
    pub read_mbs: Option<f32>,
    pub write_mbs: Option<f32>,
}

impl StorageStats {
    pub fn pct(&self) -> Option<f32> {
        match (self.used_gb, self.total_gb) {
            (Some(u), Some(t)) if t > 0 => Some(u as f32 / t as f32 * 100.0),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    pub down_mbps: Option<f32>,
    pub up_mbps: Option<f32>,
    pub local_ip: Option<String>,
    pub adapter: Option<String>,
    /// Connection type, e.g. "ETHERNET" / "WI-FI" / "VPN".
    pub conn_type: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CoolingStats {
    pub cpu_fan_rpm: Option<u32>,
    pub gpu_fan_rpm: Option<u32>,
    pub pump_rpm: Option<u32>,
    /// Unmapped tachometers keep the provider's labels; no guessed fan roles.
    pub motherboard_fans: Vec<(String, u32)>,
}

#[derive(Debug, Clone, Default)]
pub struct SystemStats {
    pub os: Option<String>,
    pub hostname: Option<String>,
    pub uptime_secs: Option<u64>,
    /// Local wall-clock "HH:MM" (collector-filled; themes never call the OS).
    pub time_hm: Option<String>,
    /// Local date "YYYY-MM-DD".
    pub date_ymd: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub elapsed_s: u32,
    pub duration_s: u32,
    pub playing: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TelemetrySnapshot {
    /// True only for generated gallery samples, never for collected data.
    pub demo: bool,
    pub cpu: CpuStats,
    pub gpu: GpuStats,
    pub memory: MemoryStats,
    pub storage: Vec<StorageStats>,
    pub network: NetworkStats,
    pub cooling: CoolingStats,
    pub system: SystemStats,
    pub media: Option<MediaInfo>,
    /// Ring buffers for graphs (60–120 samples).
    pub history: History,
}





#[cfg(feature = "live-telemetry")]
pub mod hardware;
