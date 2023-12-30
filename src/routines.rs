use std::time::{Duration, Instant};

use spin_sleep::sleep;

use crate::{
    frames::{FrameBuilder, FrameHandler},
    protocols::{Receiver, Sender},
    speed_controllers::Limiter,
};

/// Serial loop for receiving data on a Receiver implementer.
///
/// Reports stats every `report_interval`` second unless blocked.
///
pub fn reciever_loop(mut socket: impl Receiver, report_interval: u8) -> ! {
    let mut handler = FrameHandler::new();
    let mut time = Instant::now();
    let report_interval = Duration::from_secs(report_interval.into());
    loop {
        if time.elapsed() > report_interval {
            println!(
                "Avg receive speed: {} kbps \n{:?}",
                handler.get_avg_kbps(),
                handler.get_statistics()
            );
            time = Instant::now();
        }
        if let Ok(data) = socket.recv() {
            handler.handle(data)
        } else {
            handler.reset();
        }
    }
}

/// Serial loop for sending data over Sender implementer.
///
/// Takes `impl Limiter` for speed adjustment on the fly.
///
pub fn sender_loop(mut socket: impl Sender, mtu: u16, mut limiter: impl Limiter) -> ! {
    let mut builder = FrameBuilder::new(mtu);
    let mut time = Instant::now();
    let report_interval = Duration::from_secs(1);
    loop {
        if time.elapsed() > report_interval {
            println!("Avg send speed: {} kbps", builder.get_avg_kbps());
            time = Instant::now();
        }
        if let Err(_) = &socket.send(builder.next()) {
            std::process::exit(1);
        };
        sleep(limiter.sleep_interval());
    }
}
