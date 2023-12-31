use std::time::{Duration, Instant};

use anyhow::Result;
use spin_sleep::sleep;
type ShutdownReceiver = std::sync::mpsc::Receiver<()>;

use crate::{
    frames::{FrameBuilder, FrameHandler},
    protocols::{ProtoError, Receiver, Sender},
    speed_controllers::Limiter,
};

/// Serial loop for receiving data on a Receiver implementer.
///
/// Reports stats every `report_interval`` second unless blocked.
///
pub fn reciever_loop(
    mut socket: impl Receiver,
    report_interval: u8,
    shutdown: ShutdownReceiver,
) -> Result<()> {
    let mut handler = FrameHandler::new();
    let mut time = Instant::now();
    let report_interval = Duration::from_secs(report_interval.into());
    loop {
        if shutdown.try_recv().is_ok() {
            return Ok(());
        }
        if time.elapsed() > report_interval {
            println!(
                "Avg receive speed: {} kbps \n{:?}",
                handler.get_avg_kbps(),
                handler.get_statistics()
            );
            time = Instant::now();
        }
        match socket.recv() {
            Ok(data) => handler.handle(data),
            Err(ProtoError::Disconnected(_)) => handler.reset(),
            Err(ProtoError::ConflictingClient(_)) => {}
            Err(ProtoError::IOErr(err)) => {
                eprintln!("Error receiving datagram: {err}")
            }
        }
    }
}

/// Serial loop for sending data over Sender implementer.
///
/// Takes `impl Limiter` for speed adjustment on the fly.
///
pub fn sender_loop(
    mut socket: impl Sender,
    mtu: u16,
    mut limiter: impl Limiter,
    shutdown: ShutdownReceiver,
) -> Result<()> {
    let mut builder = FrameBuilder::new(mtu);
    let mut time = Instant::now();
    let report_interval = Duration::from_secs(1);
    loop {
        if shutdown.try_recv().is_ok() {
            return Ok(());
        }
        if time.elapsed() > report_interval {
            println!("Avg send speed: {} kbps", builder.get_avg_kbps());
            time = Instant::now();
        }
        if let Err(_) = &socket.send(builder.next()) {
            return Ok(());
        };
        sleep(limiter.sleep_interval());
    }
}
