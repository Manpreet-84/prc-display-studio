//! Event-driven Windows CLI runner. No UI framework, keyboard polling or PNG IO.
use crate::{
    display::{
        hid,
        thermalright::{build_frame, chunk_reports, frozen_warframe_320x240},
    },
    renderer::Canvas,
    telemetry::{Collector, TelemetrySnapshot},
    themes,
};
use std::{
    io::BufRead,
    sync::mpsc,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, PartialEq)]
enum Command {
    Next,
    Previous,
    Loop(bool),
    Page(String),
    Status,
    Quit,
}

fn command(line: &str) -> Result<Command, String> {
    let words: Vec<_> = line.split_whitespace().collect();
    match words.as_slice() {
        ["next"] => Ok(Command::Next),
        ["prev"] => Ok(Command::Previous),
        ["loop"] => Ok(Command::Loop(true)),
        ["stop"] => Ok(Command::Loop(false)),
        ["status"] => Ok(Command::Status),
        ["quit"] | ["q"] => Ok(Command::Quit),
        ["page", id] => Ok(Command::Page((*id).into())),
        _ => Err("Commands: next | prev | loop | stop | page <id> | status | quit".into()),
    }
}

pub struct Options {
    pub page: String,
    pub rotate: bool,
    pub demo: bool,
    pub interval: Duration,
    pub dwell: Duration,
    pub seconds: Option<u64>,
}

pub fn print_snapshot(t: &TelemetrySnapshot) {
    println!("CPU load={:?}% temp={:?}C clock={:?}MHz | GPU load={:?}% temp={:?}C | RAM={:?}% | down={:?}Mbps up={:?}Mbps | CPU fan={:?}RPM pump={:?}RPM",t.cpu.usage_pct,t.cpu.temp_c,t.cpu.freq_mhz,t.gpu.usage_pct,t.gpu.temp_c,t.memory.pct(),t.network.down_mbps,t.network.up_mbps,t.cooling.cpu_fan_rpm,t.cooling.pump_rpm);
}

