//! Optional existing LibreHardwareMonitor WMI provider. No installed service.
use super::TelemetrySnapshot;
use serde::Deserialize;
use std::time::{Duration, Instant};
use wmi::WMIConnection;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Sensor {
    #[serde(default)]
    identifier: String,
    name: String,
    parent: String,
    sensor_type: String,
    value: Option<f32>,
}

pub struct HardwareProbe {
    connection: Option<WMIConnection>,
    retry_at: Instant,
}

impl Default for HardwareProbe {
    fn default() -> Self {
        Self {
            connection: None,
            retry_at: Instant::now(),
        }
    }
}

impl HardwareProbe {
    pub fn apply(&mut self, snapshot: &mut TelemetrySnapshot) {
        if let Some(rows) = bridge_sensors() {
            apply_sensors(&rows, snapshot);
            return;
        }
        if self.connection.is_none() && Instant::now() >= self.retry_at {
            self.connection = WMIConnection::with_namespace_path(r"root\LibreHardwareMonitor").ok();
            self.retry_at = Instant::now() + Duration::from_secs(30);
            if self.connection.is_some() {
                println!("LibreHardwareMonitor WMI connected");
            }
        }
        let Some(connection) = &self.connection else {
            return;
        };
        let rows:Vec<Sensor>=match connection.raw_query("SELECT Name, Parent, SensorType, Value FROM Sensor WHERE SensorType='Temperature' OR SensorType='Fan' OR SensorType='Power'") {
            Ok(rows)=>rows,
            Err(_)=>{ self.connection=None; self.retry_at=Instant::now()+Duration::from_secs(30); return }
        };
        apply_sensors(&rows, snapshot);
    }
}

fn apply_sensors(rows: &[Sensor], t: &mut TelemetrySnapshot) {
    // Prefer explicit CPU package sensors. Never substitute motherboard/ACPI zones.
    for name in [
        "CPU Package",
        "CPU (Tctl/Tdie)",
        "Core (Tctl/Tdie)",
        "CPU Die (average)",
        "Package",
        "CPU Core",
    ] {
        if let Some(v) = rows
            .iter()
            .find(|s| s.parent.contains("cpu") && s.sensor_type == "Temperature" && s.name == name)
            .and_then(|s| s.value)
            .filter(|v| v.is_finite() && (-20.0..150.0).contains(v))
        {
            t.cpu.temp_c = Some(v);
            break;
        }
    }
    for s in rows {
        let Some(v) = s.value.filter(|v| v.is_finite()) else {
            continue;
        };
        let name = s.name.to_ascii_lowercase();
        match s.sensor_type.as_str() {
            "Fan" if (0.0..=30000.0).contains(&v) => {
                if !s.parent.contains("gpu") {
                    t.cooling.motherboard_fans.push((s.name.clone(), v as u32));
                }
                if std::env::var("PRC_PUMP_SENSOR").is_ok_and(|id| id == s.identifier)
                    || name.contains("pump")
                {
                    t.cooling.pump_rpm = Some(v as u32);
                } else if std::env::var("PRC_CPU_FAN_SENSOR").is_ok_and(|id| id == s.identifier) {
                    t.cooling.cpu_fan_rpm = Some(v as u32);
                } else if s.parent.contains("gpu") {
                    t.cooling.gpu_fan_rpm = Some(v as u32);
                } else if name.contains("cpu") {
                    t.cooling.cpu_fan_rpm = Some(v as u32);
                }
            }
            "Power"
                if s.parent.contains("cpu")
                    && (name == "cpu package" || name == "package")
                    && (0.0..=2000.0).contains(&v) =>
            {
                t.cpu.power_w = Some(v)
            }
            "Clock"
                if s.parent.contains("cpu")
                    && name == "cores (average)"
                    && (0.0..=20000.0).contains(&v) =>
            {
                t.cpu.freq_mhz = Some(v)
            }
            "Temperature"
                if s.parent.contains("gpu")
                    && name == "gpu core"
                    && t.gpu.temp_c.is_none()
                    && (-20.0..150.0).contains(&v) =>
            {
                t.gpu.temp_c = Some(v)
            }
            _ => {}
        }
    }
}

fn bridge_path() -> std::path::PathBuf {
    std::env::var_os("PRC_SENSOR_FILE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tools/LibreHardwareMonitor/sensors.tsv")
        })
}

fn parse_bridge(data: &str, now: u64) -> Option<Vec<Sensor>> {
    let mut lines = data.lines();
    let timestamp = lines.next()?.parse::<u64>().ok()?;
    if now < timestamp || now - timestamp > 8 {
        return None;
    }
    let rows = lines
        .filter_map(|line| {
            let fields: Vec<_> = line.split('\t').collect();
            if fields.len() != 5 {
                return None;
            }
            let value = fields[4].parse::<f32>().ok().filter(|v| v.is_finite())?;
            Some(Sensor {
                identifier: fields[0].into(),
                name: fields[1].into(),
                parent: fields[2].into(),
                sensor_type: fields[3].into(),
                value: Some(value),
            })
        })
        .collect();
    Some(rows)
}

fn bridge_sensors() -> Option<Vec<Sensor>> {
    let data = std::fs::read_to_string(bridge_path()).ok()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    parse_bridge(&data, now)
}

pub fn list_sensors() {
    if let Some(rows) = bridge_sensors() {
        for s in rows {
            println!(
                "{} | {} | {} | {:?}",
                s.identifier, s.name, s.sensor_type, s.value
            )
        }
    } else {
        println!("No fresh hardware bridge snapshot; run scripts/start-hardware.ps1 (admin).")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_explicit_hardware_sensors_are_used() {
        let sensor = |name: &str, parent: &str, kind: &str, value| Sensor {
            identifier: String::new(),
            name: name.into(),
            parent: parent.into(),
            sensor_type: kind.into(),
            value,
        };
        let rows = vec![
            sensor("CPU Package", "/amdcpu/0", "Temperature", Some(61.0)),
            sensor("CPU", "/mainboard/0", "Temperature", Some(80.0)),
            sensor("CPU Fan", "/mainboard/0", "Fan", Some(1100.0)),
            sensor("Pump", "/mainboard/0", "Fan", Some(f32::NAN)),
        ];
        let mut t = TelemetrySnapshot::default();
        apply_sensors(&rows, &mut t);
        assert_eq!(t.cpu.temp_c, Some(61.0));
        assert_eq!(t.cooling.cpu_fan_rpm, Some(1100));
        assert_eq!(t.cooling.pump_rpm, None);
    }

    #[test]
    fn bridge_rejects_stale_invalid_and_future_readings() {
        let data="100\n/amdcpu/0/temperature/2\tCore (Tctl/Tdie)\t/amdcpu/0\tTemperature\t62\n/x\tBad\t/cpu/0\tTemperature\tNaN\n";
        assert!(parse_bridge(data, 109).is_none());
        assert!(parse_bridge(data, 99).is_none());
        let rows = parse_bridge(data, 105).unwrap();
        assert_eq!(rows.len(), 1);
        let mut t = TelemetrySnapshot::default();
        apply_sensors(&rows, &mut t);
        assert_eq!(t.cpu.temp_c, Some(62.0));
    }
}
