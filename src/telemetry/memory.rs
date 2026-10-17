//! RAM metrics from sysinfo.

use sysinfo::System;

use super::MemoryStats;

const MB: u64 = 1024 * 1024;

pub fn collect(sys: &System) -> MemoryStats {
    MemoryStats {
        used_mb: Some(sys.used_memory() / MB),
        total_mb: Some(sys.total_memory() / MB),
    }
}
