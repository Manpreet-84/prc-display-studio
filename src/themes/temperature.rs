//! PAGE 08 — temperature history: 60s window with current/avg/max.

use crate::renderer::canvas::Canvas;
use crate::renderer::colors::Color;
use crate::renderer::graph::sparkline;
use crate::renderer::text::draw_text;
use crate::telemetry::TelemetrySnapshot;

use super::util;
use super::Theme;

pub struct Temperature;

impl Theme for Temperature {
    fn id(&self) -> &'static str {
        "temperature"
    }
    fn name(&self) -> &'static str {
        "Temperature History"
    }

    fn render(&mut self, canvas: &mut Canvas, t: &TelemetrySnapshot) {
        canvas.fill(Color::BLACK);
        util::header(canvas, "TEMP 60S");

        // Primary: CPU; GPU shown as second line.
        let cpu = t.history.cpu_temp.values();
        let gpu = t.history.gpu_temp.values();

        util::label(canvas, 10, 34, "CPU C", Color::CPU);
        util::label(canvas, 70, 34, "GPU C", Color::GPU);

        sparkline(canvas, 10, 48, 300, 120, &cpu, Color::CPU, Some(Color::CPU_DIM));
        // Overlay GPU line in same window (thin, no fill).
        if gpu.len() >= 2 {
            let min = cpu.iter().chain(gpu.iter()).cloned().fold(f32::INFINITY, f32::min);
            let max = cpu.iter().chain(gpu.iter()).cloned().fold(f32::NEG_INFINITY, f32::max);
            let span = (max - min).max(1e-6);
            let (x, y, w, h) = (10, 48, 300, 120);
            let n = gpu.len();
            let mapx = |i: usize| x + 1 + ((i as f32 / (n - 1) as f32) * (w - 3) as f32).round() as i32;
            let mapy = |v: f32| y + h - 1 - (((v - min) / span) * (h - 3) as f32).round() as i32;
            for i in 0..n - 1 {
                canvas.draw_line(mapx(i), mapy(gpu[i]), mapx(i + 1), mapy(gpu[i + 1]), Color::GPU);
            }
        }

        // Stats row.
        canvas.fill_rect(8, 178, 304, 54, Color::PANEL);
        util::label(canvas, 14, 184, "CURRENT", Color::FAINT);
        util::label(canvas, 114, 184, "AVERAGE", Color::FAINT);
        util::label(canvas, 214, 184, "MAX", Color::FAINT);
        draw_text(canvas, 14, 198, &util::temp_c(t.cpu.temp_c.or(t.history.cpu_temp.last())), 3, Color::WHITE);
        let avg = t.history.cpu_temp.avg().map(|v| format!("{:.0}C", v)).unwrap_or_else(|| "--".into());
        draw_text(canvas, 114, 198, &avg, 3, Color::CPU);
        let max = t.history.cpu_temp.max().map(|v| format!("{:.0}C", v)).unwrap_or_else(|| "--".into());
        draw_text(canvas, 214, 198, &max, 3, Color::WARN);
    }
}
