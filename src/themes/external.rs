use super::{sheet::SheetFace, Theme};
use crate::{renderer::Canvas, telemetry::TelemetrySnapshot};
use serde::Deserialize;
use std::path::Path;

const LAYOUTS: [&str; 16] = [
    "naruto",
    "sasuke",
    "itachi",
    "kakashi",
    "kurama",
    "team7",
    "sunset",
    "shinobi",
    "rivals",
    "team-portraits",
    "sage",
    "rasengan",
    "konoha",
    "akatsuki",
    "eyes",
    "hokage",
];
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    name: String,
    #[serde(default)]
    layout: String,
    #[serde(default)]
    elements: Vec<super::custom::Element>,
    background: String,
}
struct External {
    id: String,
    name: String,
    face: Option<SheetFace>,
    custom: Option<super::custom::Custom>,
}
impl Theme for External {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn render(&mut self, c: &mut Canvas, t: &TelemetrySnapshot) {
        if let Some(face)=&mut self.face {face.render(c,t)}
        if let Some(face)=&mut self.custom {face.render(c,t)}
    }
}
fn load(dir: &Path) -> Result<External, Box<dyn std::error::Error>> {
    let id = dir
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or("Invalid folder name")?
        .to_owned();
    if id.is_empty()
        || id.len() > 64
        || !id
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
    {
        return Err("Theme folder must use lowercase letters, digits and hyphens".into());
    }
    let config = std::fs::read(dir.join("theme.json"))?;
    if config.len() > 65536 {
        return Err("Theme manifest exceeds 64 KB".into());
    }
    let config: Config = serde_json::from_slice(&config)?;
    if config.name.trim().is_empty() || config.name.len() > 128 {
        return Err("Invalid theme name".into());
    }
    let layout = LAYOUTS
        .iter()
        .position(|v| *v == config.layout)
        .unwrap_or(0);
    if config.elements.is_empty() && !LAYOUTS.contains(&config.layout.as_str()) {return Err("Unknown HUD layout".into())}
    if config.elements.len()>64 {return Err("Maximum 64 HUD elements".into())}
    for e in &config.elements {
        if !["text","bar","ring","graph","label","panel"].contains(&e.kind.as_str()) || ![e.x,e.y,e.width,e.height,e.max].iter().all(|v|v.is_finite()) || e.x<0.0 || e.y<0.0 || e.width<12.0 || e.height<12.0 || e.x+e.width>320.0 || e.y+e.height>240.0 || e.max<=0.0 || e.size<8 || e.size>72 || e.label_size<8 || e.label_size>32 || !e.stroke.is_finite() || e.stroke<1.0 || e.stroke>20.0 || e.decimals>3 || e.label.as_ref().is_some_and(|v|v.len()>64) || e.unit.as_ref().is_some_and(|v|v.len()>16) || e.text.len()>128 {return Err("Invalid canvas element".into())}
    }
    let asset = Path::new(&config.background);
    if asset.components().count() != 1
        || !matches!(
            asset.components().next(),
            Some(std::path::Component::Normal(_))
        )
    {
        return Err("Background must be a filename inside the theme folder".into());
    }
    let bytes = std::fs::read(dir.join(asset))?;
    if bytes.len() > 5 * 1024 * 1024 {
        return Err("Background exceeds 5 MB".into());
    }
    let reader = image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format()?;
    let pixels = reader
        .decode()?
        .resize_exact(320, 240, image::imageops::FilterType::Lanczos3)
        .to_rgb8()
        .into_raw();
    Ok(External {
        id,
        name: config.name,
        face: if config.elements.is_empty(){Some(SheetFace::with_pixels(layout,pixels.clone()))}else{None},
        custom: if config.elements.is_empty(){None}else{Some(super::custom::Custom{pixels,elements:config.elements,samples:Default::default()})},
    })
}
pub fn append(themes: &mut Vec<Box<dyn Theme>>) {
    let dir = std::env::var_os("PRC_THEME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("themes"));
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut dirs: Vec<_> = entries
        .filter_map(Result::ok)
        .map(|v| v.path())
        .filter(|v| v.is_dir() && v.join("theme.json").is_file())
        .collect();
    dirs.sort();
    for dir in dirs.into_iter().take(64) {
        match load(&dir) {
            Ok(theme) if !themes.iter().any(|v| v.id() == theme.id) => themes.push(Box::new(theme)),
            Ok(_) => eprintln!("Skipping duplicate theme: {}", dir.display()),
            Err(e) => eprintln!("Skipping theme {}: {e}", dir.display()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn custom_theme_loads_and_rejects_invalid_layout_and_path() {
        let dir = std::env::temp_dir().join(format!("prc-theme-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("art.webp"),
            include_bytes!("../../assets/neutral.png"),
        )
        .unwrap();
        std::fs::write(
            dir.join("theme.json"),
            r#"{"name":"Test","layout":"naruto","background":"art.webp"}"#,
        )
        .unwrap();
        let mut theme = load(&dir).unwrap();
        let mut c = Canvas::new(320, 240);
        theme.render(&mut c, &TelemetrySnapshot::default());
        assert_eq!(theme.name(), "Test");
        assert!(c.as_rgb_bytes().iter().any(|v| *v > 0));
        let missing = c.as_rgb_bytes().to_vec();
        theme.render(&mut c, &crate::telemetry::fake_telemetry());
        assert_ne!(
            missing,
            c.as_rgb_bytes(),
            "Custom themes must use current telemetry"
        );
        for manifest in [
            r#"{"name":"Test","layout":"bogus","background":"art.webp"}"#,
            r#"{"name":"Test","layout":"naruto","background":"../art.webp"}"#,
        ] {
            std::fs::write(dir.join("theme.json"), manifest).unwrap();
            assert!(load(&dir).is_err());
        }
        std::fs::remove_file(dir.join("theme.json")).unwrap();
        std::fs::remove_file(dir.join("art.webp")).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }
}
