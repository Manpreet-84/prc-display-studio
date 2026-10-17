//! Shared formatting helpers for themes. Never queries sensors — only
//! formats `Option<T>` from the snapshot; `None` becomes "--".

use crate::renderer::canvas::Canvas;
use crate::renderer::colors::Color;
use crate::renderer::text::draw_text;

pub fn temp_c(v: Option<f32>) -> String {
    v.map(|t| format!("{:.0}C", t)).unwrap_or_else(|| "--".into())
}

pub fn pct(v: Option<f32>) -> String {
    v.map(|p| format!("{:.0}%", p)).unwrap_or_else(|| "--".into())
}

pub fn ghz(v: Option<f32>) -> String {
    v.map(|f| {
        if f >= 1000.0 {
            format!("{:.2} GHZ", f / 1000.0)
        } else {
            format!("{:.0} MHZ", f)
        }
    })
    .unwrap_or_else(|| "--".into())
}

pub fn watts(v: Option<f32>) -> String {
    v.map(|w| format!("{:.0}W", w)).unwrap_or_else(|| "--".into())
}

pub fn rpm(v: Option<u32>) -> String {
    v.map(|r| format!("{}", r)).unwrap_or_else(|| "--".into())
}

pub fn gb_pair(used_mb: Option<u64>, total_mb: Option<u64>) -> String {
    match (used_mb, total_mb) {
        (Some(u), Some(t)) => format!("{:.1}/{:.0}G", u as f32 / 1024.0, t as f32 / 1024.0),
        _ => "--".into(),
    }
}

pub fn vram_pair(used_mb: Option<u32>, total_mb: Option<u32>) -> String {
    match (used_mb, total_mb) {
        (Some(u), Some(t)) => format!("{:.1}/{:.0}G", u as f32 / 1024.0, t as f32 / 1024.0),
        _ => "--".into(),
    }
}

pub fn mbps(v: Option<f32>) -> String {
    v.map(|m| format!("{:.0}", m)).unwrap_or_else(|| "--".into())
}

pub fn hm(v: &Option<String>) -> String {
    v.clone().unwrap_or_else(|| "--:--".into())
}

/// Standard page header: brand left, page tag right, divider.
pub fn header(canvas: &mut Canvas, tag: &str) {
    draw_text(canvas, 8, 6, "PRC", 2, Color::WHITE);
    let tx = 312 - (tag.chars().count() as i32 * 6 - 1);
    draw_text(canvas, tx, 8, tag, 1, Color::DIM);
    canvas.draw_hline(8, 26, 304, Color::FAINT);
    canvas.draw_hline(8, 26, 38, Color::CPU);
    canvas.draw_line(8, 26, 4, 30, Color::CPU);
}

/// Small dim field label.
pub fn label(canvas: &mut Canvas, x: i32, y: i32, text: &str, color: Color) {
    draw_text(canvas, x, y, text, 1, color);
}

/// Chamfered panel with a restrained accent and layered corner brackets.
pub fn tech_panel(c: &mut Canvas, x: i32, y: i32, w: i32, h: i32, accent: Color) {
    c.fill_rect(x+1,y+1,w-2,h-2,Color::PANEL);
    let edge=Color(47,59,68);
    c.draw_hline(x+6,y,w-12,edge);
    c.draw_hline(x+6,y+h-1,w-12,edge);
    c.draw_vline(x,y+6,h-12,edge);
    c.draw_vline(x+w-1,y+6,h-12,edge);
    for (a,b,d,e) in [(x,y+6,x+6,y),(x+w-7,y,x+w-1,y+6),(x,y+h-7,x+6,y+h-1),(x+w-7,y+h-1,x+w-1,y+h-7)] { c.draw_line(a,b,d,e,edge); }
    c.draw_line(x,y+6,x+6,y,accent);
    c.draw_hline(x+6,y,30,accent);
    c.draw_vline(x,y+6,16,accent);
    c.draw_hline(x+w-24,y+h-1,18,accent);
}
