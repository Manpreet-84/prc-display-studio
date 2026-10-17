//! Storage: capacity from sysinfo disks; read/write rates from
//! `DiskUsage::{read_bytes, written_bytes}` which are byte counts *since the
//! last refresh* — divide by elapsed time for MB/s. Disk temperature has no
//! free Windows source yet → `None` ("--").

use std::time::Instant;

use sysinfo::Disks;

use super::StorageStats;

const MB: f64 = 1024.0 * 1024.0;

pub struct StorageProbe {
    last_refresh: Option<Instant>,
}

impl Default for StorageProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageProbe {
    pub fn new() -> Self {
        Self { last_refresh: None }
    }
}

/// Refreshes disk list/usage and returns per-disk stats.
/// Also returns total IO rate (MB/s) across disks for the activity graph.
pub fn collect(disks: &mut Disks, probe: &mut StorageProbe) -> (Vec<StorageStats>, f64) {
    disks.refresh(true);
    let now = Instant::now();
    let dt = probe
        .last_refresh
        .map(|t| now.duration_since(t).as_secs_f64())
        .unwrap_or(0.0);
    probe.last_refresh = Some(now);

    let mut out = Vec::new();
    let mut total_mbs = 0.0f64;

    for d in disks.list() {
        if d.is_removable() {
            continue;
        }
        let total = d.total_space();
        let available = d.available_space();
        let used = total.saturating_sub(available);
        let key = d.mount_point().to_string_lossy().to_string();
        let label = key.trim_end_matches('\\').to_string();
        let label = if label.is_empty() {
            d.name().to_string_lossy().to_string()
        } else {
            label
        };

        let usage = d.usage(); // *_bytes = since last refresh
        let (mut read_mbs, mut write_mbs) = (None, None);
        if dt > 0.05 {
            let r = usage.read_bytes as f64 / dt / MB;
            let w = usage.written_bytes as f64 / dt / MB;
            if r <= 10_000.0 {
                read_mbs = Some(r as f32);
                total_mbs += r;
            }
            if w <= 10_000.0 {
                write_mbs = Some(w as f32);
                total_mbs += w;
            }
        }

        out.push(StorageStats {
            label,
            used_gb: Some(used / (1024 * 1024 * 1024)),
            total_gb: Some(total / (1024 * 1024 * 1024)),
            temp_c: None,
            read_mbs,
            write_mbs,
        });
    }
    (out, total_mbs)
}
