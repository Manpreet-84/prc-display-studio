//! PAGE 16 — alert: overrides almost everything on critical events.

use crate::renderer::canvas::Canvas;
use crate::renderer::colors::Color;
use crate::renderer::text::draw_text;
use crate::telemetry::TelemetrySnapshot;

use super::util;
use super::Theme;

pub struct Alert;

const CRITICAL: Color = Color(255, 40, 40);

impl Theme for Alert {
    fn id(&self) -> &'static str {
        "alert"
    }
    fn name(&self) -> &'static str {
        "Alert"
    }

    fn render(&mut self, canvas: &mut Canvas, t: &TelemetrySnapshot) {
        canvas.fill(Color(16, 0, 0));

        // Hazard stripes top/bottom.
        for i in 0..40 {
            let c = if i % 2 == 0 { CRITICAL } else { Color(30, 4, 4) };
            canvas.fill_rect(i * 8, 0, 8, 8, c);
            canvas.fill_rect(i * 8, 232, 8, 8, c);
        }

        // Determine the worse of CPU/GPU (collector sets thresholds later;
        // for M2 we simply show whichever is hotter).
        let cpu_t = t.cpu.temp_c.unwrap_or(0.0);
        let gpu_t = t.gpu.temp_c.unwrap_or(0.0);
        let (what, temp, usage) = if gpu_t >= cpu_t {
            ("GPU TEMPERATURE", gpu_t, t.gpu.usage_pct)
        } else {
            ("CPU TEMPERATURE", cpu_t, t.cpu.usage_pct)
        };

        draw_text(canvas, 40, 30, "CRITICAL", 2, CRITICAL);
        draw_text(canvas, 40, 56, what, 2, Color::WHITE);
        let big = format!("{:.0}C", temp);
        draw_text(canvas, 40, 86, &big, 6, CRITICAL);
        draw_text(canvas, 40, 146, &format!("USAGE {}", util::pct(usage)), 3, Color::WHITE);

        canvas.draw_rect(24, 20, 272, 156, CRITICAL);

        draw_text(canvas, 40, 186, "CHECK COOLING / AIRFLOW", 1, Color::DIM);
        draw_text(canvas, 40, 204, "THRESHOLDS CONFIGURABLE", 1, Color::FAINT);
    }
}
