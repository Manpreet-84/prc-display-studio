//! Local time formatting for the telemetry collector.
//!
//! Uses Win32 `GetLocalTime` — one `unsafe` FFI call whose only effect is
//! filling a zeroed `SYSTEMTIME` struct the OS is required to write.

use windows_sys::Win32::Foundation::SYSTEMTIME;
use windows_sys::Win32::System::SystemInformation::GetLocalTime;

/// Returns (HH:MM, YYYY-MM-DD) in local time.
pub fn local_hm_and_date() -> (String, String) {
    // SAFETY: SYSTEMTIME is a plain-old-data struct; GetLocalTime writes all
    // fields and takes a valid pointer. No other memory is touched.
    unsafe {
        let mut st: SYSTEMTIME = std::mem::zeroed();
        GetLocalTime(&mut st);
        (
            format!("{:02}:{:02}", st.wHour, st.wMinute),
            format!("{:04}-{:02}-{:02}", st.wYear, st.wMonth, st.wDay),
        )
    }
}
