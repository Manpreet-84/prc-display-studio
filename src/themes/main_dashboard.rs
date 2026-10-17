use crate::{renderer::{canvas::Canvas, colors::Color, graph::sparkline, text::{draw_text, text_width}}, telemetry::TelemetrySnapshot};
use super::{util, Theme};
pub struct MainDashboard;
impl Theme for MainDashboard {
    fn id(&self) -> &'static str { "main" }
    fn name(&self) -> &'static str { "Main Dashboard" }
    fn render(&mut self, c: &mut Canvas, t: &TelemetrySnapshot) {
        c.fill(Color::BLACK);
        util::header(c, &util::hm(&t.system.time_hm));
        for (x, label, temp, usage, freq, data, color, dim) in [
            (10,"CPU",t.cpu.temp_c,t.cpu.usage_pct,t.cpu.freq_mhz,t.history.cpu_usage.values(),Color::CPU,Color::CPU_DIM),
            (164,"GPU",t.gpu.temp_c,t.gpu.usage_pct,t.gpu.core_mhz,t.history.gpu_usage.values(),Color::GPU,Color::GPU_DIM)] {
            util::tech_panel(c,x,34,146,142,color);
            draw_text(c,x+9,43,label,2,color);
            let value=util::temp_c(temp);
            draw_text(c,x+(146-text_width(&value,4))/2,69,&value,4,Color::WHITE);
            sparkline(c,x+9,110,67,48,&data,color,Some(dim));
            draw_text(c,x+84,116,&util::pct(usage),2,Color::WHITE);
            draw_text(c,x+84,139,&util::ghz(freq),1,Color::DIM);
            draw_text(c,x+9,164,"THERMAL / LOAD",1,Color::DIM);
        }
        let vram=match(t.gpu.vram_used_mb,t.gpu.vram_total_mb){(Some(u),Some(v)) if v>0=>Some(u as f32/v as f32*100.0),_=>None};
        for (x,label,value,pct,color) in [(10,"RAM",util::gb_pair(t.memory.used_mb,t.memory.total_mb),t.memory.pct(),Color::WARN),(164,"VRAM",util::vram_pair(t.gpu.vram_used_mb,t.gpu.vram_total_mb),vram,Color::GPU)] {
            util::tech_panel(c,x,183,146,43,color);
            draw_text(c,x+8,190,label,1,color);
            draw_text(c,x+49,190,&value,1,Color::DIM);
            draw_text(c,x+8,204,&util::pct(pct),2,Color::WHITE);
            c.progress_bar(x+58,207,78,7,pct.unwrap_or(0.0)/100.0,color,Color::BAR_BG);
        }
        draw_text(c,12,231,"DEMO / SYNTHETIC TELEMETRY",1,Color::DIM);
    }
}
