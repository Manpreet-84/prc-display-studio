use crate::{
    renderer::{
        modern::Surface,
        Canvas,
    },
    telemetry::TelemetrySnapshot,
};
use serde::Deserialize;
fn yes() -> bool {
    true
}
fn white() -> [u8; 3] {
    [245, 248, 255]
}
fn track() -> [u8; 3] {
    [43, 57, 65]
}
fn label_size() -> u16 {
    12
}
fn stroke() -> f32 {
    5.0
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Element {
    pub kind: String,
    pub sensor: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub size: u16,
    pub max: f32,
    pub color: [u8; 3],
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub text: String,
    #[serde(default = "yes")]
    pub show_label: bool,
    #[serde(default = "yes")]
    pub show_value: bool,
    #[serde(default = "yes")]
    pub show_unit: bool,
    #[serde(default = "label_size")]
    pub label_size: u16,
    #[serde(default = "white")]
    pub value_color: [u8; 3],
    #[serde(default)]
    pub label_color: Option<[u8; 3]>,
    #[serde(default = "track")]
    pub track_color: [u8; 3],
    #[serde(default)]
    pub background_color: [u8; 3],
    #[serde(default)]
    pub opacity: u8,
    #[serde(default = "stroke")]
    pub stroke: f32,
    #[serde(default)]
    pub decimals: u8,
}
pub struct Custom {
    pub pixels: Vec<u8>,
    pub elements: Vec<Element>,
    pub samples: std::collections::HashMap<String, std::collections::VecDeque<f32>>,
}
pub fn metric(t: &TelemetrySnapshot, id: &str) -> Option<f32> {
    match id {
        "cpu.temp" => t.cpu.temp_c,
        "cpu.load" => t.cpu.usage_pct,
        "cpu.power" => t.cpu.power_w,
        "cpu.clock" => t.cpu.freq_mhz,
        "gpu.temp" => t.gpu.temp_c,
        "gpu.load" => t.gpu.usage_pct,
        "gpu.power" => t.gpu.power_w,
        "gpu.clock" => t.gpu.core_mhz,
        "gpu.fan" => t.gpu.fan_rpm.map(|v| v as f32),
        "ram.load" => t.memory.pct(),
        "ram.used" => t.memory.used_mb.map(|v| v as f32 / 1024.0),
        "vram.used" => t.gpu.vram_used_mb.map(|v| v as f32 / 1024.0),
        "vram.load" => t
            .gpu
            .vram_used_mb
            .zip(t.gpu.vram_total_mb)
            .filter(|(_, n)| *n > 0)
            .map(|(v, n)| v as f32 / n as f32 * 100.0),
        "net.down" => t.network.down_mbps,
        "net.up" => t.network.up_mbps,
        "cpu.fan" => t.cooling.cpu_fan_rpm.map(|v| v as f32),
        "pump" => t.cooling.pump_rpm.map(|v| v as f32),
        "disk.temp" => t.storage.first().and_then(|v| v.temp_c),
        "disk.load" => t.storage.first().and_then(|v| v.pct()),
        "disk.read" => t.storage.first().and_then(|v| v.read_mbs),
        "disk.write" => t.storage.first().and_then(|v| v.write_mbs),
        _ => None,
    }
}
pub fn unit(sensor: &str) -> &'static str {
    match sensor {
        "cpu.temp" | "gpu.temp" | "disk.temp" => "°C",
        "cpu.load" | "gpu.load" | "ram.load" | "vram.load" | "disk.load" => "%",
        "cpu.power" | "gpu.power" => " W",
        "cpu.clock" | "gpu.clock" => " MHz",
        "cpu.fan" | "gpu.fan" | "pump" => " RPM",
        "ram.used" | "vram.used" => " GB",
        "net.down" | "net.up" => " Mbps",
        "disk.read" | "disk.write" => " MB/s",
        _ => "",
    }
}
fn rgb(c: [u8; 3]) -> crate::renderer::modern::Rgb {
    (c[0], c[1], c[2])
}
fn fitted(
    s: &mut Surface,
    x: f32,
    y: f32,
    w: f32,
    text: &str,
    size: u16,
    color: crate::renderer::modern::Rgb,
    center: bool,
) {
    let mut size = size;
    while size > 8 && crate::renderer::modern::measure(text, size as f32, true) > w {
        size -= 1;
    }
    if center {
        s.center(x + w / 2.0, y, text, size, true, color)
    } else {
        s.text(x, y, text, size, true, color)
    }
}
fn graph_values(t: &TelemetrySnapshot, id: &str) -> Vec<f32> {
    match id {
        "cpu.temp" => t.history.cpu_temp.values(),
        "gpu.temp" => t.history.gpu_temp.values(),
        "cpu.load" => t.history.cpu_usage.values(),
        "gpu.load" => t.history.gpu_usage.values(),
        "net.down" => t.history.net_down.values(),
        "net.up" => t.history.net_up.values(),
        "cpu.fan" => t.history.cpu_fan.values(),
        "gpu.fan" => t.history.gpu_fan.values(),
        "pump" => t.history.pump.values(),
        _ => vec![],
    }
}
impl Custom {
    pub fn render(&mut self, c: &mut Canvas, t: &TelemetrySnapshot) {
        let mut s = Surface::from_rgb(&self.pixels);
        // One bounded history per sensor; hidden/missing readings never create fake samples.
        let sensors: std::collections::HashSet<_> = self
            .elements
            .iter()
            .filter(|e| e.kind == "graph")
            .map(|e| e.sensor.as_str())
            .collect();
        for id in sensors {
            if let Some(v) = metric(t, id).filter(|v| v.is_finite()) {
                let history = self.samples.entry(id.to_owned()).or_default();
                if history.len() == 120 {
                    history.pop_front();
                }
                history.push_back(v);
            }
        }
        for e in &self.elements {
            let v = metric(t, &e.sensor).filter(|v| v.is_finite());
            let suffix = if e.show_unit {e.unit.as_deref().unwrap_or(unit(&e.sensor))} else {""};
            let value = v
                .map(|v| format!("{:.*}{}", e.decimals as usize, v, suffix))
                .unwrap_or_else(|| "—".into());
            let label = e
                .label
                .clone()
                .unwrap_or_else(|| e.sensor.to_uppercase().replace('.', " "));
            let data = {
                let data = graph_values(t, &e.sensor);
                if data.len() > 1 {
                    data
                } else {
                    self.samples
                        .get(&e.sensor)
                        .map(|v| v.iter().copied().collect())
                        .unwrap_or_default()
                }
            };
            s.region(e.x, e.y, e.width, e.height, |s| {
                let (accent, ink, label_ink, track) = (
                    rgb(e.color),
                    rgb(e.value_color),
                    rgb(e.label_color.unwrap_or(e.color)),
                    rgb(e.track_color),
                );
                s.rect(
                    e.x,
                    e.y,
                    e.width,
                    e.height,
                    5.0,
                    rgb(e.background_color),
                    e.opacity,
                );
                let pad = 4.0;
                let x = e.x + pad;
                let w = (e.width - pad * 2.0).max(4.0);
                let mut y = e.y + pad;
                if e.kind == "panel" {
                    return;
                }
                if e.kind == "label" {
                    fitted(s, x, y, w, &e.text, e.size, ink, false);
                    return;
                }
                if e.kind == "ring" {
                    let r = (e.width.min(e.height) / 2.0 - e.stroke / 2.0 - pad).max(1.0);
                    let (cx, cy) = (e.x + e.width / 2.0, e.y + e.height / 2.0);
                    let a = std::f32::consts::PI * 0.75;
                    let sweep = std::f32::consts::PI * 1.5;
                    s.arc(cx, cy, r, a, sweep, e.stroke, track, 255);
                    if let Some(v) = v {
                        s.arc(
                            cx,
                            cy,
                            r,
                            a,
                            sweep * (v / e.max).clamp(0.0, 1.0),
                            e.stroke,
                            accent,
                            255,
                        )
                    }
                    if e.show_label {
                        fitted(
                            s,
                            x,
                            cy - e.label_size as f32 - 8.0,
                            w,
                            &label,
                            e.label_size,
                            label_ink,
                            true,
                        )
                    }
                    if e.show_value {
                        fitted(s, x, cy - 2.0, w, &value, e.size, ink, true)
                    }
                    return;
                }
                if e.show_label {
                    fitted(s, x, y, w, &label, e.label_size, label_ink, false);
                    y += e.label_size as f32 + 2.0;
                }
                if e.show_value {
                    fitted(s, x, y, w, &value, e.size, ink, false);
                    y += e.size as f32 + 3.0;
                }
                let h = (e.y + e.height - pad - y).max(0.0);
                if e.kind == "bar" && h >= 3.0 {
                    s.rect(x, y, w, h, h.min(8.0) / 2.0, track, 255);
                    if let Some(v) = v {
                        let fw = w * (v / e.max).clamp(0.0, 1.0);
                        if fw > 0.0 {
                            s.rect(x, y, fw, h, h.min(8.0) / 2.0, accent, 255)
                        }
                    }
                }
                if e.kind == "graph" && h >= 3.0 {
                    for i in 0..3 {
                        s.line(
                            x,
                            y + h * i as f32 / 2.0,
                            x + w,
                            y + h * i as f32 / 2.0,
                            1.0,
                            track,
                            160,
                        )
                    }
                    if data.len() < 2 {
                        fitted(s, x, y, w, "Waiting for samples", 10, label_ink, false)
                    } else {
                        let mut path = tiny_skia::PathBuilder::new();
                        for (i, v) in data.iter().enumerate() {
                            let xx = x + w * i as f32 / (data.len() - 1) as f32;
                            let yy = y + h - h * (v / e.max).clamp(0.0, 1.0);
                            if i == 0 {
                                path.move_to(xx, yy)
                            } else {
                                path.line_to(xx, yy)
                            }
                        }
                        if let Some(path) = path.finish() {
                            s.stroke(&path, e.stroke.min(6.0), accent, 255)
                        }
                    }
                }
            });
        }
        s.finish(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn element(kind: &str) -> Element {
        serde_json::from_value(serde_json::json!({"kind":kind,"sensor":"cpu.temp","x":15,"y":10,"width":130,"height":130,"size":28,"max":100,"color":[0,195,255]})).unwrap()
    }
    #[test]
    fn units_and_legacy_defaults() {
        assert_eq!(unit("cpu.temp"), "°C");
        assert_eq!(unit("gpu.power"), " W");
        assert_eq!(unit("ram.load"), "%");
        let e = element("text");
        assert!(e.show_value);
        assert_eq!(e.opacity, 0);
    }
    #[test]
    fn hud_modes_clip_and_missing_values_render() {
        for kind in ["text", "bar", "ring", "graph", "label", "panel"] {
            let mut e = element(kind);
            e.opacity = 180;
            e.text = "My computer".into();
            e.unit = Some("°C".into());
            let mut theme = Custom {
                pixels: vec![80; 320 * 240 * 3],
                elements: vec![e],
                samples: Default::default(),
            };
            for snapshot in [
                TelemetrySnapshot::default(),
                crate::telemetry::fake_telemetry(),
            ] {
                let mut c = Canvas::new(320, 240);
                theme.render(&mut c, &snapshot);
                assert_eq!(&c.as_rgb_bytes()[..3], &[80, 80, 80]);
                assert_ne!(
                    &c.as_rgb_bytes()[(20 * 320 + 20) * 3..(20 * 320 + 20) * 3 + 3],
                    &[80, 80, 80]
                );
            }
        }
    }
}
