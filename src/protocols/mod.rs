#![allow(unused)]

#[cfg(feature = "async")]
mod async_tcp;
#[cfg(feature = "async")]
mod async_udp;

#[cfg(not(feature = "async"))]
mod tcp;
#[cfg(not(feature = "async"))]
mod udp;

#[cfg(feature = "async")]
pub use async_tcp::{TcpReceiver, TcpSender};
#[cfg(feature = "async")]
pub use async_udp::{UdpReceiver, UdpSender};

use std::net::SocketAddr;
#[cfg(not(feature = "async"))]
pub use tcp::{TcpReceiver, TcpSender};
#[cfg(not(feature = "async"))]
pub use udp::{UdpReceiver, UdpSender};

use thiserror::Error;

/// Buffer size for receive operations
///
/// Correlates to MAX_FRAME_SIZE in protocols
const RECV_BUF: usize = 65536;

pub trait Sender {
    fn send(&mut self, data: &[u8]) -> Result<(), ProtoError>;
}

pub trait Receiver {
    fn recv(&mut self) -> Result<&[u8], ProtoError>;
}

pub(crate) trait AsyncSender {
    async fn send(&mut self, data: &[u8]) -> Result<(), ProtoError>;
}

pub(crate) trait AsyncReceiver {
    async fn recv(&mut self) -> Result<&[u8], ProtoError>;
}

#[derive(Error, Debug)]
pub enum ProtoError {
    #[error("Client {0} connected")]
    Connected(SocketAddr),
    #[error("Client {0} disconnected")]
    Disconnected(SocketAddr),
    #[error("Client {0} is not connected: datagram ignored")]
    ConflictingClient(SocketAddr),
    #[error("IO Error: {0}")]
    IOErr(#[from] std::io::Error),
}
