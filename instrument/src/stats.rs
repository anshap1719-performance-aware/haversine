use crate::cpu_timer::estimate_cpu_frequency;
use std::fmt::{Display, Formatter};
use std::time::Duration;

pub struct Throughput {
    bytes: u64,
    duration: Duration,
}

impl Throughput {
    pub fn new(bytes: u64, duration: impl Into<Duration>) -> Self {
        Self {
            bytes,
            duration: duration.into(),
        }
    }

    /// Total size in MB
    #[must_use]
    pub fn data_processed(&self) -> f64 {
        (self.bytes as f64) / 1024. / 1024.
    }

    /// GB/s processed
    #[must_use]
    pub fn throughput(&self) -> f64 {
        self.data_processed() / 1024. / self.duration.as_secs_f64()
    }
}

impl Display for Throughput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2} GB/s", self.throughput())
    }
}

#[derive(Copy, Clone)]
pub struct RunTime {
    clocks: u64,
    cpu_timer_frequency: u64,
}

impl RunTime {
    #[must_use]
    pub fn new(clocks: u64) -> Self {
        Self {
            clocks,
            cpu_timer_frequency: estimate_cpu_frequency(),
        }
    }

    #[must_use]
    pub fn with_timer_frequency(clocks: u64, timer_frequency: u64) -> Self {
        Self {
            clocks,
            cpu_timer_frequency: timer_frequency,
        }
    }

    #[must_use]
    pub fn elapsed(&self) -> Duration {
        Duration::from_secs_f64(self.clocks as f64 / self.cpu_timer_frequency as f64)
    }
}

impl From<RunTime> for Duration {
    fn from(value: RunTime) -> Self {
        value.elapsed()
    }
}

impl Display for RunTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({:.2}ms)",
            self.clocks,
            self.elapsed().as_secs_f64() * 1000.
        )
    }
}
