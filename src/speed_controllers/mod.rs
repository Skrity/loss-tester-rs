mod r#static;
pub use r#static::StaticLimiter;

mod unlimited;
pub use unlimited::UnLimiter;

mod burst;
pub use burst::BurstLimiter;

mod overtime;
pub use overtime::OverTimeLimiter;

use std::time::Duration;

pub trait Limiter {
    fn sleep_interval(&mut self) -> Duration;
}
