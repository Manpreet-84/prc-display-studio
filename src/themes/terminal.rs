//! PAGE 12 — terminal / ASCII: monospaced technical readout.

use crate::renderer::canvas::Canvas;
use crate::renderer::colors::Color;
use crate::renderer::text::draw_text;
use crate::telemetry::TelemetrySnapshot;

use super::util;
use super::Theme;

pub struct Terminal;

fn field(canvas: &mut Canvas, y: i32, key: &str, val: &str, color: Color) {
    draw_text(canvas, 10, y, key, 1, Color::GPU);
    draw_text(canvas, 66, y, val, 1, color);
}

impl Theme for Terminal {
    fn id(&self) -> &'static str {
        "terminal"
    }
    fn name(&self) -> &'static str {
        "Terminal"
    }

    fn render(&mut self, canvas: &mut Canvas, t: &TelemetrySnapshot) {
        canvas.fill(Color::BLACK);
        draw_text(canvas, 10, 6, "PRC://SYSTEM", 2, Color::GPU);
        canvas.draw_hline(10, 24, 300, Color::GPU_DIM);

        let ram = util::gb_pair(t.memory.used_mb, t.memory.total_mb);
        let vram = util::vram_pair(t.gpu.vram_used_mb, t.gpu.vram_total_mb);
        let ssd = t.storage.first().and_then(|s| s.pct()).map(|p| format!("{:.0}%", p)).unwrap_or_else(|| "--".into());
        let net = match (t.network.down_mbps, t.network.up_mbps) {
            (Some(d), Some(u)) => format!("DN {:.0} UP {:.0} MBPS", d, u),
            _ => "--".to_string(),
        };

        field(canvas, 36, "OS", t.system.os.as_deref().unwrap_or("--"), Color::WHITE);
        field(canvas, 50, "HOST", t.system.hostname.as_deref().unwrap_or("--"), Color::WHITE);
        field(canvas, 64, "CPU", &format!("{}  {}", util::temp_c(t.cpu.temp_c), util::pct(t.cpu.usage_pct)), Color::WHITE);
        field(canvas, 78, "GPU", &format!("{}  {}", util::temp_c(t.gpu.temp_c), util::pct(t.gpu.usage_pct)), Color::WHITE);
        field(canvas, 92, "RAM", &ram, Color::WHITE);
        field(canvas, 106, "VRAM", &vram, Color::WHITE);
        field(canvas, 120, "SSD", &ssd, Color::WHITE);
        field(canvas, 134, "NET", &net, Color::WHITE);
        field(canvas, 148, "FANS", &format!("C{} G{} P{}", util::rpm(t.cooling.cpu_fan_rpm), util::rpm(t.cooling.gpu_fan_rpm), util::rpm(t.cooling.pump_rpm)), Color::WHITE);
        let up = t.system.uptime_secs.map(|s| format!("{}D {}H", s / 86400, (s % 86400) / 3600)).unwrap_or_else(|| "--".into());
        field(canvas, 162, "UPTIME", &up, Color::WHITE);

        canvas.draw_hline(10, 184, 300, Color::GPU_DIM);
        draw_text(canvas, 10, 196, "> SYSTEM NOMINAL", 2, Color::GPU);
        // Cursor block right after the scale-2 prompt text.
        canvas.fill_rect(10 + 16 * 6 * 2 + 4, 196, 10, 14, Color::GPU);
        draw_text(canvas, 10, 222, "ALL SENSORS REPORTING", 1, Color::FAINT);
    }
}
