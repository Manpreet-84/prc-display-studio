//! Palette: black background, high-contrast text, restrained neon.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub const BLACK: Color = Color(4, 8, 12);
    pub const PANEL: Color = Color(9, 14, 20);
    pub const WHITE: Color = Color(235, 235, 240);
    pub const DIM: Color = Color(130, 130, 140);
    pub const FAINT: Color = Color(70, 70, 80);
    pub const CPU: Color = Color(255, 49, 68);
    pub const CPU_DIM: Color = Color(63, 14, 23);
    pub const GPU: Color = Color(40, 244, 112);
    pub const GPU_DIM: Color = Color(11, 48, 28);
    pub const RAM: Color = Color(90, 160, 255);
    pub const WARN: Color = Color(255, 190, 60);
    pub const BAR_BG: Color = Color(38, 38, 46);
}
