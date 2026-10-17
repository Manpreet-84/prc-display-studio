//! PAGE 04 — system overview: compact list with progress indicators.

use crate::renderer::canvas::Canvas;
use crate::renderer::colors::Color;
use crate::renderer::text::draw_text;
use crate::telemetry::TelemetrySnapshot;

use super::util;
use super::Theme;

pub struct SystemOverview;

fn row(canvas: &mut Canvas, y: i32, name: &str, value: &str, frac: f32, accent: Color) {
    draw_text(canvas, 10, y, name, 1, Color::DIM);
    let vx = 300 - (value.chars().count() as i32 * 6 - 1);
    draw_text(canvas, vx, y, value, 1, Color::WHITE);
    canvas.progress_bar(10, y + 12, 300, 8, frac, accent, Color::BAR_BG);
}

impl Theme for SystemOverview {
    fn id(&self) -> &'static str {
        "system"
    }
    fn name(&self) -> &'static str {
        "System Overview"
    }

    fn render(&mut self, canvas: &mut Canvas, t: &TelemetrySnapshot) {
        canvas.fill(Color::BLACK);
        util::header(canvas, "SYSTEM");

        let vram_frac = match (t.gpu.vram_used_mb, t.gpu.vram_total_mb) {
            (Some(u), Some(tot)) if tot > 0 => u as f32 / tot as f32,
            _ => 0.0,
        };
        let ssd = t.storage.first();
        let ssd_frac = ssd.and_then(|s| s.pct()).unwrap_or(0.0) / 100.0;
        let net_frac = (t.network.down_mbps.unwrap_or(0.0) / 1000.0).clamp(0.0, 1.0);

        row(canvas, 38, "CPU", &format!("{}  {}", util::temp_c(t.cpu.temp_c), util::pct(t.cpu.usage_pct)),
            t.cpu.usage_pct.unwrap_or(0.0) / 100.0, Color::CPU);
        row(canvas, 68, "GPU", &format!("{}  {}", util::temp_c(t.gpu.temp_c), util::pct(t.gpu.usage_pct)),
            t.gpu.usage_pct.unwrap_or(0.0) / 100.0, Color::GPU);
        row(canvas, 98, "RAM", &format!("{}  {}", util::pct(t.memory.pct()), util::gb_pair(t.memory.used_mb, t.memory.total_mb)),
            t.memory.pct().unwrap_or(0.0) / 100.0, Color::RAM);
        row(canvas, 128, "VRAM", &format!("{}  {}", util::pct(if t.gpu.vram_total_mb.unwrap_or(0) > 0 { Some(vram_frac * 100.0) } else { None }), util::vram_pair(t.gpu.vram_used_mb, t.gpu.vram_total_mb)),
            vram_frac, Color::GPU);
        row(canvas, 158, "SSD", &ssd.map(|s| util::pct(s.pct())).unwrap_or_else(|| "--".into()),
            ssd_frac, Color::WARN);
        row(canvas, 188, "NET", &format!("DN {}  UP {} MBPS", util::mbps(t.network.down_mbps), util::mbps(t.network.up_mbps)),
            net_frac, Color::WHITE);
    }
}
