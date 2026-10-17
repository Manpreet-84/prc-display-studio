//! PAGE 06 — network: throughput, graphs, local info. No public IP.

use crate::renderer::canvas::Canvas;
use crate::renderer::colors::Color;
use crate::renderer::graph::sparkline;
use crate::renderer::text::draw_text;
use crate::telemetry::TelemetrySnapshot;

use super::util;
use super::Theme;

pub struct Network;

impl Theme for Network {
    fn id(&self) -> &'static str {
        "network"
    }
    fn name(&self) -> &'static str {
        "Network"
    }

    fn render(&mut self, canvas: &mut Canvas, t: &TelemetrySnapshot) {
        canvas.fill(Color::BLACK);
        util::header(canvas, "NET");

        let down = t.history.net_down.values();
        let up = t.history.net_up.values();

        // Down.
        util::label(canvas, 10, 36, "DOWNLOAD MBPS", Color::FAINT);
        draw_text(canvas, 10, 50, &util::mbps(t.network.down_mbps), 4, Color::WHITE);
        sparkline(canvas, 10, 84, 300, 48, &down, Color::GPU, Some(Color::GPU_DIM));

        // Up.
        util::label(canvas, 10, 140, "UPLOAD MBPS", Color::FAINT);
        draw_text(canvas, 10, 154, &util::mbps(t.network.up_mbps), 3, Color::RAM);
        sparkline(canvas, 110, 150, 200, 40, &up, Color::RAM, None);

        // Footer: IP / adapter / type. Local only — no public IP by default.
        canvas.draw_hline(8, 196, 304, Color::FAINT);
        let ip = t.network.local_ip.clone().unwrap_or_else(|| "--".into());
        let adapter = t.network.adapter.clone().unwrap_or_else(|| "--".into());
        draw_text(canvas, 10, 204, &format!("IP {}", ip), 1, Color::DIM);
        draw_text(canvas, 10, 216, &format!("ADAPTER {}", adapter), 1, Color::DIM);
        let conn = t.network.conn_type.clone().unwrap_or_else(|| "--".into());
        draw_text(canvas, 190, 204, &format!("TYPE {}", conn), 1, Color::DIM);
        draw_text(canvas, 190, 216, "PUBLIC --", 1, Color::FAINT);
    }
}
