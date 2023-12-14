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

/// Useless Limiter, doesn't limit
pub struct UnLimiter {}

impl UnLimiter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Limiter for UnLimiter {
    fn sleep_interval(&mut self) -> Duration {
        Duration::ZERO
    }
}

/// Bursty Limiter for optimal CPU usage, not unlike iperf
pub struct BurstLimiter {
    burst_window: Duration,
    burst_count: u64,
    state: Option<(Instant, Range<u64>)>,
    disabled: bool,
}

impl BurstLimiter {
    pub fn new(speed: u32, mtu: u16, dynamic_window: bool) -> Self {
        let frames_per_second = (Into::<u64>::into(speed) * (1024 / 8)) / Into::<u64>::into(mtu);
        let window: u64 = if dynamic_window {
            match frames_per_second {
                0 => 1,
                1..=99 => 1_000_000,
                1..=999 => 100_000,
                1000..=9999 => 10_000,
                10000..=99999 => 100,
                100000.. => 1,
            }
        } else {
            1_000_000
        };
        println!("frames_per_second={frames_per_second}, window={window}");
        Self {
            burst_window: Duration::from_micros(window),
            burst_count: frames_per_second / (1_000_000 / window),
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
        let (time, mut range) = self
            .state
            .take()
            .unwrap_or((Instant::now(), 0..self.burst_count));
        if let Some(_) = range.next() {
            self.state = Some((time, range));
            Duration::ZERO // no sleep while some bursts left
        } else {
            self.state = None;
            self.burst_window
                .checked_sub(time.elapsed())
                .unwrap_or(Duration::ZERO) // sleep all the remaining burst time
        }
    }
}
