use anyhow::Result;
use std::net::{Ipv4Addr, UdpSocket};

use super::{Receiver, Sender, RECV_BUF};

pub struct UnicastSender {
    socket: UdpSocket,
}

impl UnicastSender {
    pub fn new(peer: Ipv4Addr, port: u16, bind: Ipv4Addr) -> Result<Self> {
        let socket = UdpSocket::bind((bind, 0))?;
        socket.connect(&(peer, port))?;
        Ok(Self { socket })
    }
}

impl Sender for UnicastSender {
    fn send(&mut self, data: &[u8]) -> Result<()> {
        self.socket.send(data)?;
        Ok(())
    }
}

pub struct UnicastReceiver {
    socket: UdpSocket,
    buf: Box<[u8]>,
}

impl Receiver for UnicastReceiver {
    fn recv<'a>(&mut self) -> Result<&[u8]> {
        let (size, _addr) = self.socket.recv_from(&mut self.buf)?;
        return Ok(&self.buf[..size]);
    }
}

impl UnicastReceiver {
    pub fn new(bind: Ipv4Addr, port: u16) -> Result<Self> {
        Ok(Self {
            socket: UdpSocket::bind(&(bind, port))?,
            buf: vec![0; RECV_BUF].into_boxed_slice(),
        })
    }
}
