//! PAGE 03 — split performance: CPU left, GPU right with mini graphs.

use crate::renderer::canvas::Canvas;
use crate::renderer::colors::Color;
use crate::renderer::graph::sparkline;
use crate::renderer::text::draw_text;
use crate::telemetry::TelemetrySnapshot;

use super::util;
use super::Theme;

pub struct SplitDetail;

impl Theme for SplitDetail {
    fn id(&self) -> &'static str {
        "performance"
    }
    fn name(&self) -> &'static str {
        "Split Performance"
    }

    fn render(&mut self, canvas: &mut Canvas, t: &TelemetrySnapshot) {
        canvas.fill(Color::BLACK);
        util::header(canvas, "PERF");

        let cpu_hist = t.history.cpu_usage.values();
        let gpu_hist = t.history.gpu_usage.values();

        // Left: CPU.
        canvas.fill_rect(8, 32, 148, 196, Color::PANEL);
        util::label(canvas, 14, 38, "CPU", Color::CPU);
        draw_text(canvas, 14, 52, &util::temp_c(t.cpu.temp_c), 3, Color::WHITE);
        draw_text(canvas, 14, 80, &util::pct(t.cpu.usage_pct), 2, Color::CPU);
        canvas.progress_bar(14, 102, 136, 8, t.cpu.usage_pct.unwrap_or(0.0) / 100.0, Color::CPU, Color::BAR_BG);
        util::label(canvas, 14, 118, "POWER", Color::FAINT);
        draw_text(canvas, 14, 130, &util::watts(t.cpu.power_w), 1, Color::DIM);
        util::label(canvas, 84, 118, "CLOCK", Color::FAINT);
        draw_text(canvas, 84, 130, &util::ghz(t.cpu.freq_mhz), 1, Color::DIM);
        util::label(canvas, 14, 148, "USAGE HISTORY", Color::FAINT);
        sparkline(canvas, 14, 162, 136, 56, &cpu_hist, Color::CPU, Some(Color::CPU_DIM));

        // Right: GPU.
        canvas.fill_rect(164, 32, 148, 196, Color::PANEL);
        util::label(canvas, 170, 38, "GPU", Color::GPU);
        draw_text(canvas, 170, 52, &util::temp_c(t.gpu.temp_c), 3, Color::WHITE);
        draw_text(canvas, 170, 80, &util::pct(t.gpu.usage_pct), 2, Color::GPU);
        canvas.progress_bar(170, 102, 136, 8, t.gpu.usage_pct.unwrap_or(0.0) / 100.0, Color::GPU, Color::BAR_BG);
        util::label(canvas, 170, 118, "POWER", Color::FAINT);
        draw_text(canvas, 170, 130, &util::watts(t.gpu.power_w), 1, Color::DIM);
        util::label(canvas, 240, 118, "CLOCK", Color::FAINT);
        draw_text(canvas, 240, 130, &util::ghz(t.gpu.core_mhz), 1, Color::DIM);
        util::label(canvas, 170, 148, "USAGE HISTORY", Color::FAINT);
        sparkline(canvas, 170, 162, 136, 56, &gpu_hist, Color::GPU, Some(Color::GPU_DIM));
    }
}
