#[cfg(feature = "async")]
use crate::protocols::{AsyncReceiver, AsyncSender};

#[cfg(not(feature = "async"))]
use crate::protocols::{Receiver, Sender};
#[allow(unused)]
use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use anyhow::Result;
type ShutdownReceiver = std::sync::mpsc::Receiver<()>;

use crate::{
    frames::{FrameBuilder, FrameHandler},
    protocols::ProtoError,
    speed_controllers::Limiter,
};

/// Serial loop for receiving data on a Receiver implementer.
///
/// Reports stats every `report_interval`` second unless blocked.
///
#[cfg(not(feature = "async"))]
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
        std::thread::spawn(move || loop {
            if rx.try_recv().is_ok() {
                return;
            }
            {
                let guard = handler.read().unwrap();
                if let Some(stats) = guard.get_statistics() {
                    if need_to_print_header {
                        need_to_print_header = false;
                        println!(
                            "[ ID]    Latency      Bitrate   Sess.Avg. |Bad, Mangled|  Lost/Total"
                        )
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
            }
            Err(ProtoError::Connected(peer)) => {
                eprintln!("Peer connected: {peer}");
            }
            Err(ProtoError::Disconnected(peer)) => {
                eprintln!("Peer disconnected: {peer}");
                handler.write().unwrap().reset();
            }
            Err(ProtoError::IOErr(_err)) => {
                // TODO: maybe nonblock, thonk
                // if err.kind() == std::io::ErrorKind::WouldBlock {
                //     std::thread::sleep(Duration::from_micros(1))
                // } else {

                // }
            }
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
#[cfg(not(feature = "async"))]
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
        if socket.send(builder.next()).is_err() {
            return Ok(());
        };
        spin_sleep::sleep(limiter.sleep_interval());
    }
}

#[cfg(feature = "async")]
pub(super) fn sender_loop(
    socket: impl AsyncSender,
    mtu: u16,
    limiter: impl Limiter,
    shutdown: ShutdownReceiver,
) -> Result<()> {
    smol::block_on(r#async::sender_loop_async(socket, mtu, limiter, shutdown))
}

#[cfg(feature = "async")]
pub(super) fn reciever_loop(
    socket: impl AsyncReceiver,
    report_interval: u8,
    shutdown: ShutdownReceiver,
) -> Result<()> {
    smol::block_on(r#async::reciever_loop_async(
        socket,
        report_interval,
        shutdown,
    ))
}

#[cfg(feature = "async")]
mod r#async {
    use super::*;
    use smol::future::{poll_once, yield_now};
    use smol::stream::StreamExt;
    use smol::Timer;

    pub(super) async fn sender_loop_async(
        mut socket: impl AsyncSender,
        mtu: u16,
        mut limiter: impl Limiter,
        shutdown: ShutdownReceiver,
    ) -> Result<()> {
        let mut builder = FrameBuilder::new(mtu);
        let mut timer = Timer::interval(Duration::from_secs(1));
        loop {
            if shutdown.try_recv().is_ok() {
                return Ok(());
            }
            if poll_once(timer.next()).await.is_some() {
                println!("Avg send speed: {} kbps", builder.get_avg_kbps());
            }
            if socket.send(builder.next()).await.is_err() {
                return Ok(());
            };
            let to_sleep = limiter.sleep_interval();
            if to_sleep != Duration::ZERO {
                Timer::after(to_sleep).await;
            }
        }
    }

    /// Serial loop for receiving data on a Receiver implementer.
    ///
    /// Reports stats every `report_interval`` second unless blocked.
    ///
    pub(super) async fn reciever_loop_async(
        mut socket: impl AsyncReceiver,
        report_interval: u8,
        shutdown: ShutdownReceiver,
    ) -> Result<()> {
        let mut handler = FrameHandler::new();
        let mut need_to_print_header = true;
        let mut timer = Timer::interval(Duration::from_secs(report_interval.into()));
        loop {
            if shutdown.try_recv().is_ok() {
                return Ok(());
            }
            if poll_once(timer.next()).await.is_some() {
                if let Some(stats) = handler.get_statistics() {
                    if need_to_print_header {
                        need_to_print_header = false;
                        println!(
                            "[ ID]    Latency      Bitrate   Sess.Avg. |Bad, Mangled|  Lost/Total"
                        )
                    }
                    let (avg, instant) = handler.get_speeds();
                    let total = stats.valid + stats.invalid + stats.lost + stats.internally_bad;
                    let invalid = stats.invalid;
                    let internally_bad = stats.internally_bad;
                    let lost = stats.lost;
                    let percent = lost as f64 / total as f64 * 100_f64;
                    println!("[{: >3}] {: >8}us {instant: >8}kbps {avg: >8}kbps {pad: >3}|{invalid}, {internally_bad}| {pad: >5}{lost}/{total} ({percent:.2}%)", stats.session_id, handler.get_latency(), pad="");
                } else {
                    need_to_print_header = true;
                }
            };
            match poll_once(socket.recv()).await {
                Some(Ok(data)) => {
                    handler.handle(data);
                }
                Some(Err(ProtoError::Connected(peer))) => {
                    eprintln!("Peer connected: {peer}");
                }
                Some(Err(ProtoError::Disconnected(peer))) => {
                    eprintln!("Peer disconnected: {peer}");
                    handler.reset();
                }
                Some(Err(ProtoError::IOErr(_err))) => {
                    eprintln!("stuff happened: {_err}")
                }
                Some(Err(ProtoError::ConflictingClient(peer))) => {
                    eprintln!("Datagram from a different peer ignored: {peer}");
                }
                None => {
                    Timer::after(Duration::from_millis(1)).await;
                    continue;
                }
            }
            yield_now().await;
        }
    }
}
