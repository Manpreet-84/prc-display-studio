//! PAGE 10 — storage: SSD temp, capacity, R/W rates, activity graph.

use crate::renderer::canvas::Canvas;
use crate::renderer::colors::Color;
use crate::renderer::graph::sparkline;
use crate::renderer::text::draw_text;
use crate::telemetry::TelemetrySnapshot;

use super::util;
use super::Theme;

pub struct Storage;

impl Theme for Storage {
    fn id(&self) -> &'static str {
        "storage"
    }
    fn name(&self) -> &'static str {
        "Storage"
    }

    fn render(&mut self, canvas: &mut Canvas, t: &TelemetrySnapshot) {
        canvas.fill(Color::BLACK);
        util::header(canvas, "STORAGE");

        let Some(s) = t.storage.first() else {
            draw_text(canvas, 90, 110, "NO DRIVE DATA", 2, Color::DIM);
            return;
        };

        // Top: temp + capacity.
        canvas.fill_rect(8, 34, 304, 64, Color::PANEL);
        let label = if s.label.is_empty() { "DRIVE".to_string() } else { s.label.clone() };
        util::label(canvas, 14, 40, &label, Color::WARN);
        draw_text(canvas, 14, 54, &util::temp_c(s.temp_c), 3, Color::WHITE);
        let cap = format!("{} / {} GB", s.used_gb.unwrap_or(0), s.total_gb.unwrap_or(0));
        draw_text(canvas, 130, 44, &cap, 1, Color::DIM);
        canvas.progress_bar(130, 60, 176, 12, s.pct().unwrap_or(0.0) / 100.0, Color::WARN, Color::BAR_BG);
        draw_text(canvas, 130, 78, &util::pct(s.pct()), 1, Color::WHITE);

        // Middle: read / write.
        canvas.fill_rect(8, 106, 148, 56, Color::PANEL);
        util::label(canvas, 14, 112, "READ MB/S", Color::FAINT);
        draw_text(canvas, 14, 126, &s.read_mbs.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "--".into()), 3, Color::GPU);
        canvas.fill_rect(164, 106, 148, 56, Color::PANEL);
        util::label(canvas, 170, 112, "WRITE MB/S", Color::FAINT);
        draw_text(canvas, 170, 126, &s.write_mbs.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "--".into()), 3, Color::CPU);

        // Bottom: activity graph.
        util::label(canvas, 10, 170, "ACTIVITY", Color::FAINT);
        sparkline(canvas, 8, 182, 304, 50, &t.history.ssd_activity.values(), Color::WARN, None);
    }
}
