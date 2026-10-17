//! Modern distance-readable LCD faces using antialiased vectors and TrueType text.
use super::{util, Theme};
use crate::{
    renderer::{
        modern::{Rgb, Surface, BLUE, GOLD, GREEN, MUTED, PURPLE, RED, WHITE},
        Canvas,
    },
    telemetry::TelemetrySnapshot,
};
use tiny_skia::PathBuilder;

pub struct DistanceFace(pub &'static str, pub &'static str);
const TRACK: Rgb = (44, 60, 80);

fn temp(value: Option<f32>) -> String {
    value
        .map(|v| format!("{v:.0}°C"))
        .unwrap_or_else(|| "—".into())
}
fn frac(value: Option<f32>) -> Option<f32> {
    value.map(|v| v / 100.0)
}
fn rate(value: Option<f32>) -> String {
    match value.filter(|v| v.is_finite()) {
        Some(v) if v >= 1000.0 => format!("{:.1}G", v / 1000.0),
        Some(v) => format!("{v:.0}"),
        None => "—".into(),
    }
}
fn header(s: &mut Surface, title: &str, demo: bool, accent: Rgb) {
    s.text(13.0, 13.0, "PRC", 20, true, WHITE);
    s.line(55.0, 14.0, 55.0, 29.0, 1.0, TRACK, 255);
    s.text(68.0, 14.0, title, 18, false, MUTED);
    s.rect(258.0, 9.0, 53.0, 27.0, 13.5, accent, 25);
    s.center(
        284.0,
        14.0,
        if demo { "Demo" } else { "Live" },
        18,
        true,
        accent,
    );
}
fn ring(s: &mut Surface, cx: f32, cy: f32, r: f32, fraction: Option<f32>, accent: Rgb, width: f32) {
    let start = std::f32::consts::PI * 0.75;
    let sweep = std::f32::consts::PI * 1.5;
    s.arc(cx, cy, r, start, sweep, width, TRACK, 255);
    if let Some(f) = fraction.filter(|v| v.is_finite()) {
        s.arc(
            cx,
            cy,
            r,
            start,
            sweep * f.clamp(0.0, 1.0),
            width + 6.0,
            accent,
            18,
        );
        s.arc(
            cx,
            cy,
            r,
            start,
            sweep * f.clamp(0.0, 1.0),
            width,
            accent,
            255,
        );
    }
}
fn dual(s: &mut Surface, t: &TelemetrySnapshot, load: bool, colors: (Rgb, Rgb), cards: bool) {
    for (cx, label, temperature, usage, accent) in [
        (82.0, "CPU", t.cpu.temp_c, t.cpu.usage_pct, colors.0),
        (238.0, "GPU", t.gpu.temp_c, t.gpu.usage_pct, colors.1),
    ] {
        if cards {
            s.card(cx - 74.0, 47.0, 148.0, 185.0)
        }
        s.center(cx, 61.0, label, 21, true, accent);
        ring(
            s,
            cx,
            137.0,
            54.0,
            frac(if load { usage } else { temperature }),
            accent,
            8.0,
        );
        let value = if load {
            util::pct(usage)
        } else {
            temp(temperature)
        };
        s.fit(cx, 118.0, &value, 42, 128.0, true, WHITE);
        if load {
            s.center(cx, 200.0, &temp(temperature), 24, true, MUTED)
        } else {
            s.center(
                cx,
                201.0,
                &format!("{} load", util::pct(usage)),
                20,
                false,
                MUTED,
            )
        }
    }
}
fn chart(s: &mut Surface, x: f32, y: f32, w: f32, h: f32, data: &[f32], accent: Rgb) {
    for i in 0..3 {
        let yy = y + h * i as f32 / 2.0;
        s.line(x, yy, x + w, yy, 1.0, TRACK, 140)
    }
    if data.len() < 2 {
        s.center(
            x + w / 2.0,
            y + h / 2.0 - 10.0,
            "Waiting for data",
            20,
            false,
            MUTED,
        );
        return;
    }
    let low = data.iter().copied().fold(f32::INFINITY, f32::min) - 3.0;
    let high = data.iter().copied().fold(f32::NEG_INFINITY, f32::max) + 3.0;
    let span = (high - low).max(1.0);
    let mut line = PathBuilder::new();
    let mut area = PathBuilder::new();
    area.move_to(x, y + h);
    for (i, value) in data.iter().enumerate() {
        let xx = x + w * i as f32 / (data.len() - 1) as f32;
        let yy = y + h - (value - low) / span * h;
        if i == 0 {
            line.move_to(xx, yy)
        } else {
            line.line_to(xx, yy)
        }
        area.line_to(xx, yy);
    }
    area.line_to(x + w, y + h);
    area.close();
    s.path(&area.finish().unwrap(), accent, 23);
    s.stroke(&line.finish().unwrap(), 2.5, accent, 255);
}
fn mountains(s: &mut Surface, accent: Rgb, clock: bool) {
    if clock {
        s.circle(285.0, 174.0, 23.0, accent, 210);
    } else {
        s.circle(160.0, 132.0, 23.0, accent, 180);
    }
    for (points, color) in [
        (
            vec![
                (0.0, 193.0),
                (38.0, 159.0),
                (80.0, 188.0),
                (127.0, 141.0),
                (177.0, 180.0),
                (213.0, 155.0),
                (261.0, 191.0),
                (320.0, 153.0),
            ],
            (30, 42, 65),
        ),
        (
            vec![
                (0.0, 215.0),
                (53.0, 184.0),
                (98.0, 209.0),
                (163.0, 172.0),
                (202.0, 201.0),
                (258.0, 174.0),
                (320.0, 204.0),
            ],
            (14, 23, 40),
        ),
    ] {
        let mut p = PathBuilder::new();
        p.move_to(0.0, 240.0);
        for (x, y) in points {
            p.line_to(x, y)
        }
        p.line_to(320.0, 240.0);
        p.close();
        s.path(&p.finish().unwrap(), color, 255);
    }
}
fn hero(
    s: &mut Surface,
    label: &str,
    value: &str,
    fraction: Option<f32>,
    accent: Rgb,
    footer: &str,
) {
    s.card(8.0, 47.0, 304.0, 185.0);
    s.center(160.0, 64.0, label, 22, false, accent);
    s.fit(160.0, 99.0, value, 74, 274.0, true, WHITE);
    s.bar(26.0, 174.0, 268.0, 12.0, fraction, accent);
    s.fit(160.0, 203.0, footer, 22, 268.0, false, MUTED);
}

