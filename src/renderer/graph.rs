//! Small line/fill graphs over fixed-size sample windows.

use super::canvas::Canvas;
use super::colors::Color;

/// Draw a sparkline of `data` into the box (x, y, w, h).
/// Empty/short data degrades gracefully (draws axis only).
#[allow(clippy::too_many_arguments)] // pairs with canvas primitives; keeps call sites readable
pub fn sparkline(
    canvas: &mut Canvas,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    data: &[f32],
    line: Color,
    fill: Option<Color>,
) {
    canvas.draw_rect(x, y, w, h, Color::FAINT);
    if data.len() < 2 || w < 2 || h < 2 {
        return;
    }
    let min = data.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let span = (max - min).max(1e-6);
    let n = data.len() as f32;
    let mapx = |i: usize| x + 1 + ((i as f32 / (n - 1.0)) * (w - 3) as f32).round() as i32;
    let mapy = |v: f32| y + h - 1 - (((v - min) / span) * (h - 3) as f32).round() as i32;

    if let Some(fc) = fill {
        // Area fill under the line, row-major for cheap solid blocks.
        for i in 0..data.len() - 1 {
            let x0 = mapx(i);
            let x1 = mapx(i + 1);
            let y0 = mapy(data[i]);
            let y1 = mapy(data[i + 1]);
            for xx in x0..=x1 {
                let t = if x1 > x0 { (xx - x0) as f32 / (x1 - x0) as f32 } else { 0.0 };
                let vy = (y0 as f32 + (y1 - y0) as f32 * t).round() as i32;
                canvas.draw_vline(xx, vy, (y + h - 1 - vy).max(0), fc);
            }
        }
    }
    for i in 0..data.len() - 1 {
        canvas.draw_line(mapx(i), mapy(data[i]), mapx(i + 1), mapy(data[i + 1]), line);
    }
}
