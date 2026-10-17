//! Reference-specific layouts over cleaned supplied HUD artwork.
use super::Theme;
use crate::{
    renderer::{
        modern::{Rgb, Surface, WHITE},
        Canvas,
    },
    telemetry::TelemetrySnapshot,
};
const RED: Rgb = (255, 69, 42);
const GREEN: Rgb = (75, 228, 106);
const BLUE: Rgb = (54, 189, 255);
const PURPLE: Rgb = (181, 83, 255);
const BLACK: Rgb = (3, 4, 7);
pub struct SheetFace {
    pub id: &'static str,
    pub name: &'static str,
    pub artwork: &'static [u8],
    pub layout: usize,
    pixels: Option<Vec<u8>>,
}
impl SheetFace {
    pub fn with_pixels(layout: usize, pixels: Vec<u8>) -> Self {
        Self { id: "custom", name: "Custom", artwork: &[], layout, pixels: Some(pixels) }
    }
    pub fn new(
        id: &'static str,
        name: &'static str,
        artwork: &'static [u8],
        layout: usize,
    ) -> Self {
        Self {
            id,
            name,
            artwork,
            layout,
            pixels: None,
        }
    }
}
fn val(v: Option<f32>, unit: &str) -> String {
    v.filter(|v| v.is_finite())
        .map(|v| format!("{v:.0}{unit}"))
        .unwrap_or_else(|| "—".into())
}
fn gb(v: Option<u64>) -> String {
    v.map(|v| format!("{:.1} GB", v as f32 / 1024.0))
        .unwrap_or_else(|| "—".into())
}
#[derive(Clone, Copy)]
struct Metric {
    label: &'static str,
    temp: Option<f32>,
    load: Option<f32>,
    power: Option<f32>,
    color: Rgb,
}
fn readings(s: &mut Surface, x: f32, y: f32, m: Metric, size: u16) {
    s.text(x, y, m.label, 17, true, WHITE);
    s.text(x, y + 20.0, &val(m.temp, "°C"), size, true, WHITE);
    s.text(x, y + 57.0, &val(m.load, "%"), 16, true, m.color);
    s.text(x, y + 76.0, &val(m.power, " W"), 17, true, WHITE);
}
fn ring(s: &mut Surface, cx: f32, cy: f32, r: f32, m: Metric, power: bool) {
    s.circle(cx, cy, r + 6.0, BLACK, 235);
    let a = std::f32::consts::PI * 0.73;
    let sweep = std::f32::consts::PI * 1.54;
    s.arc(cx, cy, r + 5.0, a, sweep, 1.0, m.color, 160);
    s.arc(cx, cy, r, a, sweep, 8.0, (34, 34, 36), 255);
    if let Some(v) = m.temp.filter(|v| v.is_finite()) {
        s.arc(
            cx,
            cy,
            r,
            a,
            sweep * (v / 100.0).clamp(0.0, 1.0),
            8.0,
            m.color,
            255,
        )
    }
    s.center(cx, cy - r + 10.0, m.label, 18, true, WHITE);
    s.fit(cx, cy - 17.0, &val(m.temp, "°C"), 36, r * 1.75, true, WHITE);
    s.center(cx, cy + 21.0, &val(m.load, "%"), 17, true, WHITE);
    if power {
        s.center(cx, cy + r - 10.0, &val(m.power, " W"), 19, true, WHITE)
    }
}
fn mem(
    s: &mut Surface,
    x: f32,
    y: f32,
    w: f32,
    label: &str,
    used: Option<u64>,
    pct: Option<f32>,
    color: Rgb,
    light: bool,
) {
    let ink = if light { (29, 25, 20) } else { WHITE };
    s.rect(
        x - 3.0,
        y - 2.0,
        w + 6.0,
        34.0,
        3.0,
        if light { (255, 233, 170) } else { BLACK },
        220,
    );
    if label.is_empty() {
        s.text(x, y, &gb(used), 14, true, ink)
    } else {
        s.text(x, y, label, 14, true, color);
        s.text(x + 44.0, y, &gb(used), 14, true, ink)
    }
    s.bar(x, y + 19.0, w - 40.0, 9.0, pct.map(|v| v / 100.0), color);
    s.text(x + w - 36.0, y + 14.0, &val(pct, "%"), 15, true, ink);
}
fn vp(t: &TelemetrySnapshot) -> Option<f32> {
    t.gpu
        .vram_used_mb
        .zip(t.gpu.vram_total_mb)
        .filter(|(_, n)| *n > 0)
        .map(|(v, n)| v as f32 / n as f32 * 100.0)
}
fn footer(s: &mut Surface, t: &TelemetrySnapshot, y: f32) {
    mem(
        s,
        9.0,
        y,
        142.0,
        "",
        t.memory.used_mb,
        t.memory.pct(),
        BLUE,
        false,
    );
    mem(
        s,
        173.0,
        y,
        138.0,
        "",
        t.gpu.vram_used_mb.map(u64::from),
        vp(t),
        PURPLE,
        false,
    )
}
fn bars(s: &mut Surface, x: f32, y: f32, w: f32, m: Metric, light: bool) {
    let ink = if light { (29, 25, 20) } else { WHITE };
    s.rect(
        x - 4.0,
        y - 3.0,
        w + 8.0,
        43.0,
        3.0,
        if light { (255, 233, 170) } else { BLACK },
        220,
    );
    s.text(x, y, m.label, 13, true, m.color);
    s.text(x, y + 14.0, &val(m.temp, "°C"), 20, true, ink);
    s.bar(
        x + 52.0,
        y + 6.0,
        w - 90.0,
        7.0,
        m.load.map(|v| v / 100.0),
        m.color,
    );
    s.bar(
        x + 52.0,
        y + 22.0,
        w - 90.0,
        7.0,
        m.power.map(|v| v / 250.0),
        m.color,
    );
    s.text(x + w - 34.0, y + 1.0, &val(m.load, "%"), 12, true, ink);
    s.text(x + w - 34.0, y + 18.0, &val(m.power, "W"), 12, true, ink);
}
impl Theme for SheetFace {
    fn id(&self) -> &'static str {
        self.id
    }
    fn name(&self) -> &'static str {
        self.name
    }
    fn render(&mut self, c: &mut Canvas, t: &TelemetrySnapshot) {
        let px = self.pixels.get_or_insert_with(|| {
            image::load_from_memory(self.artwork)
                .expect("Bundled WebP")
                .to_rgb8()
                .into_raw()
        });
        let mut s = Surface::from_rgb(px);
        let cpu = Metric {
            label: "CPU",
            temp: t.cpu.temp_c,
            load: t.cpu.usage_pct,
            power: t.cpu.power_w,
            color: RED,
        };
        let gpu = Metric {
            label: "GPU",
            temp: t.gpu.temp_c,
            load: t.gpu.usage_pct,
            power: t.gpu.power_w,
            color: GREEN,
        };
        match self.layout {
            0 => {
                ring(&mut s, 85.0, 168.0, 58.0, cpu, true);
                ring(&mut s, 237.0, 168.0, 58.0, gpu, true)
            }
            1 => {
                s.rect(16.0, 79.0, 179.0, 110.0, 4.0, BLACK, 205);
                readings(&mut s, 29.0, 83.0, Metric { color: BLUE, ..cpu }, 35);
                readings(&mut s, 131.0, 83.0, Metric { color: BLUE, ..gpu }, 35);
                footer(&mut s, t, 202.0)
            }
            2 => {
                ring(&mut s, 71.0, 131.0, 51.0, cpu, false);
                ring(&mut s, 186.0, 131.0, 51.0, gpu, false);
                footer(&mut s, t, 202.0)
            }
            3 => {
                bars(&mut s, 18.0, 79.0, 151.0, cpu, false);
                bars(&mut s, 18.0, 123.0, 151.0, gpu, false);
                mem(
                    &mut s,
                    18.0,
                    172.0,
                    210.0,
                    "RAM",
                    t.memory.used_mb,
                    t.memory.pct(),
                    BLUE,
                    false,
                );
                mem(
                    &mut s,
                    18.0,
                    207.0,
                    210.0,
                    "VRAM",
                    t.gpu.vram_used_mb.map(u64::from),
                    vp(t),
                    PURPLE,
                    false,
                )
            }
            4 => {
                s.rect(20.0, 99.0, 280.0, 97.0, 7.0, BLACK, 177);
                readings(&mut s, 56.0, 104.0, cpu, 39);
                readings(&mut s, 208.0, 104.0, gpu, 39);
                footer(&mut s, t, 202.0)
            }
            5 => {
                s.rect(13.0, 148.0, 148.0, 91.0, 3.0, BLACK, 221);
                for (y, m) in [(152.0, cpu), (194.0, gpu)] {
                    s.text(17.0, y, m.label, 13, true, GREEN);
                    s.text(17.0, y + 14.0, &val(m.temp, "°C"), 21, true, WHITE);
                    s.bar(73.0, y + 9.0, 72.0, 8.0, m.load.map(|v| v / 100.0), GREEN);
                    s.bar(73.0, y + 24.0, 72.0, 8.0, m.power.map(|v| v / 250.0), GREEN)
                }
                mem(
                    &mut s,
                    181.0,
                    159.0,
                    121.0,
                    "",
                    t.memory.used_mb,
                    t.memory.pct(),
                    BLUE,
                    false,
                );
                mem(
                    &mut s,
                    181.0,
                    202.0,
                    121.0,
                    "",
                    t.gpu.vram_used_mb.map(u64::from),
                    vp(t),
                    PURPLE,
                    false,
                )
            }
            6 => {
                s.rect(237.0, 9.0, 83.0, 231.0, 0.0, BLACK, 248);
                for (y, m) in [(19.0, cpu), (70.0, gpu)] {
                    s.text(246.0, y, m.label, 13, true, m.color);
                    s.text(246.0, y + 18.0, &val(m.temp, "°C"), 25, true, WHITE)
                }
                for (y, label, pct, color) in [
                    (122.0, "RAM", t.memory.pct(), BLUE),
                    (176.0, "VRAM", vp(t), PURPLE),
                ] {
                    s.text(246.0, y, label, 13, true, color);
                    s.text(246.0, y + 14.0, &val(pct, "%"), 22, true, color);
                    s.bar(246.0, y + 38.0, 62.0, 11.0, pct.map(|v| v / 100.0), color)
                }
            }
            7 => {
                bars(&mut s, 171.0, 72.0, 140.0, cpu, false);
                bars(&mut s, 171.0, 119.0, 140.0, gpu, false);
                mem(
                    &mut s,
                    171.0,
                    163.0,
                    140.0,
                    "",
                    t.memory.used_mb,
                    t.memory.pct(),
                    BLUE,
                    false,
                );
                mem(
                    &mut s,
                    171.0,
                    203.0,
                    140.0,
                    "",
                    t.gpu.vram_used_mb.map(u64::from),
                    vp(t),
                    PURPLE,
                    false,
                )
            }
            8 | 10 => {
                let (x1, x2) = if self.layout == 8 {
                    (22.0, 239.0)
                } else {
                    (22.0, 235.0)
                };
                s.rect(x1 - 9.0, 101.0, 93.0, 94.0, 4.0, BLACK, 170);
                s.rect(x2 - 7.0, 101.0, 82.0, 94.0, 4.0, BLACK, 170);
                readings(
                    &mut s,
                    x1,
                    104.0,
                    Metric {
                        color: if self.layout == 8 { BLUE } else { RED },
                        ..cpu
                    },
                    30,
                );
                readings(&mut s, x2, 104.0, gpu, 30);
                footer(&mut s, t, 202.0)
            }
            9 => {
                for (x, label, v, unit, pct, color) in [
                    (12.0, "CPU", t.cpu.temp_c, "°C", t.cpu.usage_pct, RED),
                    (89.0, "GPU", t.gpu.temp_c, "°C", t.gpu.usage_pct, BLUE),
                    (166.0, "RAM", t.memory.pct(), "%", None, BLUE),
                    (244.0, "VRAM", vp(t), "%", None, PURPLE),
                ] {
                    s.rect(x - 5.0, 150.0, 76.0, 90.0, 0.0, BLACK, 206);
                    s.text(x, 151.0, label, 14, true, color);
                    s.text(
                        x,
                        172.0,
                        &val(v, unit),
                        26,
                        true,
                        if label == "VRAM" { PURPLE } else { WHITE },
                    );
                    if pct.is_some() {
                        s.text(x, 205.0, &val(pct, "%"), 17, true, WHITE)
                    }
                }
            }
            11 | 12 => {
                let (x, y, w) = if self.layout == 11 {
                    (23.0, 83.0, 158.0)
                } else {
                    (23.0, 102.0, 143.0)
                };
                bars(
                    &mut s,
                    x,
                    y,
                    w,
                    Metric {
                        color: if self.layout == 11 {
                            BLUE
                        } else {
                            (255, 166, 55)
                        },
                        ..cpu
                    },
                    false,
                );
                bars(
                    &mut s,
                    x,
                    if self.layout == 12 { 137.0 } else { y + 43.0 },
                    w,
                    Metric {
                        color: if self.layout == 11 { BLUE } else { GREEN },
                        ..gpu
                    },
                    false,
                );
                mem(
                    &mut s,
                    x,
                    if self.layout == 12 { 174.0 } else { y + 88.0 },
                    w,
                    "RAM",
                    t.memory.used_mb,
                    t.memory.pct(),
                    BLUE,
                    false,
                );
                mem(
                    &mut s,
                    x,
                    if self.layout == 12 { 209.0 } else { y + 123.0 },
                    w,
                    "VRAM",
                    t.gpu.vram_used_mb.map(u64::from),
                    vp(t),
                    PURPLE,
                    false,
                )
            }
            13 | 14 => {
                ring(&mut s, 83.0, 137.0, 51.0, cpu, false);
                ring(
                    &mut s,
                    235.0,
                    137.0,
                    51.0,
                    Metric {
                        color: PURPLE,
                        ..gpu
                    },
                    false,
                );
                footer(&mut s, t, 202.0)
            }
            15 => {
                bars(&mut s, 114.0, 69.0, 128.0, cpu, true);
                bars(&mut s, 114.0, 112.0, 128.0, gpu, true);
                mem(
                    &mut s,
                    114.0,
                    153.0,
                    128.0,
                    "",
                    t.memory.used_mb,
                    t.memory.pct(),
                    BLUE,
                    true,
                );
                mem(
                    &mut s,
                    114.0,
                    190.0,
                    128.0,
                    "",
                    t.gpu.vram_used_mb.map(u64::from),
                    vp(t),
                    PURPLE,
                    true,
                )
            }
            _ => unreachable!("16 reference layouts"),
        };
        s.finish(c);
    }
}
