//! Three compact instrument faces, drawn directly into the RGB framebuffer.
use super::{util, Theme};
use crate::{renderer::{canvas::Canvas, colors::Color, text::{draw_text, text_width}}, telemetry::TelemetrySnapshot};

pub struct Face(pub u8);
const BG: Color = Color(8, 17, 25);
const MUTED: Color = Color(125, 151, 165);
const TEAL: Color = Color(74, 235, 206);
const AMBER: Color = Color(255, 189, 87);
const BLUE: Color = Color(132, 174, 255);

fn centered(c: &mut Canvas, x: i32, y: i32, value: &str, scale: i32, color: Color) {
    draw_text(c, x - text_width(value, scale) / 2, y, value, scale, color);
}

impl Theme for Face {
    fn id(&self) -> &'static str { match self.0 { 0 => "orbit", 1 => "instrument", _ => "night" } }
    fn name(&self) -> &'static str { match self.0 { 0 => "Orbit / dual thermal", 1 => "Instrument / amber", _ => "Night / idle clock" } }
    fn render(&mut self, c: &mut Canvas, t: &TelemetrySnapshot) {
        c.fill(BG);
        let accent = match self.0 { 0 => TEAL, 1 => AMBER, _ => BLUE };
        draw_text(c, 14, 12, "PRC /", 1, accent);
        draw_text(c, 56, 12, &self.id().to_uppercase(), 1, Color::WHITE);
        draw_text(c, 250, 12, "DEMO", 1, MUTED);
        c.draw_hline(14, 30, 292, Color(37, 56, 68));
        match self.0 {
            0 => {
                for (x, label, temp, usage, color) in [(83, "CPU", t.cpu.temp_c, t.cpu.usage_pct, TEAL), (237, "GPU", t.gpu.temp_c, t.gpu.usage_pct, BLUE)] {
                    let start = std::f32::consts::PI * 0.75;
                    let sweep = std::f32::consts::PI * 1.5;
                    c.fill_ring_arc(x, 114, 62, 57, start, start + sweep, Color(29, 48, 60));
                    c.fill_ring_arc(x, 114, 62, 57, start, start + sweep * (temp.unwrap_or(0.0) / 100.0).clamp(0.0, 1.0), color);
                    centered(c, x, 78, label, 1, color);
                    centered(c, x, 100, &util::temp_c(temp), 3, Color::WHITE);
                    centered(c, x, 139, &util::pct(usage), 2, color);
                }
                centered(c, 160, 194, &format!("RAM {}  /  PUMP {}", util::pct(t.memory.pct()), util::rpm(t.cooling.pump_rpm)), 1, MUTED);
            }
            1 => {
                draw_text(c, 16, 47, "PROCESSOR TEMPERATURE", 1, MUTED);
                draw_text(c, 14, 67, &util::temp_c(t.cpu.temp_c), 7, AMBER);
                for i in 0..28 {
                    c.fill_rect(16 + i * 10, 132, 6, 14, if (i as f32) < t.cpu.usage_pct.unwrap_or(0.0).clamp(0.0, 100.0) * 0.28 { AMBER } else { Color(45, 42, 36) });
                }
                for (x, label, value) in [(16, "LOAD", util::pct(t.cpu.usage_pct)), (120, "GPU", util::temp_c(t.gpu.temp_c)), (224, "POWER", util::watts(t.cpu.power_w))] {
                    draw_text(c, x, 163, label, 1, MUTED);
                    draw_text(c, x, 179, &value, 2, Color::WHITE);
                }
            }
            _ => {
                for (x,y,r) in [(27,57,1),(289,70,2),(267,158,1),(42,162,2)] { c.fill_circle(x,y,r,BLUE); }
                c.draw_circle(160, 111, 72, Color(28, 44, 66));
                c.draw_circle(160, 111, 75, Color(18, 32, 49));
                centered(c, 160, 86, &util::hm(&t.system.time_hm), 5, Color::WHITE);
                centered(c, 160, 137, t.system.date_ymd.as_deref().unwrap_or("---- -- --"), 1, BLUE);
                centered(c, 160, 193, &format!("CPU {}   GPU {}", util::temp_c(t.cpu.temp_c), util::temp_c(t.gpu.temp_c)), 1, MUTED);
            }
        }
        c.draw_hline(14, 214, 292, Color(37, 56, 68));
        draw_text(c, 14, 224, "320 X 240 / PREVIEW", 1, MUTED);
        c.fill_circle(300, 227, 2, accent);
    }
}
