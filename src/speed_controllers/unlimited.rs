use std::time::Duration;

use super::Limiter;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unlimiter() {
        let mut limiter = UnLimiter::new();
        assert_eq!(limiter.sleep_interval(), Duration::ZERO);
    }
}
