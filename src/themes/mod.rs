//! 16 supplied sheet faces plus the original clock.

use crate::renderer::canvas::Canvas;
use crate::telemetry::TelemetrySnapshot;

pub mod alert;
pub mod art;
pub mod central_gauge;
pub mod clock;
pub mod cooling;
pub mod cyberpunk;
pub mod main_dashboard;
pub mod media;
pub mod minimal;
pub mod network;
pub mod servers;
pub mod split_detail;
pub mod storage;
pub mod system_overview;
pub mod temperature;
pub mod terminal;
pub mod util;
pub mod faces;
pub mod distance;
pub mod sheet;
mod external;
pub mod custom;

pub use alert::Alert;
pub use art::Art;
pub use central_gauge::CentralGauge;
pub use clock::Clock;
pub use cooling::Cooling;
pub use cyberpunk::Cyberpunk;
pub use main_dashboard::MainDashboard;
pub use media::Media;
pub use minimal::Minimal;
pub use network::Network;
pub use servers::Servers;
pub use split_detail::SplitDetail;
pub use storage::Storage;
pub use system_overview::SystemOverview;
pub use temperature::Temperature;
pub use terminal::Terminal;

/// Theme contract: sensors never queried here â€” only `TelemetrySnapshot`.
pub trait Theme {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn render(&mut self, canvas: &mut Canvas, telemetry: &TelemetrySnapshot);
}

/// Sheet reading order, then the original clock: 17 faces.
pub fn all_themes() -> Vec<Box<dyn Theme>> {
    let mut themes: Vec<Box<dyn Theme>> = vec![Box::new(sheet::SheetFace::new("custom", "Custom HUD", include_bytes!("../../assets/neutral.png"), 0))];
    for (id,name) in [("main","Main Dashboard"),("central","Central Gauge"),
        ("performance","Performance"),("system","System Overview"),("media","Media"),
        ("network","Network"),("servers","Home Lab"),("temperature","Temperature History"),
        ("cooling","Fans & Pump"),("storage","Storage"),("terminal","Terminal"),
        ("art","Art"),("cyberpunk","Cyberpunk"),("minimal","Minimal"),("alert","Temperature Alert")] {
        themes.push(Box::new(distance::DistanceFace(id,name)));
    }
    external::append(&mut themes);
    themes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faces_render_with_demo_and_missing_sensors() {
        let mut ids = std::collections::HashSet::new();
        for mut theme in all_themes() {
            assert!(ids.insert(theme.id().to_owned()), "duplicate face id");
            for telemetry in [crate::telemetry::fake_telemetry(), TelemetrySnapshot::default()] {
                let mut canvas = Canvas::new(320, 240);
                theme.render(&mut canvas, &telemetry);
                assert_eq!(canvas.as_rgb_bytes().len(), 320 * 240 * 3);
                assert!(canvas.as_rgb_bytes().chunks_exact(3).any(|p| p != &canvas.as_rgb_bytes()[..3]));
            }
        }
    }
}
