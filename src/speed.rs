#![allow(dead_code)]
/// Contains Speed Limiting strategies
use std::{
    ops::Range,
    time::{Duration, Instant},
};

pub trait Limiter {
    fn sleep_interval(&mut self) -> Duration;
}

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

/// Bursty Limiter for optimal CPU usage, not unlike iperf
pub struct BurstLimiter {
    burst_window: Duration,
    burst_count: usize,
    state: Option<(Instant, Range<usize>)>,
    disabled: bool,
}

impl BurstLimiter {
    pub fn new(speed: u32, mtu: u16, burst_window: Duration) -> Self {
        Self {
            burst_window: burst_window,
            burst_count: speed as usize * 1024 / 8 / mtu as usize,
            state: None,
            disabled: if speed == 0 { true } else { false },
        }
    }
}

impl Limiter for BurstLimiter {
    fn sleep_interval(&mut self) -> Duration {
        if self.disabled {
            return Duration::ZERO; // no sleep while disabled
        }
        if let None = &mut self.state {
            self.state = Some((Instant::now(), 0..self.burst_count));
        }
        if let Some((time, range)) = &mut self.state {
            if let Some(_) = range.next() {
                Duration::ZERO // no sleep while some bursts left
            } else {
                let time_left = self.burst_window - time.elapsed();
                self.state = None;
                time_left // Sleep all the time left
            }
        } else {
            unreachable!(); // It's instantiated just before this block
        }
    }
}
