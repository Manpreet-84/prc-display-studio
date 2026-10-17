//! 320x240 RGB framebuffer + primitives. No USB knowledge here.

use super::colors::Color;

pub struct Canvas {
    w: u32,
    h: u32,
    buf: Vec<u8>,
}

impl Canvas {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            w,
            h,
            buf: vec![0; (w * h * 3) as usize],
        }
    }

    pub fn width(&self) -> u32 {
        self.w
    }

    pub fn height(&self) -> u32 {
        self.h
    }

    pub fn fill(&mut self, c: Color) {
        for px in self.buf.chunks_exact_mut(3) {
            px[0] = c.0;
            px[1] = c.1;
            px[2] = c.2;
        }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, c: Color) {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return;
        }
        let i = ((y as u32 * self.w + x as u32) * 3) as usize;
        self.buf[i] = c.0;
        self.buf[i + 1] = c.1;
        self.buf[i + 2] = c.2;
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Color) {
        for yy in y..(y + h) {
            for xx in x..(x + w) {
                self.set_pixel(xx, yy, c);
            }
        }
    }

    pub fn draw_rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Color) {
        self.draw_hline(x, y, w, c);
        self.draw_hline(x, y + h - 1, w, c);
        self.draw_vline(x, y, h, c);
        self.draw_vline(x + w - 1, y, h, c);
    }

    pub fn draw_hline(&mut self, x: i32, y: i32, w: i32, c: Color) {
        for xx in x..(x + w) {
            self.set_pixel(xx, y, c);
        }
    }

    pub fn draw_vline(&mut self, x: i32, y: i32, h: i32, c: Color) {
        for yy in y..(y + h) {
            self.set_pixel(x, yy, c);
        }
    }

    pub fn draw_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, c: Color) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.set_pixel(x0, y0, c);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    #[allow(clippy::too_many_arguments)] // primitive draw op; keeping canvas API flat
    pub fn progress_bar(&mut self, x: i32, y: i32, w: i32, h: i32, frac: f32, fg: Color, bg: Color) {
        let f = frac.clamp(0.0, 1.0);
        self.fill_rect(x, y, w, h, bg);
        let fw = ((w as f32 * f) as i32).clamp(0, w);
        if fw > 0 {
            self.fill_rect(x, y, fw, h, fg);
        }
        self.draw_rect(x, y, w, h, Color::FAINT);
    }

    pub fn fill_circle(&mut self, cx: i32, cy: i32, r: i32, c: Color) {
        let r2 = r * r;
        for yy in (cy - r)..=(cy + r) {
            for xx in (cx - r)..=(cx + r) {
                let dx = xx - cx;
                let dy = yy - cy;
                if dx * dx + dy * dy <= r2 {
                    self.set_pixel(xx, yy, c);
                }
            }
        }
    }

    pub fn draw_circle(&mut self, cx: i32, cy: i32, r: i32, c: Color) {
        // Midpoint circle.
        let mut x = r;
        let mut y = 0;
        let mut err = 1 - x;
        while x >= y {
            for (px, py) in [
                (cx + x, cy + y),
                (cx + y, cy + x),
                (cx - y, cy + x),
                (cx - x, cy + y),
                (cx - x, cy - y),
                (cx - y, cy - x),
                (cx + y, cy - x),
                (cx + x, cy - y),
            ] {
                self.set_pixel(px, py, c);
            }
            y += 1;
            if err < 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err += 2 * (y - x) + 1;
            }
        }
    }

    /// Filled ring segment from `start_rad` to `end_rad` (radians, 0 = east,
    /// clockwise positive as screen-y grows). Used for gauges.
    #[allow(clippy::too_many_arguments)] // primitive draw op; keeping canvas API flat
    pub fn fill_ring_arc(
        &mut self,
        cx: i32,
        cy: i32,
        r_outer: i32,
        r_inner: i32,
        start_rad: f32,
        end_rad: f32,
        c: Color,
    ) {
        if end_rad <= start_rad {
            return;
        }
        // One step per ~0.5 degree, scaled by radius for uniform coverage.
        let steps = (((end_rad - start_rad).to_degrees() * 2.0) as i32).max(1);
        for i in 0..=steps {
            let a = start_rad + (end_rad - start_rad) * i as f32 / steps as f32;
            let (sin, cos) = a.sin_cos();
            let mut r = r_inner;
            while r <= r_outer {
                let px = cx + (r as f32 * cos).round() as i32;
                let py = cy + (r as f32 * sin).round() as i32;
                self.set_pixel(px, py, c);
                r += 1;
            }
        }
    }

    pub fn fill_rounded_rect(&mut self, x: i32, y: i32, w: i32, h: i32, r: i32, c: Color) {
        let r = r.min(w / 2).min(h / 2).max(0);
        if r == 0 {
            self.fill_rect(x, y, w, h, c);
            return;
        }
        self.fill_rect(x + r, y, w - 2 * r, h, c);
        self.fill_rect(x, y + r, r, h - 2 * r, c);
        self.fill_rect(x + w - r, y + r, r, h - 2 * r, c);
        for (ccx, ccy, x0, y0) in [
            (x + r, y + r, x, y),
            (x + w - r - 1, y + r, x + w - r, y),
            (x + r, y + h - r - 1, x, y + h - r),
            (x + w - r - 1, y + h - r - 1, x + w - r, y + h - r),
        ] {
            for yy in y0..y0 + r {
                for xx in x0..x0 + r {
                    let dx = xx - ccx;
                    let dy = yy - ccy;
                    if dx * dx + dy * dy <= r * r {
                        self.set_pixel(xx, yy, c);
                    }
                }
            }
        }
    }

    /// Composite raw RGB888 data (w2*h2) at (x, y), clipping at edges.
    pub fn blit_rgb(&mut self, x: i32, y: i32, w2: u32, h2: u32, data: &[u8]) {
        if data.len() < (w2 * h2 * 3) as usize {
            return;
        }
        for yy in 0..h2 as i32 {
            for xx in 0..w2 as i32 {
                let sx = x + xx;
                let sy = y + yy;
                if sx < 0 || sy < 0 || sx >= self.w as i32 || sy >= self.h as i32 {
                    continue;
                }
                let si = ((yy as u32 * w2 + xx as u32) * 3) as usize;
                let di = ((sy as u32 * self.w + sx as u32) * 3) as usize;
                self.buf[di] = data[si];
                self.buf[di + 1] = data[si + 1];
                self.buf[di + 2] = data[si + 2];
            }
        }
    }

    pub fn as_rgb_bytes(&self) -> &[u8] {
        &self.buf
    }

    pub fn save_png(&self, path: &str) -> Result<(), String> {
        let img: image::RgbImage =
            image::RgbImage::from_raw(self.w, self.h, self.buf.clone())
                .ok_or_else(|| "bad framebuffer size".to_string())?;
        img.save(path).map_err(|e| e.to_string())
    }
}
