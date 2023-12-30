use std::time::{Duration, Instant};

use super::Limiter;

/// Toy limiter for testing
pub struct OverTimeLimiter {
    speed: u32,
    mtu: u16,
    time: Instant,
}

impl OverTimeLimiter {
    pub fn new(speed: u32, mtu: u16) -> Self {
        Self {
            speed,
            mtu,
            time: Instant::now(),
        }
    }
}

impl Limiter for OverTimeLimiter {
    fn sleep_interval(&mut self) -> Duration {
        let time_elapsed = self.time.elapsed().as_secs();
        let limit_magnitude = 50;
        let mut speed_reduced =
            ((self.speed as u64 - time_elapsed * limit_magnitude) * 1024 / 8) / self.mtu as u64;
        if speed_reduced == 0 {
            speed_reduced = 1;
            self.time = Instant::now();
        }
        Duration::from_micros(1_000_000 / speed_reduced)
    }
}
