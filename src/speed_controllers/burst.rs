use std::{
    ops::Range,
    time::{Duration, Instant},
};

use super::Limiter;

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
                100..=999 => 100_000,
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
            disabled: speed == 0,
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
        if range.next().is_some() {
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
