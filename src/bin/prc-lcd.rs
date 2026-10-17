use prc_lcd::{renderer::Canvas, telemetry, themes};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut selected = None;
    let mut live = false;
    let mut rotate = false;
    let mut demo = false;
    let mut interval = 2u64;
    let mut dwell = 10u64;
    let mut seconds = None;
    let mut sensors = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--decode-image" => {
                let source=args.next().ok_or("Image path required")?;
                let output=args.next().ok_or("Output PNG path required")?;
                let reader=image::ImageReader::open(source)?;
                reader.decode()?.save(output)?;
                return Ok(());
            }
            "--help" | "-h" => {
                println!("prc-lcd --list | prc-lcd --live <face> --loop | prc-lcd <face>");
                return Ok(());
            }
            "--list" => {
                for p in themes::all_themes() {
                    println!("{}: {}", p.id(), p.name())
                }
                return Ok(());
            }
            "--live" => live = true,
            "--loop" => rotate = true,
            "--demo" => demo = true,
            "--sensors" => sensors = true,
            "--cpu-fan" | "--pump" | "--sensor-file" => {
                let value = args
                    .next()
                    .ok_or(format!("{arg} needs a sensor id or file"))?;
                std::env::set_var(
                    match arg.as_str() {
                        "--cpu-fan" => "PRC_CPU_FAN_SENSOR",
                        "--pump" => "PRC_PUMP_SENSOR",
                        _ => "PRC_SENSOR_FILE",
                    },
                    value,
                );
            }
            "--interval" | "--dwell" | "--seconds" => {
                let n = args
                    .next()
                    .ok_or(format!("{arg} needs seconds"))?
                    .parse::<u64>()?;
                if n == 0 {
                    return Err(format!("{arg} must be greater than zero").into());
                }
                match arg.as_str() {
                    "--interval" => interval = n,
                    "--dwell" => {
                        if n < 2 {
                            return Err("--dwell minimum is 2 seconds".into());
                        }
                        dwell = n
                    }
                    _ => seconds = Some(n),
                }
            }
            _ if arg.starts_with('-') => {
                return Err(format!("Unknown option: {arg}; use --help").into())
            }
            _ => {
                if selected.replace(arg).is_some() {
                    return Err("Use one face id".into());
                }
            }
        }
    }
    if sensors {
        #[cfg(feature = "live-telemetry")]
        {
            let mut c = telemetry::Collector::new(2000);
            c.snapshot();
            std::thread::sleep(std::time::Duration::from_secs(2));
            prc_lcd::live::print_snapshot(&c.snapshot());
            telemetry::hardware::list_sensors();
            return Ok(());
        }
        #[cfg(not(feature = "live-telemetry"))]
        {
            return Err("Rebuild with live-telemetry feature".into());
        }
    }
    if live {
        #[cfg(feature = "live-telemetry")]
        {
            return prc_lcd::live::run(prc_lcd::live::Options {
                page: selected.unwrap_or_else(|| "custom".into()),
                rotate,
                demo,
                interval: std::time::Duration::from_secs(interval),
                dwell: std::time::Duration::from_secs(dwell),
                seconds,
            });
        }
        #[cfg(not(feature = "live-telemetry"))]
        {
            return Err("Rebuild with live-telemetry feature".into());
        }
    }
    if rotate || demo || seconds.is_some() || interval != 2 || dwell != 10 {
        return Err("Live controls require --live".into());
    }
    gallery(selected)
}

fn gallery(selected: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut pages = themes::all_themes();
    if let Some(id) = selected.as_deref() {
        if !pages.iter().any(|p| p.id() == id) {
            return Err(format!(
                "Unknown face {id}. Available: {}",
                pages.iter().map(|p| p.id()).collect::<Vec<_>>().join(", ")
            )
            .into());
        }
        pages.retain(|p| p.id() == id);
    }
    std::fs::create_dir_all("preview")?;
    let mut gallery = image::RgbImage::from_pixel(
        3 * 336,
        ((pages.len() + 2) / 3) as u32 * 256,
        image::Rgb([8, 17, 25]),
    );
    for (i, page) in pages.iter_mut().enumerate() {
        let snapshot = match page.id() {
            "media" => telemetry::fake_media_telemetry(),
            "alert" => telemetry::fake_alert_telemetry(),
            _ => telemetry::fake_telemetry(),
        };
        let mut c = Canvas::new(320, 240);
        page.render(&mut c, &snapshot);
        c.save_png(&format!("preview/{}.png", page.id()))?;
        let frame = image::RgbImage::from_raw(320, 240, c.as_rgb_bytes().to_vec())
            .ok_or("Invalid framebuffer")?;
        image::imageops::replace(
            &mut gallery,
            &frame,
            (i % 3) as i64 * 336 + 8,
            (i / 3) as i64 * 256 + 8,
        );
        if selected.is_some() {
            c.save_png("preview.png")?;
        }
        println!("{}: {}", page.id(), page.name());
    }
    gallery.save("preview/gallery.png")?;
    if selected.is_none() {
        gallery.save("preview.png")?;
    }
    Ok(())
}