impl Theme for DistanceFace {
    fn id(&self) -> &'static str {
        self.0
    }
    fn name(&self) -> &'static str {
        self.1
    }
    fn render(&mut self, c: &mut Canvas, t: &TelemetrySnapshot) {
        let mut s = Surface::new();
        let (title, accent) = match self.0 {
            "main" => ("Thermals", RED),
            "central" => ("CPU core", RED),
            "performance" => ("Performance", BLUE),
            "system" => ("System", BLUE),
            "media" => ("Now playing", PURPLE),
            "network" => ("Network", BLUE),
            "servers" => ("Home lab", GREEN),
            "temperature" => ("History", RED),
            "cooling" => ("Cooling", BLUE),
            "storage" => ("Storage", BLUE),
            "clock" => ("Clock", BLUE),
            "terminal" => ("Telemetry", GREEN),
            "art" => ("Horizon", GOLD),
            "cyberpunk" => ("Neon city", PURPLE),
            "minimal" => ("Minimal", RED),
            "alert" => ("GPU thermal", RED),
            "orbit" => ("Orbit", (80, 229, 216)),
            "instrument" => ("Instrument", GOLD),
            "night" => ("Night", PURPLE),
            _ => unreachable!(),
        };
        header(&mut s, title, t.demo, accent);
        match self.0 {
            "main" => dual(&mut s, t, false, (RED, GREEN), true),
            "orbit" => {
                s.arc(
                    160.0,
                    143.0,
                    91.0,
                    0.0,
                    std::f32::consts::TAU,
                    1.0,
                    BLUE,
                    40,
                );
                dual(&mut s, t, false, ((80, 229, 216), BLUE), false);
            }
            "cyberpunk" => {
                for i in 0..12 {
                    let x = i as f32 * 29.0;
                    let h = 34.0 + (i % 4) as f32 * 16.0;
                    s.rect(x, 240.0 - h, 22.0, h, 2.0, (24, 18, 46), 255);
                    s.line(
                        x + 5.0,
                        243.0 - h,
                        x + 5.0,
                        230.0,
                        2.0,
                        if i % 2 == 0 { PURPLE } else { BLUE },
                        120,
                    );
                }
                dual(&mut s, t, false, (PURPLE, BLUE), false);
            }
            "art" => {
                mountains(&mut s, GOLD, false);
                dual(&mut s, t, false, (GOLD, (118, 204, 238)), false);
            }
            "performance" => dual(&mut s, t, true, (RED, GREEN), true),
            "central" => {
                ring(&mut s, 113.0, 137.0, 77.0, frac(t.cpu.temp_c), RED, 11.0);
                s.center(113.0, 73.0, "CPU", 22, true, RED);
                s.fit(113.0, 117.0, &temp(t.cpu.temp_c), 58, 145.0, true, WHITE);
                s.center(113.0, 179.0, &util::pct(t.cpu.usage_pct), 26, true, MUTED);
                for (y, label, value, color) in [
                    (50.0, "GPU", temp(t.gpu.temp_c), GREEN),
                    (145.0, "RAM", util::pct(t.memory.pct()), BLUE),
                ] {
                    s.card(220.0, y, 92.0, 87.0);
                    s.center(266.0, y + 12.0, label, 20, true, color);
                    s.fit(266.0, y + 42.0, &value, 31, 79.0, true, WHITE);
                }
            }
            "instrument" => hero(
                &mut s,
                "CPU temperature",
                &temp(t.cpu.temp_c),
                frac(t.cpu.temp_c),
                GOLD,
                &format!(
                    "{} load   {}",
                    util::pct(t.cpu.usage_pct),
                    util::watts(t.cpu.power_w)
                ),
            ),
            "alert" => {
                s.rect(5.0, 44.0, 310.0, 191.0, 20.0, RED, 20);
                hero(
                    &mut s,
                    "GPU temperature",
                    &temp(t.gpu.temp_c),
                    frac(t.gpu.temp_c),
                    RED,
                    &format!("{} GPU load", util::pct(t.gpu.usage_pct)),
                );
            }
            "system" | "terminal" => {
                for (i, label, value, fraction, color) in [
                    (
                        0,
                        "CPU",
                        util::pct(t.cpu.usage_pct),
                        frac(t.cpu.usage_pct),
                        RED,
                    ),
                    (
                        1,
                        "GPU",
                        util::pct(t.gpu.usage_pct),
                        frac(t.gpu.usage_pct),
                        GREEN,
                    ),
                    (
                        2,
                        "RAM",
                        util::pct(t.memory.pct()),
                        frac(t.memory.pct()),
                        BLUE,
                    ),
                ] {
                    let y = 48.0 + i as f32 * 63.0;
                    s.card(8.0, y, 304.0, 57.0);
                    s.text(24.0, y + 17.0, label, 22, true, color);
                    s.fit(151.0, y + 10.0, &value, 36, 102.0, true, WHITE);
                    s.bar(210.0, y + 24.0, 83.0, 9.0, fraction, color);
                }
            }
            "clock" | "night" => {
                mountains(&mut s, if self.0 == "night" { PURPLE } else { RED }, true);
                s.fit(
                    160.0,
                    58.0,
                    &util::hm(&t.system.time_hm),
                    83,
                    294.0,
                    true,
                    WHITE,
                );
                s.center(
                    160.0,
                    149.0,
                    t.system.date_ymd.as_deref().unwrap_or("Date unavailable"),
                    24,
                    false,
                    MUTED,
                );
                s.center(
                    160.0,
                    210.0,
                    &format!("CPU {}", temp(t.cpu.temp_c)),
                    22,
                    true,
                    WHITE,
                );
            }
            "network" => {
                for (x, label, value, data, color) in [
                    (
                        8.0,
                        "Download",
                        t.network.down_mbps,
                        t.history.net_down.values(),
                        BLUE,
                    ),
                    (
                        164.0,
                        "Upload",
                        t.network.up_mbps,
                        t.history.net_up.values(),
                        PURPLE,
                    ),
                ] {
                    s.card(x, 47.0, 148.0, 144.0);
                    s.center(x + 74.0, 61.0, label, 20, false, color);
                    s.fit(x + 74.0, 91.0, &rate(value), 44, 123.0, true, WHITE);
                    s.center(x + 74.0, 133.0, "Mbps", 18, false, MUTED);
                    chart(&mut s, x + 13.0, 161.0, 122.0, 18.0, &data, color);
                }
                s.fit(
                    160.0,
                    210.0,
                    t.network
                        .local_ip
                        .as_deref()
                        .unwrap_or("Network unavailable"),
                    22,
                    294.0,
                    false,
                    MUTED,
                );
            }
            "temperature" | "minimal" => {
                if self.0 == "temperature" {
                    s.card(8.0, 47.0, 304.0, 185.0)
                }
                s.text(24.0, 65.0, "CPU", 23, true, RED);
                s.fit(238.0, 57.0, &temp(t.cpu.temp_c), 42, 135.0, true, WHITE);
                chart(
                    &mut s,
                    25.0,
                    113.0,
                    270.0,
                    74.0,
                    &t.history.cpu_temp.values(),
                    RED,
                );
                s.text(24.0, 206.0, "GPU", 21, true, GREEN);
                s.fit(250.0, 203.0, &temp(t.gpu.temp_c), 25, 100.0, true, WHITE);
            }
            "storage" => {
                let disk = t.storage.first();
                hero(
                    &mut s,
                    "Disk used",
                    &util::pct(disk.and_then(|d| d.pct())),
                    frac(disk.and_then(|d| d.pct())),
                    BLUE,
                    &format!("RAM {}", util::pct(t.memory.pct())),
                );
            }
            "cooling" => {
                let generic = t.cooling.cpu_fan_rpm.is_none()
                    && t.cooling.pump_rpm.is_none()
                    && !t.cooling.motherboard_fans.is_empty();
                let values = if generic {
                    let running: Vec<_> = t
                        .cooling
                        .motherboard_fans
                        .iter()
                        .filter(|(_, rpm)| *rpm > 0)
                        .collect();
                    let selected = if running.len() > 3 {
                        vec![running[0], running[1], running[running.len() - 1]]
                    } else {
                        running
                    };
                    selected
                        .iter()
                        .map(|(name, rpm)| (name.clone(), Some(*rpm)))
                        .collect::<Vec<_>>()
                } else {
                    vec![
                        ("CPU fan".into(), t.cooling.cpu_fan_rpm),
                        ("GPU fan".into(), t.cooling.gpu_fan_rpm),
                        ("Pump".into(), t.cooling.pump_rpm),
                    ]
                };
                for (i, (name, rpm)) in values.iter().enumerate() {
                    let x = 8.0 + i as f32 * 104.0;
                    let color = [RED, GREEN, BLUE][i];
                    s.card(x, 47.0, 96.0, 145.0);
                    s.fit(x + 48.0, 64.0, name, 20, 80.0, true, color);
                    s.fit(x + 48.0, 107.0, &util::rpm(*rpm), 34, 82.0, true, WHITE);
                    s.center(x + 48.0, 154.0, "RPM", 20, false, MUTED);
                }
                s.center(
                    160.0,
                    211.0,
                    if generic {
                        "Numbered motherboard fans"
                    } else {
                        "Live cooling telemetry"
                    },
                    20,
                    false,
                    MUTED,
                );
            }
            "media" => {
                if let Some(m) = &t.media {
                    s.card(8.0, 47.0, 304.0, 185.0);
                    s.circle(56.0, 91.0, 26.0, PURPLE, 180);
                    s.circle(56.0, 91.0, 14.0, (17, 20, 38), 255);
                    s.circle(56.0, 91.0, 4.0, WHITE, 255);
                    s.fit(191.0, 69.0, &m.title, 29, 212.0, true, WHITE);
                    s.fit(191.0, 107.0, &m.artist, 20, 212.0, false, MUTED);
                    s.bar(
                        25.0,
                        163.0,
                        270.0,
                        9.0,
                        Some(m.elapsed_s as f32 / m.duration_s.max(1) as f32),
                        PURPLE,
                    );
                    s.center(
                        160.0,
                        202.0,
                        if m.playing { "Playing" } else { "Paused" },
                        23,
                        true,
                        PURPLE,
                    );
                } else {
                    s.center(160.0, 93.0, "No media", 43, true, WHITE);
                    s.center(160.0, 162.0, "Media provider unavailable", 21, false, MUTED);
                }
            }
            "servers" => {
                s.card(8.0, 47.0, 304.0, 185.0);
                s.center(160.0, 83.0, "Home lab", 31, true, GREEN);
                s.center(160.0, 135.0, "No provider", 38, true, WHITE);
                s.center(160.0, 202.0, "Configure through CLI", 21, false, MUTED);
            }
            _ => unreachable!(),
        }
        s.finish(c);
    }
}
