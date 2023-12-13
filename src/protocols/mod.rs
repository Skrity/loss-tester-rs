mod multicast;
mod tcp;
mod unicast;

pub use multicast::{MulticastReceiver, MulticastSender};
pub use tcp::{TcpReceiver, TcpSender};
pub use unicast::{UnicastReceiver, UnicastSender};

use anyhow::Result;

/// Buffer size for receive operations
///
/// Correlates to MAX_FRAME_SIZE in protocols
const RECV_BUF: usize = 65536;

pub trait Sender {
    fn send(&mut self, data: &[u8]) -> Result<()>;
}

pub trait Receiver {
    fn recv<'a>(&mut self) -> Result<&[u8]>;
}
