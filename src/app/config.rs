//! PRC LCD application configuration (Milestone 1 subset).
//!
//! Only `[display]` is honoured in M1. Everything else is parsed
//! leniently / ignored so the full example `config.toml` from the spec
//! already parses without adding serde/toml dependencies yet.

#[derive(Debug, Clone, Copy)]
pub struct DisplayConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub brightness: u8,
    pub orientation: u32,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            width: 320,
            height: 240,
            fps: 5,
            brightness: 80,
            orientation: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AppConfig {
    pub display: DisplayConfig,
    /// Telemetry refresh interval (ms). Spec default: 1000.
    pub telemetry_interval_ms: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            display: DisplayConfig::default(),
            telemetry_interval_ms: 1000,
        }
    }
}

impl AppConfig {
    /// Load `config.toml` if present, else defaults.
    /// M1 uses a tiny hand parser to avoid serde/toml deps.
    pub fn load_or_default(path: &str) -> Self {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        if text.trim().is_empty() {
            return Self::default();
        }
        let mut cfg = Self::default();
        let mut section = String::new();
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].trim().to_ascii_lowercase();
                continue;
            }
            let mut parts = line.splitn(2, '=');
            let key = parts.next().unwrap_or("").trim();
            let mut val = parts.next().unwrap_or("").trim();
            // Strip inline comments.
            if let Some(i) = val.find('#') {
                val = val[..i].trim();
            }
            if let Some(i) = val.find(';') {
                val = val[..i].trim();
            }
            val = val.trim_matches('"').trim_matches('\'');
            match (section.as_str(), key) {
                ("display", "fps") => {
                    if let Ok(v) = val.parse::<u32>() {
                        cfg.display.fps = v.clamp(1, 30);
                    }
                }
                ("display", "brightness") => {
                    if let Ok(v) = val.parse::<u32>() {
                        cfg.display.brightness = v.clamp(0, 100) as u8;
                    }
                }
                ("display", "orientation") => {
                    if let Ok(v) = val.parse::<u32>() {
                        cfg.display.orientation = v % 4;
                    }
                }
                ("telemetry", "interval_ms") => {
                    if let Ok(v) = val.parse::<u64>() {
                        cfg.telemetry_interval_ms = v.clamp(200, 60_000);
                    }
                }
                _ => {}
            }
        }
        cfg
    }
}
