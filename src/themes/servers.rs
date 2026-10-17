//! PAGE 07 — servers / home lab: generic rows, providers come later (M8+).

use crate::renderer::canvas::Canvas;
use crate::renderer::colors::Color;
use crate::renderer::text::draw_text;
use crate::telemetry::TelemetrySnapshot;

use super::util;
use super::Theme;

pub struct Servers;

/// Generic placeholder rows — no real hostnames; sources configurable later.
const ROWS: [(&str, &str, bool); 5] = [
    ("MAIN PC", "ONLINE", true),
    ("SERVER", "ONLINE", true),
    ("VPN MESH", "CONNECTED", true),
    ("LIGHTS", "ON", true),
    ("NAS DISK", "72%", false),
];

impl Theme for Servers {
    fn id(&self) -> &'static str {
        "servers"
    }
    fn name(&self) -> &'static str {
        "Servers"
    }

    fn render(&mut self, canvas: &mut Canvas, _t: &TelemetrySnapshot) {
        canvas.fill(Color::BLACK);
        util::header(canvas, "LAB");

        for (i, (name, status, is_ok)) in ROWS.iter().enumerate() {
            let y = 40 + i as i32 * 38;
            canvas.fill_rect(8, y, 304, 32, Color::PANEL);
            draw_text(canvas, 14, y + 8, name, 1, Color::DIM);
            let sx = 306 - (status.chars().count() as i32 * 6 - 1) * 2;
            let sc = if *is_ok { Color::GPU } else { Color::WARN };
            draw_text(canvas, sx, y + 8, status, 2, sc);
            // Status dot.
            canvas.fill_circle(14, y + 24, 3, sc);
            draw_text(canvas, 24, y + 21, if *is_ok { "OK" } else { "INFO" }, 1, sc);
        }
    }
}
