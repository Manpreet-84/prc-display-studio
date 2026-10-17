//! Antialiased vector surfaces and cached TrueType glyphs for the physical LCD.
use super::Canvas;
use fontdue::{Font, FontSettings, Metrics};
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};
use tiny_skia::{
    Color, FillRule, GradientStop, LineCap, LinearGradient, Paint, Path, PathBuilder, Pixmap,
    Point, SpreadMode, Stroke, Transform,
};

pub type Rgb = (u8, u8, u8);
pub const WHITE: Rgb = (244, 247, 253);
pub const MUTED: Rgb = (178, 193, 215);
pub const RED: Rgb = (255, 78, 103);
pub const GREEN: Rgb = (65, 232, 159);
pub const BLUE: Rgb = (97, 172, 255);
pub const GOLD: Rgb = (255, 194, 102);
pub const PURPLE: Rgb = (203, 126, 255);

struct Fonts {
    regular: Font,
    bold: Font,
}
fn fonts() -> &'static Fonts {
    static FONTS: OnceLock<Fonts> = OnceLock::new();
    FONTS.get_or_init(|| {
        let directory = std::path::PathBuf::from(
            std::env::var_os("WINDIR").unwrap_or_else(|| "C:/Windows".into()),
        )
        .join("Fonts");
        let load = |name: &str| {
            Font::from_bytes(
                std::fs::read(directory.join(name)).expect("Windows Segoe UI font required"),
                FontSettings::default(),
            )
            .expect("Valid Segoe UI font")
        };
        Fonts {
            regular: load("segoeui.ttf"),
            bold: load("seguisb.ttf"),
        }
    })
}

type GlyphCache = HashMap<(char, u16, bool), (Metrics, Vec<u8>)>;
fn glyphs() -> &'static Mutex<GlyphCache> {
    static CACHE: OnceLock<Mutex<GlyphCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn measure(value: &str, size: f32, bold: bool) -> f32 {
    let font = if bold {
        &fonts().bold
    } else {
        &fonts().regular
    };
    let mut width = 0.0;
    let mut previous = None;
    for ch in value.chars() {
        if let Some(prev) = previous {
            width += font.horizontal_kern(prev, ch, size).unwrap_or(0.0)
        }
        width += font.metrics(ch, size).advance_width;
        previous = Some(ch);
    }
    width
}

fn color(rgb: Rgb, alpha: u8) -> Color {
    Color::from_rgba8(rgb.0, rgb.1, rgb.2, alpha)
}
fn paint(rgb: Rgb, alpha: u8) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color(color(rgb, alpha));
    paint.anti_alias = true;
    paint
}

fn rounded(x: f32, y: f32, w: f32, h: f32, r: f32) -> Path {
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    let mut p = PathBuilder::new();
    p.move_to(x + r, y);
    p.line_to(x + w - r, y);
    p.quad_to(x + w, y, x + w, y + r);
    p.line_to(x + w, y + h - r);
    p.quad_to(x + w, y + h, x + w - r, y + h);
    p.line_to(x + r, y + h);
    p.quad_to(x, y + h, x, y + h - r);
    p.line_to(x, y + r);
    p.quad_to(x, y, x + r, y);
    p.close();
    p.finish().unwrap()
}

