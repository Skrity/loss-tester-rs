pub mod args;
pub mod frames;
pub mod protocols;
pub mod routines;
pub mod speed_controllers;

use anyhow::Result;
use args::*;
use clap::Parser;
use protocols::{
    MulticastReceiver, MulticastSender, TcpReceiver, TcpSender, UnicastReceiver, UnicastSender,
};
use routines::*;
#[allow(unused_imports)]
use speed_controllers::{BurstLimiter, Limiter, OverTimeLimiter, StaticLimiter};

pub fn entrypoint() -> Result<()> {
    let args = Args::parse();
    match (args.r#type, args.proto) {
        (
            Commands::Server {
                addr,
                port,
                interval,
            },
            Proto::Multicast,
        ) => reciever_loop(MulticastReceiver::new(addr, port, args.bind)?, interval),
        (
            Commands::Server {
                addr,
                port,
                interval,
            },
            Proto::Unicast,
        ) => reciever_loop(UnicastReceiver::new(addr, port)?, interval),
        (
            Commands::Server {
                addr,
                port,
                interval,
            },
            Proto::TCP,
        ) => reciever_loop(TcpReceiver::new(addr, port)?, interval),

        (
            Commands::Client {
                addr,
                port,
                mtu,
                bandwidth,
            },
            Proto::Multicast,
        ) => sender_loop(
            MulticastSender::new(addr, port, args.bind)?,
            mtu - 28,
            BurstLimiter::new(bandwidth, mtu, true),
        ),
        (
            Commands::Client {
                addr,
                port,
                mtu,
                bandwidth,
            },
            Proto::Unicast,
        ) => sender_loop(
            UnicastSender::new(addr, port, args.bind)?,
            mtu - 28,
            BurstLimiter::new(bandwidth, mtu, true),
        ),
        (
            Commands::Client {
                addr,
                port,
                mtu,
                bandwidth,
            },
            Proto::TCP,
        ) => sender_loop(
            TcpSender::new(addr, port, args.bind)?,
            mtu - 40,
            BurstLimiter::new(bandwidth, mtu, false),
        ),
    }
}
