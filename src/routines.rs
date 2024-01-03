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
        let mut need_to_print_header = true;
        std::thread::spawn(move || {
            loop {
                if rx.try_recv().is_ok() {
                    return ();
                }
                {
                    let guard = handler.read().unwrap();
                    if let Some(stats) = guard.get_statistics() {
                        if need_to_print_header {
                            need_to_print_header = false;
                            println!("[ ID]    Latency      Bitrate   Sess.Avg. |Bad, Mangled|  Lost/Total")
                        }
                        let (avg, instant) = guard.get_speeds();
                        let total = stats.valid + stats.invalid + stats.lost + stats.internally_bad;
                        let invalid = stats.invalid;
                        let internally_bad = stats.internally_bad;
                        let lost = stats.lost;
                        let percent = lost as f64 / total as f64 * 100_f64;
                        println!("[{: >3}] {: >8}us {instant: >8}kbps {avg: >8}kbps {pad: >3}|{invalid}, {internally_bad}| {pad: >5}{lost}/{total} ({percent:.2}%)", stats.session_id, guard.get_latency(), pad="");
                    } else {
                        need_to_print_header = true;
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
                eprintln!("Peer connected: {peer}");
            },
            Err(ProtoError::Disconnected(peer)) => {
                eprintln!("Peer disconnected: {peer}");
                handler.write().unwrap().reset();
            },
            Err(ProtoError::IOErr(_err)) => {
                // TODO: maybe nonblock, thonk
                // if err.kind() == std::io::ErrorKind::WouldBlock {
                //     std::thread::sleep(Duration::from_micros(1))
                // } else {

                // }
            },
            Err(ProtoError::ConflictingClient(peer)) => {
                eprintln!("Datagram from a different peer ignored: {peer}");
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
