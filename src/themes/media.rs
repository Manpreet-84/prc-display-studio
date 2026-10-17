//! PAGE 05 — media: artwork, title/artist, progress (display-only).

use crate::renderer::canvas::Canvas;
use crate::renderer::colors::Color;
use crate::renderer::text::draw_text;
use crate::telemetry::TelemetrySnapshot;

use super::util;
use super::Theme;

pub struct Media;

fn mmss(s: u32) -> String {
    format!("{}:{:02}", s / 60, s % 60)
}

impl Theme for Media {
    fn id(&self) -> &'static str {
        "media"
    }
    fn name(&self) -> &'static str {
        "Media"
    }

    fn render(&mut self, canvas: &mut Canvas, t: &TelemetrySnapshot) {
        canvas.fill(Color::BLACK);
        util::header(canvas, "MEDIA");

        let Some(m) = t.media.as_ref() else {
            draw_text(canvas, 60, 110, "NOTHING PLAYING", 2, Color::DIM);
            return;
        };

        // Artwork placeholder: deterministic gradient block (user image later).
        let ax = 10;
        let ay = 40;
        for yy in 0..96i32 {
            for xx in 0..96i32 {
                let r = (40 + xx / 2) as u8;
                let g = (30 + yy / 2) as u8;
                let b = (90 + (xx + yy) / 3) as u8;
                canvas.set_pixel(ax + xx, ay + yy, Color(r, g, b));
            }
        }
        canvas.draw_rect(ax, ay, 96, 96, Color::FAINT);
        // Simple note glyph over artwork.
        draw_text(canvas, ax + 34, ay + 34, "MUSIC", 2, Color::WHITE);

        // Title / artist.
        let title = if m.title.chars().count() > 16 { m.title.chars().take(16).collect::<String>() } else { m.title.clone() };
        let artist = if m.artist.chars().count() > 16 { m.artist.chars().take(16).collect::<String>() } else { m.artist.clone() };
        draw_text(canvas, 116, 44, &title, 2, Color::WHITE);
        draw_text(canvas, 116, 64, &artist, 1, Color::DIM);
        let state = if m.playing { "PLAYING" } else { "PAUSED" };
        draw_text(canvas, 116, 84, state, 1, Color::GPU);

        // Progress.
        let frac = if m.duration_s > 0 { m.elapsed_s as f32 / m.duration_s as f32 } else { 0.0 };
        canvas.progress_bar(10, 156, 300, 12, frac, Color::WHITE, Color::BAR_BG);
        draw_text(canvas, 10, 174, &mmss(m.elapsed_s), 1, Color::DIM);
        let dur = mmss(m.duration_s);
        let dx = 310 - (dur.chars().count() as i32 * 6 - 1);
        draw_text(canvas, dx, 174, &dur, 1, Color::DIM);

        // Bottom telemetry strip (a few values as per spec).
        canvas.fill_rect(10, 196, 300, 32, Color::PANEL);
        util::label(canvas, 16, 202, "CPU", Color::CPU);
        draw_text(canvas, 40, 202, &util::temp_c(t.cpu.temp_c), 1, Color::WHITE);
        util::label(canvas, 100, 202, "GPU", Color::GPU);
        draw_text(canvas, 124, 202, &util::temp_c(t.gpu.temp_c), 1, Color::WHITE);
        util::label(canvas, 184, 202, "RAM", Color::RAM);
        draw_text(canvas, 212, 202, &util::pct(t.memory.pct()), 1, Color::WHITE);
        util::label(canvas, 16, 216, "NOW", Color::FAINT);
        draw_text(canvas, 40, 216, &util::hm(&t.system.time_hm), 1, Color::DIM);
    }
}
