mod tcp;
mod udp;

use std::net::SocketAddr;

pub use tcp::{TcpReceiver, TcpSender};
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
    fn recv<'a>(&mut self) -> Result<&[u8], ProtoError>;
}

#[derive(Error, Debug)]
pub enum ProtoError {
    #[error("Client {0} disconnected")]
    Disconnected(SocketAddr),
    #[error("Client {0} is not connected: datagram ignored")]
    ConflictingClient(SocketAddr),
    #[error("IO Error: {0}")]
    IOErr(#[from] std::io::Error),
}
