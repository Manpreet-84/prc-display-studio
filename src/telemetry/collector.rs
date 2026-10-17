//! Live telemetry coordinator: refreshes all providers on a timer and
//! assembles a `TelemetrySnapshot` + history ring buffers.

use std::time::Duration;

use sysinfo::{Disks, Networks, System};

use super::history::History;
use super::TelemetrySnapshot;
use super::{cooling, cpu, gpu, media, memory, network, storage, system};

pub struct Collector {
    sys: System,
    disks: Disks,
    networks: Networks,
    cpu: cpu::CpuProbe,
    gpu: gpu::GpuProbe,
    storage: storage::StorageProbe,
    network: network::NetProbe,
    history: History,
    tick: u64,
    interval: Duration,
    hardware: super::hardware::HardwareProbe,
    cached_storage: Vec<super::StorageStats>,
    last_slow: std::time::Instant,
}

impl Collector {
    /// Initializes all providers once (expensive probes done here, not per tick).
    pub fn new(interval_ms: u64) -> Self {
        let mut sys = System::new();

        sys.refresh_cpu_frequency();
        sys.refresh_memory();
        // CPU load uses native kernel deltas; sysinfo supplies memory/frequency.

        let cpu = cpu::CpuProbe::new();
        let gpu = gpu::GpuProbe::new();
        println!(
            "providers: sysinfo OK | nvml {} | wmi-temp {}",
            if gpu.nvml_ok {
                "OK"
            } else {
                "absent (GPU shows --)"
            },
            "optional LibreHardwareMonitor",
        );
        println!(
            "providers: CPU temp/fans/pump optional via LibreHardwareMonitor | storage-temp -- | media --"
        );

        Self {
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
            sys,
            cpu,
            gpu,
            storage: storage::StorageProbe::new(),
            network: network::NetProbe::new(),
            history: History::default(),
            tick: 0,
            interval: Duration::from_millis(interval_ms),
            hardware: Default::default(),
            cached_storage: Vec::new(),
            last_slow: std::time::Instant::now() - Duration::from_secs(10),
        }
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }

    pub fn snapshot(&mut self) -> TelemetrySnapshot {
        self.tick += 1;

        // Refresh cheap counters.

        self.sys.refresh_memory();

        let gpu = gpu::collect(&self.gpu);
        let memory = memory::collect(&self.sys);
        let slow = self.last_slow.elapsed() >= Duration::from_secs(10);
        if slow {
            self.sys.refresh_cpu_frequency();
        }
        let cpu = cpu::collect(&self.sys, &mut self.cpu);
        let (storage, io_mbs) = if slow {
            self.last_slow = std::time::Instant::now();
            let (values, rate) = storage::collect(&mut self.disks, &mut self.storage);
            self.cached_storage = values.clone();
            (values, rate)
        } else {
            (self.cached_storage.clone(), 0.0)
        };
        let network = network::collect(&mut self.networks, &mut self.network);
        let mut cooling = cooling::collect();
        cooling.gpu_fan_rpm = gpu.fan_rpm;
        let system = system::collect();
        let media = media::collect();

        // History pushes (only present values — no fabricated samples).
        if let Some(v) = cpu.temp_c {
            self.history.cpu_temp.push(v);
        }
        if let Some(v) = cpu.usage_pct {
            self.history.cpu_usage.push(v);
        }
        if let Some(v) = gpu.temp_c {
            self.history.gpu_temp.push(v);
        }
        if let Some(v) = gpu.usage_pct {
            self.history.gpu_usage.push(v);
        }
        if let Some(v) = network.down_mbps {
            self.history.net_down.push(v);
        }
        if let Some(v) = network.up_mbps {
            self.history.net_up.push(v);
        }
        if slow && self.tick > 1 {
            self.history.ssd_activity.push(io_mbs as f32);
        }
        if let Some(v) = cooling.cpu_fan_rpm {
            self.history.cpu_fan.push(v as f32);
        }
        if let Some(v) = cooling.gpu_fan_rpm {
            self.history.gpu_fan.push(v as f32);
        }
        if let Some(v) = cooling.pump_rpm {
            self.history.pump.push(v as f32);
        }

        let mut snapshot = TelemetrySnapshot {
            demo: false,
            cpu,
            gpu,
            memory,
            storage,
            network,
            cooling,
            system,
            media,
            history: self.history.clone(),
        };
        self.hardware.apply(&mut snapshot);
        if let Some(v) = snapshot.cpu.temp_c {
            snapshot.history.cpu_temp.push(v);
            self.history.cpu_temp.push(v);
        }
        if let Some(v) = snapshot.cooling.cpu_fan_rpm {
            snapshot.history.cpu_fan.push(v as f32);
            self.history.cpu_fan.push(v as f32);
        }
        if let Some(v) = snapshot.cooling.pump_rpm {
            snapshot.history.pump.push(v as f32);
            self.history.pump.push(v as f32);
        }
        snapshot
    }
}
