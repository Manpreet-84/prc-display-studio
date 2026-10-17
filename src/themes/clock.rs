//! PAGE 11 — clock / idle: large clock + date + sensor strip.

use crate::renderer::canvas::Canvas;
use crate::renderer::colors::Color;
use crate::renderer::text::draw_text;
use crate::telemetry::TelemetrySnapshot;

use super::util;
use super::Theme;

pub struct Clock;

impl Theme for Clock {
    fn id(&self) -> &'static str {
        "clock"
    }
    fn name(&self) -> &'static str {
        "Clock"
    }

    fn render(&mut self, canvas: &mut Canvas, t: &TelemetrySnapshot) {
        canvas.fill(Color::BLACK);

        let time = util::hm(&t.system.time_hm);
        let tw = (time.chars().count() as i32 * 6 - 1) * 6;
        draw_text(canvas, (320 - tw) / 2, 60, &time, 6, Color::WHITE);

        let date = t.system.date_ymd.clone().unwrap_or_else(|| "---- -- --".into());
        let dw = (date.chars().count() as i32 * 6 - 1) * 2;
        draw_text(canvas, (320 - dw) / 2, 118, &date, 2, Color::DIM);

        // Bottom sensor row.
        canvas.fill_rect(8, 200, 304, 32, Color::PANEL);
        util::label(canvas, 14, 206, "CPU", Color::CPU);
        draw_text(canvas, 44, 206, &util::temp_c(t.cpu.temp_c), 1, Color::WHITE);
        util::label(canvas, 110, 206, "GPU", Color::GPU);
        draw_text(canvas, 140, 206, &util::temp_c(t.gpu.temp_c), 1, Color::WHITE);
        util::label(canvas, 206, 206, "IDLE", Color::FAINT);
        draw_text(canvas, 240, 206, "--W", 1, Color::DIM); // system idle power: not yet collected
        util::label(canvas, 14, 220, &format!("UPTIME {}", fmt_uptime(t.system.uptime_secs)), Color::FAINT);
    }
}

fn fmt_uptime(secs: Option<u64>) -> String {
    let Some(s) = secs else { return "--".into() };
    let d = s / 86400;
    let h = (s % 86400) / 3600;
    format!("{}D {}H", d, h)
}
