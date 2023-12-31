pub mod args;
pub mod frames;
pub mod protocols;
pub mod routines;
pub mod speed_controllers;

use anyhow::Result;
use args::*;
use clap::Parser;
use ctrlc;
use protocols::{TcpReceiver, TcpSender, UdpReceiver, UdpSender};
use routines::*;
use speed_controllers::UnLimiter;
#[allow(unused_imports)]
use speed_controllers::{BurstLimiter, Limiter, OverTimeLimiter, StaticLimiter};
use std::sync::mpsc;

pub fn entrypoint() -> Result<()> {
    let args = Args::parse();
    let (tx, rx) = mpsc::channel::<()>();
    let _ = ctrlc::set_handler(move || {
        let _ = tx.send(());
    });

    match args.r#type {
        Commands::Server {
            addr,
            port,
            interval,
        } => match args.proto {
            Proto::UDP => reciever_loop(UdpReceiver::new(addr, port, args.bind)?, interval, rx),
            Proto::TCP => reciever_loop(TcpReceiver::new(addr, port)?, interval, rx),
        },
        Commands::Client {
            addr,
            port,
            bandwidth,
            mtu,
        } => match (args.proto, bandwidth) {
            (Proto::UDP, 0) => sender_loop(
                UdpSender::new(addr, port, args.bind)?,
                mtu - 28,
                BurstLimiter::new(1000, mtu, true),
                rx,
            ),
            (Proto::TCP, 0) => sender_loop(
                TcpSender::new(addr, port, args.bind)?,
                mtu - 40,
                UnLimiter::new(),
                rx,
            ),
            (Proto::UDP, bandwidth) => sender_loop(
                UdpSender::new(addr, port, args.bind)?,
                mtu - 28,
                BurstLimiter::new(bandwidth, mtu, true),
                rx,
            ),
            (Proto::TCP, bandwidth) => sender_loop(
                TcpSender::new(addr, port, args.bind)?,
                mtu - 40,
                BurstLimiter::new(bandwidth, mtu, false),
                rx,
            ),
        },
    }
}
