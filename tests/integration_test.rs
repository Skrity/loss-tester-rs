use std::time::Duration;

use loss_tester_rs::speed_controllers::{self, Limiter};

#[test]
fn basic_test() {
    assert!(true)
}

#[test]
fn test_unlimited() {
    let mut x = speed_controllers::UnLimiter::new();
    assert_eq!(x.sleep_interval(), Duration::ZERO)
}