pub fn run(options: Options) -> Result<(), Box<dyn std::error::Error>> {
    let mut pages = themes::all_themes();
    let mut index = pages
        .iter()
        .position(|p| p.id() == options.page)
        .ok_or("Unknown face; use --list")?;
    let (tx, rx) = mpsc::channel();
    // Sensor calls stay off the HID/event thread so slow WMI cannot starve keep-alive.
    let (sample_tx, sample_rx) = mpsc::sync_channel(1);
    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let _stop_worker = stop_tx;
    if !options.demo {
        let interval = options.interval;
        std::thread::spawn(move || {
            let mut collector = Collector::new(interval.as_millis() as u64);
            loop {
                let snapshot = collector.snapshot();
                match sample_tx.try_send(snapshot) {
                    Ok(()) | Err(mpsc::TrySendError::Full(_)) => {}
                    Err(mpsc::TrySendError::Disconnected(_)) => break,
                }
                match stop_rx.recv_timeout(interval) {
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    _ => break,
                }
            }
        });
    }
    let mut snapshot = if options.demo {
        crate::telemetry::fake_telemetry()
    } else {
        sample_rx
            .recv_timeout(Duration::from_secs(15))
            .map_err(|_| "Sensor initialization timed out")?
    };
    print_snapshot(&snapshot);
    let api = hidapi::HidApi::new()?;
    let device = hid::open_frozen_warframe(
        &api,
        &hid::HidIdentity {
            vid: 0x0416,
            pid: 0x5302,
            serial: None,
        },
    )?;
    hid::send_t2_handshake(device.raw());
    let control_tx = tx.clone();
    std::thread::spawn(move || {
        for line in std::io::stdin().lock().lines() {
            match line {
                Ok(line) => match command(&line) {
                    Ok(cmd) => {
                        if tx.send(cmd).is_err() {
                            break;
                        }
                    }
                    Err(e) => eprintln!("{e}"),
                },
                Err(_) => break,
            }
        }
    });
    println!(
        "Live display: {} | source={} | sample={}s | dwell={}s",
        pages[index].id(),
        if options.demo { "DEMO" } else { "REAL" },
        options.interval.as_secs_f32(),
        options.dwell.as_secs_f32()
    );
    println!(
        "Alt+PageUp next | Alt+PageDown previous | Ctrl+Alt+Home loop | Ctrl+Alt+End stop rotation"
    );
    println!("CLI: next | prev | loop | stop | page <id> | status | quit");
    let mut canvas = Canvas::new(320, 240);
    let params = frozen_warframe_320x240();
    let mut cached = Vec::new();
    let mut dirty = true;
    let mut rotate = options.rotate;
    let mut dwell = options.dwell;
    let mut playlist: Vec<String> = Vec::new();
    let mut settings_cache = String::new();
    let start = Instant::now();
    let mut last_sample = start;
    let mut stale = false;
    let mut next_sample = start;
    let mut next_send = start;
    let mut next_page = start + options.dwell;
    let mut frames = 0u64;
    let control = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("control-command.txt");
    let _ = std::fs::remove_file(&control);
    loop {
        if let Ok(json) = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("app-settings.json")) {
            if json != settings_cache {
                if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&json) {
                    dwell = Duration::from_secs(settings["dwell"].as_u64().unwrap_or(10).clamp(2,3600));
                    rotate = settings["rotate"].as_bool().unwrap_or(rotate);
                    playlist = settings["playlist"].as_array().map(|v|v.iter().filter_map(|v|v.as_str().map(str::to_owned)).collect()).unwrap_or_default();
                    if let Some(i)=settings["face"].as_str().and_then(|id|pages.iter().position(|p|p.id()==id)) {index=i;dirty=true;}
                    next_page=Instant::now()+dwell;
                }
                settings_cache=json;
            }
        }
        if let Ok(line) = std::fs::read_to_string(&control) {
            let _ = std::fs::remove_file(&control);
            if let Ok(cmd) = command(line.trim()) { let _ = control_tx.send(cmd); }
        }
        let now = Instant::now();
        if options
            .seconds
            .is_some_and(|seconds| now.duration_since(start) >= Duration::from_secs(seconds))
        {
            break;
        }
        if let Ok(sample) = sample_rx.try_recv() {
            snapshot = sample;
            last_sample = Instant::now();
            stale = false;
            dirty = true;
        }
        if !options.demo
            && !stale
            && last_sample.elapsed() > options.interval * 3 + Duration::from_secs(2)
        {
            snapshot = TelemetrySnapshot::default();
            stale = true;
            dirty = true;
            eprintln!("Sensor worker stale; showing -- until fresh readings arrive.");
        }
        if now >= next_sample {
            if options.demo {
                snapshot = match pages[index].id() {
                    "media" => crate::telemetry::fake_media_telemetry(),
                    "alert" => crate::telemetry::fake_alert_telemetry(),
                    _ => crate::telemetry::fake_telemetry(),
                };
                dirty = true;
            }
            next_sample = Instant::now() + options.interval;
        }
        if rotate && now >= next_page {
            let eligible:Vec<usize>=pages.iter().enumerate().filter(|(_,p)|playlist.is_empty()||playlist.iter().any(|id|id==p.id())).map(|(i,_)|i).collect();
            if !eligible.is_empty(){index=eligible.iter().position(|i|*i==index).map(|i|eligible[(i+1)%eligible.len()]).unwrap_or(eligible[0]);}
            next_page = now + dwell;
            dirty = true;
            println!("face: {}", pages[index].id());
        }
        if dirty {
            pages[index].render(&mut canvas, &snapshot);
            cached = chunk_reports(&params, &build_frame(&params, canvas.as_rgb_bytes())?);
            dirty = false;
        }
        if Instant::now() >= next_send {
            for report in &cached {
                let written = device.raw().write(report)?;
                if written != report.len() {
                    return Err(format!("Short HID write: {written}/{}", report.len()).into());
                }
            }
            let mut ack = [0u8; 64];
            for _ in 0..64 {
                if device.raw().read_timeout(&mut ack, 0)? == 0 {
                    break;
                }
            }
            frames += 1;
            next_send = Instant::now() + Duration::from_millis(600);
        }
        let mut deadline = next_sample.min(next_send);
        if rotate {
            deadline = deadline.min(next_page)
        }
        if let Some(seconds) = options.seconds {
            deadline = deadline.min(start + Duration::from_secs(seconds))
        }
        match rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
            Ok(Command::Quit) => break,
            Ok(Command::Status) => {
                println!("face={} loop={} frames={frames}", pages[index].id(), rotate);
                print_snapshot(&snapshot)
            }
            Ok(cmd) => {
                match cmd {
                    Command::Next => {
                        index = (index + 1) % pages.len();
                        dirty = true
                    }
                    Command::Previous => {
                        index = (index + pages.len() - 1) % pages.len();
                        dirty = true
                    }
                    Command::Loop(on) => rotate = on,
                    Command::Page(id) => match pages.iter().position(|p| p.id() == id) {
                        Some(i) => {
                            index = i;
                            dirty = true
                        }
                        None => eprintln!("Unknown face: {id}; use --list"),
                    },
                    _ => {}
                }
                next_page = Instant::now() + dwell;
                println!("face={} loop={rotate}", pages[index].id());
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    println!("Display stopped; frames={frames}. Panel returns to firmware logo.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cli_control_commands_are_unambiguous() {
        assert_eq!(command("stop").unwrap(), Command::Loop(false));
        assert_eq!(command("loop").unwrap(), Command::Loop(true));
        assert_eq!(command("page main").unwrap(), Command::Page("main".into()));
        assert!(command("page").is_err());
        assert!(command("quit extra").is_err());
    }

}
