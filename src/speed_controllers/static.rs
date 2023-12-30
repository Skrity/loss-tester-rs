use super::Limiter;

/// Contains Speed Limiting strategies
use std::time::Duration;

/// Naive limiter relying on spinsleep
pub struct StaticLimiter {
    dur: Duration,
}

impl StaticLimiter {
    pub fn new(speed: u32, mtu: u16) -> Self {
        Self {
            dur: Duration::from_micros(if speed == 0 {
                0
            } else {
                1_000_000 / ((speed as u64 * 1024 / 8) / mtu as u64)
            }),
        }
    }
}

impl Limiter for StaticLimiter {
    fn sleep_interval(&mut self) -> Duration {
        self.dur
    }
}
