//! PAGE 14 — cyberpunk: dark, neon accents, high readability.

use crate::renderer::canvas::Canvas;
use crate::renderer::colors::Color;
use crate::renderer::text::draw_text;
use crate::telemetry::TelemetrySnapshot;

use super::util;
use super::Theme;

pub struct Cyberpunk;

const NEON_PINK: Color = Color(255, 47, 194);
const NEON_CYAN: Color = Color(34, 240, 255);
const NEON_YELLOW: Color = Color(255, 230, 60);

impl Theme for Cyberpunk {
    fn id(&self) -> &'static str {
        "cyberpunk"
    }
    fn name(&self) -> &'static str {
        "Cyberpunk"
    }

    fn render(&mut self, canvas: &mut Canvas, t: &TelemetrySnapshot) {
        canvas.fill(Color(4, 4, 8));

        // Top bar.
        canvas.fill_rect(0, 0, 320, 24, Color(12, 10, 24));
        draw_text(canvas, 8, 6, "PRC//NET", 2, NEON_CYAN);
        draw_text(canvas, 232, 8, &util::hm(&t.system.time_hm), 2, NEON_YELLOW);
        canvas.draw_hline(0, 24, 320, NEON_PINK);

        // Corner accents.
        canvas.draw_rect(4, 30, 312, 122, Color(40, 20, 60));
        canvas.draw_vline(160, 30, 122, Color(40, 20, 60));

        // CPU block.
        util::label(canvas, 14, 36, "CPU TEMP", NEON_PINK);
        draw_text(canvas, 14, 52, &util::temp_c(t.cpu.temp_c), 4, Color::WHITE);
        draw_text(canvas, 14, 92, &util::pct(t.cpu.usage_pct), 2, NEON_PINK);
        canvas.progress_bar(14, 114, 136, 8, t.cpu.usage_pct.unwrap_or(0.0) / 100.0, NEON_PINK, Color::BAR_BG);
        draw_text(canvas, 14, 130, &util::ghz(t.cpu.freq_mhz), 1, Color::DIM);

        // GPU block.
        util::label(canvas, 170, 36, "GPU TEMP", NEON_CYAN);
        draw_text(canvas, 170, 52, &util::temp_c(t.gpu.temp_c), 4, Color::WHITE);
        draw_text(canvas, 170, 92, &util::pct(t.gpu.usage_pct), 2, NEON_CYAN);
        canvas.progress_bar(170, 114, 136, 8, t.gpu.usage_pct.unwrap_or(0.0) / 100.0, NEON_CYAN, Color::BAR_BG);
        draw_text(canvas, 170, 130, &util::vram_pair(t.gpu.vram_used_mb, t.gpu.vram_total_mb), 1, Color::DIM);

        // Bottom: RAM + date strip.
        canvas.fill_rect(4, 158, 312, 78, Color(10, 8, 18));
        util::label(canvas, 14, 166, "MEMORY", NEON_YELLOW);
        draw_text(canvas, 14, 180, &util::pct(t.memory.pct()), 3, Color::WHITE);
        canvas.progress_bar(14, 206, 200, 10, t.memory.pct().unwrap_or(0.0) / 100.0, NEON_YELLOW, Color::BAR_BG);
        draw_text(canvas, 14, 222, &util::gb_pair(t.memory.used_mb, t.memory.total_mb), 1, Color::DIM);
        draw_text(canvas, 230, 166, &t.system.date_ymd.clone().unwrap_or_default(), 1, Color::DIM);
        draw_text(canvas, 230, 180, "GRID", 1, NEON_CYAN);
        draw_text(canvas, 230, 194, "ONLINE", 1, NEON_PINK);

        // Scanline accents (subtle).
        for y in (26..236).step_by(6) {
            canvas.draw_hline(4, y, 312, Color(14, 12, 22));
        }
        canvas.draw_rect(4, 30, 312, 206, NEON_PINK);
    }
}
