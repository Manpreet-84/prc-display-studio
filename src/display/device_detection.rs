//! Device detection: inspect HID devices for supported VID/PID pairs and
//! pick the panel variant. Renderer code never sees USB identities.

use super::hid::HidIdentity;
use super::TransportKind;

/// (vid, pid, width, height, transport, description)
const SUPPORTED: [(u16, u16, u32, u32, TransportKind, &str); 1] = [(
    0x0416,
    0x5302,
    320,
    240,
    TransportKind::Hid,
    "Thermalright Frozen Warframe 320x240 (HID)",
)];

#[derive(Debug, Clone)]
pub struct DetectedPanel {
    pub ident: HidIdentity,
    pub width: u32,
    pub height: u32,
    pub transport: TransportKind,
    pub description: &'static str,
}

pub fn detect(api: &hidapi::HidApi) -> Vec<DetectedPanel> {
    let mut out = Vec::new();
    for info in api.device_list() {
        for (vid, pid, w, h, transport, desc) in SUPPORTED {
            if info.vendor_id() == vid && info.product_id() == pid {
                out.push(DetectedPanel {
                    ident: HidIdentity {
                        vid,
                        pid,
                        serial: info.serial_number().map(|s| s.to_string()),
                    },
                    width: w,
                    height: h,
                    transport,
                    description: desc,
                });
            }
        }
    }
    out
}

pub fn describe_supported() -> String {
    SUPPORTED
        .iter()
        .map(|(v, p, w, h, t, d)| format!("{:04x}:{:04x} {}x{} {:?} — {}", v, p, w, h, t, d))
        .collect::<Vec<_>>()
        .join("\n")
}
