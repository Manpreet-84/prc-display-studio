//! Scaled bitmap text drawing.

use super::canvas::Canvas;
use super::colors::Color;
use super::font::glyph;

pub const GLYPH_W: i32 = 5;
pub const GLYPH_H: i32 = 7;
pub const GLYPH_ADV: i32 = 6;

pub fn text_width(text: &str, scale: i32) -> i32 {
    if text.is_empty() {
        return 0;
    }
    (text.chars().count() as i32 * GLYPH_ADV - 1) * scale
}

pub fn draw_text(canvas: &mut Canvas, x: i32, y: i32, text: &str, scale: i32, color: Color) {
    let s = scale.max(1);
    let mut cx = x;
    for ch in text.chars() {
        let rows = glyph(ch);
        for (ry, row) in rows.iter().enumerate() {
            for bx in 0..5 {
                if (row >> (4 - bx)) & 1 == 1 {
                    canvas.fill_rect(cx + bx * s, y + ry as i32 * s, s, s, color);
                }
            }
        }
        cx += GLYPH_ADV * s;
    }
}
