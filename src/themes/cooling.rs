//! PAGE 09 — fans & pump: RPM readouts with mini graphs. "--" if absent.

use crate::renderer::canvas::Canvas;
use crate::renderer::colors::Color;
use crate::renderer::graph::sparkline;
use crate::renderer::text::draw_text;
use crate::telemetry::TelemetrySnapshot;

use super::util;
use super::Theme;

pub struct Cooling;

fn column(canvas: &mut Canvas, x: i32, name: &str, rpm: Option<u32>, hist: &[f32], accent: Color) {
    canvas.fill_rect(x, 34, 96, 194, Color::PANEL);
    util::label(canvas, x + 6, 40, name, accent);
    draw_text(canvas, x + 6, 56, &util::rpm(rpm), 3, Color::WHITE);
    util::label(canvas, x + 6, 84, "RPM", Color::FAINT);
    // Tiny bar scaled to 3000 RPM.
    let frac = rpm.map(|r| (r as f32 / 3000.0).clamp(0.0, 1.0)).unwrap_or(0.0);
    canvas.progress_bar(x + 6, 100, 84, 8, frac, accent, Color::BAR_BG);
    util::label(canvas, x + 6, 116, "TREND", Color::FAINT);
    sparkline(canvas, x + 6, 130, 84, 90, hist, accent, None);
}

impl Theme for Cooling {
    fn id(&self) -> &'static str {
        "cooling"
    }
    fn name(&self) -> &'static str {
        "Fans & Pump"
    }

    fn render(&mut self, canvas: &mut Canvas, t: &TelemetrySnapshot) {
        canvas.fill(Color::BLACK);
        util::header(canvas, "COOLING");

        column(canvas, 8, "CPU FAN", t.cooling.cpu_fan_rpm, &t.history.cpu_fan.values(), Color::CPU);
        column(canvas, 112, "GPU FAN", t.cooling.gpu_fan_rpm, &t.history.gpu_fan.values(), Color::GPU);
        column(canvas, 216, "PUMP", t.cooling.pump_rpm, &t.history.pump.values(), Color::RAM);
    }
}
