//! Fixed-capacity time-series ring buffers (60–120 samples for LCD graphs).

use std::collections::VecDeque;

/// One metric series with bounded memory.
#[derive(Debug, Clone)]
pub struct Series {
    data: VecDeque<f32>,
    cap: usize,
}

impl Default for Series {
    fn default() -> Self {
        Self::new(120)
    }
}

impl Series {
    pub fn new(cap: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(cap),
            cap: cap.max(2),
        }
    }

    pub fn push(&mut self, v: f32) {
        if self.data.len() == self.cap {
            self.data.pop_front();
        }
        self.data.push_back(v);
    }

    pub fn last(&self) -> Option<f32> {
        self.data.back().copied()
    }

    pub fn values(&self) -> Vec<f32> {
        self.data.iter().copied().collect()
    }

    pub fn avg(&self) -> Option<f32> {
        if self.data.is_empty() {
            return None;
        }
        let sum: f32 = self.data.iter().sum();
        Some(sum / self.data.len() as f32)
    }

    pub fn max(&self) -> Option<f32> {
        self.data.iter().cloned().fold(None, |a: Option<f32>, b| Some(a.map_or(b, |a| a.max(b))))
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// All graph series carried in every `TelemetrySnapshot`.
#[derive(Debug, Clone, Default)]
pub struct History {
    pub cpu_temp: Series,
    pub gpu_temp: Series,
    pub cpu_usage: Series,
    pub gpu_usage: Series,
    pub net_down: Series,
    pub net_up: Series,
    pub ssd_activity: Series,
    pub cpu_fan: Series,
    pub gpu_fan: Series,
    pub pump: Series,
}