pub struct Surface {
    pixmap: Pixmap,
}
impl Surface {
    /// Keep an editable HUD's drawing inside its resize bounds.
    pub fn region(&mut self, x:f32,y:f32,w:f32,h:f32,draw:impl FnOnce(&mut Self)) {
        let mut layer=Self{pixmap:self.pixmap.clone()};
        draw(&mut layer);
        let left=x.ceil().clamp(0.0,320.0) as usize;
        let right=(x+w).ceil().clamp(0.0,320.0) as usize;
        let top=y.ceil().clamp(0.0,240.0) as usize;
        let bottom=(y+h).ceil().clamp(0.0,240.0) as usize;
        for row in top..bottom {
            let range=(row*320+left)*4..(row*320+right)*4;
            self.pixmap.data_mut()[range.clone()].copy_from_slice(&layer.pixmap.data()[range]);
        }
    }
    pub fn from_rgb(bytes: &[u8]) -> Self {
        let mut pixmap = Pixmap::new(320, 240).unwrap();
        for (dst, src) in pixmap.data_mut().chunks_exact_mut(4).zip(bytes.chunks_exact(3)) {
            dst.copy_from_slice(&[src[0], src[1], src[2], 255]);
        }
        Self { pixmap }
    }
    pub fn new() -> Self {
        let mut surface = Self {
            pixmap: Pixmap::new(320, 240).unwrap(),
        };
        surface.pixmap.fill(color((7, 12, 21), 255));
        surface.gradient(0.0, 0.0, 320.0, 240.0, 0.0, (15, 24, 39), (5, 9, 16));
        surface
    }
    pub fn gradient(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, top: Rgb, bottom: Rgb) {
        let path = rounded(x, y, w, h, r);
        let mut p = Paint::default();
        p.shader = LinearGradient::new(
            Point::from_xy(x, y),
            Point::from_xy(x, y + h),
            vec![
                GradientStop::new(0.0, color(top, 255)),
                GradientStop::new(1.0, color(bottom, 255)),
            ],
            SpreadMode::Pad,
            Transform::identity(),
        )
        .unwrap();
        self.pixmap
            .fill_path(&path, &p, FillRule::Winding, Transform::identity(), None);
    }
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, rgb: Rgb, alpha: u8) {
        self.pixmap.fill_path(
            &rounded(x, y, w, h, r),
            &paint(rgb, alpha),
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
    pub fn card(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.rect(x, y + 2.0, w, h, 16.0, (0, 0, 0), 100);
        self.gradient(x, y, w, h, 16.0, (22, 33, 49), (12, 20, 32));
        self.pixmap.stroke_path(
            &rounded(x + 0.5, y + 0.5, w - 1.0, h - 1.0, 15.5),
            &paint((51, 67, 89), 180),
            &Stroke {
                width: 1.0,
                ..Stroke::default()
            },
            Transform::identity(),
            None,
        );
    }
    pub fn path(&mut self, path: &Path, rgb: Rgb, alpha: u8) {
        self.pixmap.fill_path(
            path,
            &paint(rgb, alpha),
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
    pub fn stroke(&mut self, path: &Path, width: f32, rgb: Rgb, alpha: u8) {
        self.pixmap.stroke_path(
            path,
            &paint(rgb, alpha),
            &Stroke {
                width,
                line_cap: LineCap::Round,
                ..Stroke::default()
            },
            Transform::identity(),
            None,
        );
    }
    pub fn line(&mut self, x: f32, y: f32, x2: f32, y2: f32, width: f32, rgb: Rgb, alpha: u8) {
        let mut p = PathBuilder::new();
        p.move_to(x, y);
        p.line_to(x2, y2);
        self.stroke(&p.finish().unwrap(), width, rgb, alpha);
    }
    pub fn circle(&mut self, x: f32, y: f32, r: f32, rgb: Rgb, alpha: u8) {
        let mut p = PathBuilder::new();
        p.push_circle(x, y, r);
        self.path(&p.finish().unwrap(), rgb, alpha);
    }
    pub fn arc(
        &mut self,
        cx: f32,
        cy: f32,
        r: f32,
        start: f32,
        sweep: f32,
        width: f32,
        rgb: Rgb,
        alpha: u8,
    ) {
        if sweep <= 0.0 {
            return;
        }
        let steps = (sweep.abs() * r).ceil().max(2.0) as usize;
        let mut p = PathBuilder::new();
        for i in 0..=steps {
            let angle = start + sweep * i as f32 / steps as f32;
            let (sin, cos) = angle.sin_cos();
            if i == 0 {
                p.move_to(cx + r * cos, cy + r * sin)
            } else {
                p.line_to(cx + r * cos, cy + r * sin)
            }
        }
        self.stroke(&p.finish().unwrap(), width, rgb, alpha);
    }
    pub fn text(&mut self, x: f32, y: f32, value: &str, size: u16, bold: bool, rgb: Rgb) {
        let font = if bold {
            &fonts().bold
        } else {
            &fonts().regular
        };
        let mut cache = glyphs().lock().unwrap();
        let mut cursor = x;
        let mut previous = None;
        let baseline = y + size as f32 * 0.76;
        for ch in value.chars() {
            if let Some(prev) = previous {
                cursor += font.horizontal_kern(prev, ch, size as f32).unwrap_or(0.0)
            }
            let (metrics, bitmap) = cache
                .entry((ch, size, bold))
                .or_insert_with(|| font.rasterize(ch, size as f32));
            let top = (baseline - (metrics.height as i32 + metrics.ymin) as f32).round() as i32;
            let left = cursor.round() as i32 + metrics.xmin;
            let data = self.pixmap.data_mut();
            for yy in 0..metrics.height {
                for xx in 0..metrics.width {
                    let x = left + xx as i32;
                    let y = top + yy as i32;
                    if !(0..320).contains(&x) || !(0..240).contains(&y) {
                        continue;
                    }
                    let a = bitmap[yy * metrics.width + xx] as u32;
                    if a == 0 {
                        continue;
                    }
                    let i = (y as usize * 320 + x as usize) * 4;
                    for (channel, component) in [rgb.0, rgb.1, rgb.2].into_iter().enumerate() {
                        data[i + channel] =
                            ((component as u32 * a + data[i + channel] as u32 * (255 - a) + 127)
                                / 255) as u8;
                    }
                    data[i + 3] = 255;
                }
            }
            cursor += metrics.advance_width;
            previous = Some(ch);
        }
    }
    pub fn center(&mut self, cx: f32, y: f32, value: &str, size: u16, bold: bool, rgb: Rgb) {
        self.text(
            cx - measure(value, size as f32, bold) / 2.0,
            y,
            value,
            size,
            bold,
            rgb,
        );
    }
    pub fn fit(
        &mut self,
        cx: f32,
        y: f32,
        value: &str,
        max_size: u16,
        max_width: f32,
        bold: bool,
        rgb: Rgb,
    ) {
        let mut size = max_size;
        while size > 18 && measure(value, size as f32, bold) > max_width {
            size -= 1
        }
        let mut value = value.to_string();
        if measure(&value, size as f32, bold) > max_width {
            while !value.is_empty() && measure(&format!("{value}…"), size as f32, bold) > max_width
            {
                value.pop();
            }
            value.push('…');
        }
        self.center(cx, y, &value, size, bold, rgb);
    }
    pub fn bar(&mut self, x: f32, y: f32, w: f32, h: f32, fraction: Option<f32>, rgb: Rgb) {
        self.rect(x, y, w, h, h / 2.0, (43, 57, 77), 255);
        if let Some(f) = fraction.filter(|v| v.is_finite()) {
            let fw = w * f.clamp(0.0, 1.0);
            if fw > 0.0 {
                self.rect(x, y, fw, h, h / 2.0, rgb, 255)
            }
        }
    }
    pub fn finish(self, canvas: &mut Canvas) {
        let rgb: Vec<u8> = self
            .pixmap
            .data()
            .chunks_exact(4)
            .flat_map(|p| [p[0], p[1], p[2]])
            .collect();
        canvas.blit_rgb(0, 0, 320, 240, &rgb);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn glyphs_and_vector_edges_have_partial_coverage() {
        let mut surface = Surface::new();
        surface.rect(0.0, 0.0, 320.0, 240.0, 0.0, (0, 0, 0), 255);
        surface.text(20.0, 20.0, "48°C", 52, true, WHITE);
        surface.arc(160.0, 120.0, 65.0, 0.0, 2.0, 8.0, GREEN, 255);
        assert!(surface
            .pixmap
            .data()
            .chunks_exact(4)
            .any(|p| p[0] > 0 && p[0] < WHITE.0 && p[0] == p[1]));
        assert!(measure("100°C", 42.0, true) < 145.0);
    }
}
