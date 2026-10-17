//! Deterministic fake telemetry for renderer development (M1/M2).
//! Clearly fake values used only for previews — never presented as sensors.

use super::history::{History, Series};
use super::time_util::local_hm_and_date;
use super::{CoolingStats, CpuStats, GpuStats, MemoryStats, NetworkStats, StorageStats, SystemStats, TelemetrySnapshot};

/// Sine composite series so every graph looks plausible and deterministic.
fn synth(base: f32, amp: f32, steps: u32, phase: f32) -> Series {
    let mut s = Series::new(120);
    for i in 0..steps {
        let t = i as f32;
        s.push(base + amp * (t * 0.15 + phase).sin() + amp * 0.3 * (t * 0.05).cos());
    }
    s
}

fn synth_history() -> History {
    History {
        cpu_temp: synth(48.0, 6.0, 120, 0.0),
        gpu_temp: synth(55.0, 5.0, 120, 1.0),
        cpu_usage: synth(25.0, 15.0, 120, 0.5),
        gpu_usage: synth(40.0, 20.0, 120, 2.0),
        net_down: synth(140.0, 60.0, 120, 0.2),
        net_up: synth(45.0, 20.0, 120, 1.7),
        ssd_activity: synth(30.0, 25.0, 120, 0.9),
        cpu_fan: synth(1250.0, 80.0, 120, 0.3),
        gpu_fan: synth(1450.0, 100.0, 120, 1.4),
        pump: synth(2600.0, 40.0, 120, 0.7),
    }
}

pub fn fake_telemetry() -> TelemetrySnapshot {
    let (time_hm, date_ymd) = local_hm_and_date();
    TelemetrySnapshot {
        demo: true,
        cpu: CpuStats {
            temp_c: Some(48.0),
            usage_pct: Some(22.0),
            freq_mhz: Some(4820.0),
            power_w: Some(35.0),
        },
        gpu: GpuStats {
            temp_c: Some(55.0),
            usage_pct: Some(41.0),
            core_mhz: Some(2610.0),
            power_w: Some(120.0),
            vram_used_mb: Some(8499),
            vram_total_mb: Some(12288),
            fan_rpm: Some(1450),
        },
        memory: MemoryStats {
            used_mb: Some(15_360),
            total_mb: Some(32_768),
        },
        storage: vec![StorageStats {
            label: "SSD".to_string(),
            used_gb: Some(720),
            total_gb: Some(1000),
            temp_c: Some(45.0),
            read_mbs: Some(152.0),
            write_mbs: Some(48.0),
        }],
        network: NetworkStats {
            down_mbps: Some(152.0),
            up_mbps: Some(48.0),
            local_ip: Some("192.0.2.10".to_string()),
            adapter: Some("Ethernet".to_string()),
            conn_type: Some("ETHERNET".to_string()),
        },
        cooling: CoolingStats {
            cpu_fan_rpm: Some(1250),
            gpu_fan_rpm: Some(1450),
            pump_rpm: Some(2600),
            motherboard_fans: vec![("Fan #1".into(),1250),("Fan #2".into(),1450),("Fan #7".into(),2600)],
        },
        system: SystemStats {
            os: Some("Windows 11 Pro".to_string()),
            hostname: Some("PRC-PC".to_string()),
            uptime_secs: Some(3 * 86400 + 12 * 3600),
            time_hm: Some(time_hm),
            date_ymd: Some(date_ymd),
        },
        media: None,
        history: synth_history(),
    }
}

/// Fake snapshot with active media session (media page preview).
pub fn fake_media_telemetry() -> TelemetrySnapshot {
    let mut s = fake_telemetry();
    s.media = Some(super::MediaInfo {
        title: "NEON HORIZON".to_string(),
        artist: "SYNTHWAVE UNIT".to_string(),
        elapsed_s: 142,
        duration_s: 245,
        playing: true,
    });
    s
}

/// Fake snapshot in CPU/GPU critical territory (alert page preview).
pub fn fake_alert_telemetry() -> TelemetrySnapshot {
    let mut s = fake_telemetry();
    s.cpu.temp_c = Some(92.0);
    s.cpu.usage_pct = Some(99.0);
    s.gpu.temp_c = Some(88.0);
    s.gpu.usage_pct = Some(97.0);
    s
}
