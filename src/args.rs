use std::net::Ipv4Addr;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Subcommand)]
pub enum Commands {
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
        /// Limit transmission bandwidth, kbit/s (0 to disable limiting)
        bandwidth: u32,

        #[arg(short, long, default_value_t = 1500)]
        /// Maximum Transmission Unit
        mtu: u16,
    },
}

#[derive(Clone, ValueEnum)]
pub enum Proto {
    Multicast,
    Unicast,
    TCP,
}

/// Program to detect network packet loss and packet mangling
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub r#type: Commands,

    #[arg(short='P', long, value_enum, default_value_t = Proto::Multicast)]
    /// Protocol to send data over
    pub proto: Proto,

    #[arg(short = 'B', long, default_value_t = Ipv4Addr::UNSPECIFIED)]
    /// IP address to bind to
    pub bind: Ipv4Addr,
}
