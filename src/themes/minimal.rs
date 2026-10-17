//! PAGE 15 — minimal waves: two clean history lines + bottom stats row.

use crate::renderer::canvas::Canvas;
use crate::renderer::colors::Color;
use crate::renderer::graph::sparkline;
use crate::renderer::text::draw_text;
use crate::telemetry::TelemetrySnapshot;

use super::util;
use super::Theme;

pub struct Minimal;

impl Theme for Minimal {
    fn id(&self) -> &'static str {
        "minimal"
    }
    fn name(&self) -> &'static str {
        "Minimal Waves"
    }

    fn render(&mut self, canvas: &mut Canvas, t: &TelemetrySnapshot) {
        canvas.fill(Color::BLACK);

        // Tiny header.
        draw_text(canvas, 10, 8, "CPU", 1, Color::CPU);
        draw_text(canvas, 44, 8, &util::temp_c(t.cpu.temp_c), 1, Color::DIM);
        draw_text(canvas, 260, 8, "GPU", 1, Color::GPU);
        draw_text(canvas, 294, 8, &util::temp_c(t.gpu.temp_c), 1, Color::DIM);

        // Two stacked wave areas.
        util::label(canvas, 10, 28, "CPU", Color::CPU);
        sparkline(canvas, 10, 40, 300, 70, &t.history.cpu_usage.values(), Color::CPU, None);
        util::label(canvas, 10, 116, "GPU", Color::GPU);
        sparkline(canvas, 10, 128, 300, 70, &t.history.gpu_usage.values(), Color::GPU, None);

        // Bottom row: CPU GPU RAM VRAM NET.
        canvas.draw_hline(8, 206, 304, Color::FAINT);
        let items: [(&str, String, Color); 5] = [
            ("CPU", util::pct(t.cpu.usage_pct), Color::CPU),
            ("GPU", util::pct(t.gpu.usage_pct), Color::GPU),
            ("RAM", util::pct(t.memory.pct()), Color::RAM),
            ("VRAM", util::pct(if t.gpu.vram_total_mb.unwrap_or(0) > 0 { t.gpu.vram_used_mb.map(|u| u as f32 / t.gpu.vram_total_mb.unwrap_or(1) as f32 * 100.0) } else { None }), Color::GPU),
            ("NET", util::mbps(t.network.down_mbps), Color::WHITE),
        ];
        for (i, (name, val, accent)) in items.iter().enumerate() {
            let x = 10 + i as i32 * 62;
            draw_text(canvas, x, 214, name, 1, Color::FAINT);
            draw_text(canvas, x, 226, val, 1, *accent);
        }
    }
}
