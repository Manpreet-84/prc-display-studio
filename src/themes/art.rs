//! PAGE 13 — art: user background image with minimal overlay.
//! No bundled artwork: loads `assets/background.png` if present,
//! otherwise draws a neutral gradient fallback.

use crate::renderer::canvas::Canvas;
use crate::renderer::colors::Color;
use crate::renderer::text::draw_text;
use crate::telemetry::TelemetrySnapshot;

use super::util;
use super::Theme;

pub struct Art;

const BG_PATH: &str = "assets/background.png";

impl Theme for Art {
    fn id(&self) -> &'static str {
        "art"
    }
    fn name(&self) -> &'static str {
        "Art"
    }

    fn render(&mut self, canvas: &mut Canvas, t: &TelemetrySnapshot) {
        // Background: user image (cover-fit) or gradient fallback.
        let mut loaded = false;
        if let Ok(img) = image::open(BG_PATH) {
            let rgba = img.to_rgba8();
            let resized = image::imageops::resize(
                &rgba,
                canvas.width(),
                canvas.height(),
                image::imageops::FilterType::Triangle,
            );
            let mut rgb = vec![0u8; (canvas.width() * canvas.height() * 3) as usize];
            for (i, px) in resized.pixels().enumerate() {
                rgb[i * 3] = px[0];
                rgb[i * 3 + 1] = px[1];
                rgb[i * 3 + 2] = px[2];
            }
            canvas.blit_rgb(0, 0, canvas.width(), canvas.height(), &rgb);
            loaded = true;
        }
        if !loaded {
            for y in 0..canvas.height() as i32 {
                for x in 0..canvas.width() as i32 {
                    let r = (16 + x / 8) as u8;
                    let g = (14 + y / 10) as u8;
                    let b = (30 + (x + y) / 6) as u8;
                    canvas.set_pixel(x, y, Color(r, g, b));
                }
            }
        }

        // Scrim strip for readability (solid — no alpha yet).
        canvas.fill_rect(0, 188, 320, 52, Color::PANEL);
        canvas.draw_hline(0, 188, 320, Color::FAINT);

        util::label(canvas, 12, 194, "CPU", Color::CPU);
        draw_text(canvas, 42, 194, &util::temp_c(t.cpu.temp_c), 2, Color::WHITE);
        util::label(canvas, 130, 194, "GPU", Color::GPU);
        draw_text(canvas, 160, 194, &util::temp_c(t.gpu.temp_c), 2, Color::WHITE);
        util::label(canvas, 12, 216, "RAM", Color::RAM);
        draw_text(canvas, 42, 216, &util::pct(t.memory.pct()), 2, Color::WHITE);
        draw_text(canvas, 130, 216, &util::gb_pair(t.memory.used_mb, t.memory.total_mb), 1, Color::DIM);
        let src = if loaded { "USER BG" } else { "NO BG SET" };
        draw_text(canvas, 240, 222, src, 1, Color::FAINT);
    }
}
