mod frames;
mod protocols;
mod speed;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use frames::{FrameBuilder, FrameHandler};
use protocols::{
    MulticastReceiver, MulticastSender, Receiver, Sender, TcpReceiver, TcpSender, UnicastReceiver,
    UnicastSender,
};
use spin_sleep::sleep;
use std::{
    net::Ipv4Addr,
    time::{Duration, Instant},
};
#[allow(unused_imports)]
use speed::{Limiter, OverTimeLimiter, StaticLimiter};

#[derive(Subcommand)]
enum Commands {
    /// Server mode
    Server {
        /// IP address to serve on
        addr: Ipv4Addr,

        /// Port to serve on
        #[arg(default_value_t = 5000)]
        port: u16,

        #[arg(short = 'I', long, default_value_t = 1)]
        /// Interval between reports
        interval: u8,
    },
    /// Client mode
    Client {
        /// IP address to connect to
        addr: Ipv4Addr,

        /// Port to connect to
        #[arg(default_value_t = 5000)]
        port: u16,

        #[arg(short, long, default_value_t = 1000)]
        /// Limit transmission bandwidth, kbit/s
        bandwidth: u32,

        #[arg(short, long, default_value_t = 1500)]
        /// Maximum Transmission Unit
        mtu: u16,
    },
}

#[derive(Clone, ValueEnum)]
enum Proto {
    Multicast,
    Unicast,
    TCP,
}

/// Program to detect network packet loss and packet mangling
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    r#type: Commands,

    #[arg(short='P', long, value_enum, default_value_t = Proto::Multicast)]
    /// Protocol to send data over
    proto: Proto,

    #[arg(short = 'B', long, default_value_t = Ipv4Addr::UNSPECIFIED)]
    /// IP address to bind to
    bind: Ipv4Addr,
}

fn reciever_loop(mut socket: impl Receiver, report_interval: u8) -> ! {
    let mut handler = FrameHandler::new();
    let mut buf = [0_u8; 65536];
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
        if let Ok(data) = socket.recv(&mut buf) {
            handler.handle(data);
        } else {
            handler.reset();
        }
        sleep(Duration::from_micros(50));
    }
}


fn sender_loop(mut socket: impl Sender, mtu: u16, mut limiter: impl Limiter) -> ! {
    let mut builder = FrameBuilder::new(mtu);
    let mut time = Instant::now();
    let report_interval = Duration::from_secs(1);
    loop {
        if time.elapsed() > report_interval {
            println!("Avg send speed: {} kbps", builder.get_avg_kbps());
            time = Instant::now();
        }
        let data = builder.next();
        let _ = &socket.send(data);
        sleep(limiter.sleep_interval());
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    match args.r#type {
        Commands::Server {
            addr,
            port,
            interval,
        } => {
            match args.proto {
                Proto::Multicast => {
                    reciever_loop(MulticastReceiver::new(addr, port, args.bind)?, interval)
                }
                Proto::Unicast => reciever_loop(UnicastReceiver::new(addr, port)?, interval),
                Proto::TCP => reciever_loop(TcpReceiver::new(addr, port)?, interval),
            };
        }
        Commands::Client {
            addr,
            port,
            mtu,
            bandwidth,
        } => match args.proto {
            Proto::Multicast => sender_loop(
                MulticastSender::new(addr, port, args.bind)?,
                mtu - 28,
                StaticLimiter::new(bandwidth, mtu),
            ),
            Proto::Unicast => sender_loop(
                UnicastSender::new(addr, port, args.bind)?,
                mtu - 28,
                OverTimeLimiter::new(bandwidth, mtu),
            ),
            Proto::TCP => sender_loop(
                TcpSender::new(addr, port, args.bind)?,
                mtu - 40,
                StaticLimiter::new(bandwidth, mtu),
            ),
        },
    }
}
