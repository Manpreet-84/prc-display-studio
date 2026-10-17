use crate::{renderer::{canvas::Canvas,colors::Color,text::{draw_text,text_width}},telemetry::TelemetrySnapshot};
use super::{util,Theme};
pub struct CentralGauge;
impl Theme for CentralGauge {
    fn id(&self)-> &'static str { "central" }
    fn name(&self)-> &'static str { "Central Gauge" }
    fn render(&mut self,c:&mut Canvas,t:&TelemetrySnapshot) {
        c.fill(Color::BLACK);
        util::header(c,"THERMAL CORE");
        let start=std::f32::consts::PI*0.65;
        let sweep=std::f32::consts::PI*1.7;
        c.draw_circle(145,124,83,Color(39,46,57));
        c.fill_ring_arc(145,124,77,68,start,start+sweep,Color::BAR_BG);
        let accent=if t.cpu.temp_c.unwrap_or(0.0)>=80.0 {Color::WARN} else {Color::CPU};
        c.fill_ring_arc(145,124,77,68,start,start+sweep*(t.cpu.temp_c.unwrap_or(0.0)/100.0).clamp(0.0,1.0),accent);
        c.draw_circle(145,124,61,Color::CPU_DIM);
        draw_text(c,128,83,"CPU",2,Color::DIM);
        let value=util::temp_c(t.cpu.temp_c);
        draw_text(c,145-text_width(&value,4)/2,112,&value,4,Color::WHITE);
        let load=util::pct(t.cpu.usage_pct);
        draw_text(c,145-text_width(&load,2)/2,151,&load,2,Color::WHITE);
        for (y,label,value) in [(50,"POWER",util::watts(t.cpu.power_w)),(161,"FAN",util::rpm(t.cooling.cpu_fan_rpm))] {
            draw_text(c,10,y,label,1,Color::DIM);
            draw_text(c,10,y+15,&value,2,Color::WHITE);
        }
        for (y,label,value,pct,color) in [(55,"GPU",util::temp_c(t.gpu.temp_c),t.gpu.usage_pct,Color::GPU),(139,"RAM",util::pct(t.memory.pct()),t.memory.pct(),Color::RAM)] {
            util::tech_panel(c,236,y,74,72,color);
            draw_text(c,243,y+8,label,1,color);
            draw_text(c,243,y+27,&value,2,Color::WHITE);
            c.progress_bar(243,y+53,60,7,pct.unwrap_or(0.0)/100.0,color,Color::BAR_BG);
        }
        draw_text(c,111,214,&util::ghz(t.cpu.freq_mhz),1,Color::DIM);
        draw_text(c,12,231,"DEMO / SYNTHETIC TELEMETRY",1,Color::DIM);
    }
}
