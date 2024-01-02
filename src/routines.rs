use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

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
    let handler = Arc::new(RwLock::new(FrameHandler::new()));
    let print_killer = {
        let handler = handler.clone();
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        std::thread::spawn(move || {
            loop {
                if rx.try_recv().is_ok() {
                    return ();
                }
                {
                    let guard = handler.read().unwrap();
                    let stats = guard.get_statistics();
                    if let Some(stats) = stats {
                        let (avg, instant) = guard.get_speeds();
                        println!("Lmao latency: {}us stats: {:?} speed: {avg}, {instant}", guard.get_latency(), stats)
                    }
                }
                std::thread::sleep(Duration::from_secs(report_interval.into()));
            }
        });
        tx
    };
    loop {
        if shutdown.try_recv().is_ok() {
            let _ = print_killer.send(());
            return Ok(());
        }
        match socket.recv() {
            Ok(data) => {
                handler.write().unwrap().handle(data);
            },
            Err(ProtoError::Connected(peer)) => {
                eprintln!("Connected: {peer}")
            },
            Err(ProtoError::Disconnected(peer)) => {
                eprintln!("Disconnected: {peer}");
                handler.write().unwrap().reset();
            },
            Err(ProtoError::IOErr(_)) => {},
            Err(ProtoError::ConflictingClient(peer)) => {
                eprintln!("Datagram from a different client ignored: {peer}");
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
