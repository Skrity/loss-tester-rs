mod multicast;
mod tcp;
mod unicast;

pub use multicast::{MulticastReceiver, MulticastSender};
pub use tcp::{TcpReceiver, TcpSender};
pub use unicast::{UnicastReceiver, UnicastSender};

use anyhow::Result;

pub trait Sender {
    fn send(&mut self, data: &[u8]) -> Result<()>;
}

pub trait Receiver {
    fn recv<'a>(&mut self, buf: &'a mut [u8]) -> Result<&'a [u8]>;
}
