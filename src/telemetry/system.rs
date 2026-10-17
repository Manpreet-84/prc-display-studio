//! System: OS, hostname, uptime, local clock/date.

use sysinfo::System;

use super::time_util::local_hm_and_date;
use super::SystemStats;

pub fn collect() -> SystemStats {
    let (time_hm, date_ymd) = local_hm_and_date();
    SystemStats {
        os: System::long_os_version().or_else(System::name),
        hostname: System::host_name(),
        uptime_secs: Some(System::uptime()),
        time_hm: Some(time_hm),
        date_ymd: Some(date_ymd),
    }
}
