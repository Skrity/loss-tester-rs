// use anyhow::{Error, Result};
use net2::UdpBuilder;
use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    time::Duration,
};

use super::{ProtoError, Receiver, Sender, RECV_BUF};

pub struct UdpSender {
    socket: UdpSocket,
}

impl UdpSender {
    pub fn new(peer: Ipv4Addr, port: u16, bind: Ipv4Addr) -> anyhow::Result<Self> {
        let socket = if peer.is_multicast() {
            let socket = UdpBuilder::new_v4()?;
            // https://stackoverflow.com/questions/14388706/how-do-so-reuseaddr-and-so-reuseport-differ/14388707#14388707
            socket.reuse_address(true)?;
            let socket = socket.bind((bind, 0))?;
            socket.set_multicast_ttl_v4(1)?;
            socket.connect((peer, port))?;
            socket
        } else {
            let socket = UdpSocket::bind((bind, 0))?;
            socket.connect((peer, port))?;
            socket
        };
        Ok(Self { socket })
    }
}

impl Sender for UdpSender {
    fn send(&mut self, data: &[u8]) -> Result<(), ProtoError> {
        self.socket.send(data)?;
        Ok(())
    }
}

impl Drop for UdpSender {
    fn drop(&mut self) {
        let _x = self.socket.send(&[0]);
    }
}

pub struct UdpReceiver {
    socket: UdpSocket,
    buf: Box<[u8]>,
    client: Option<SocketAddr>,
}

impl Receiver for UdpReceiver {
    fn recv<'a>(&mut self) -> Result<&[u8], ProtoError> {
        let (size, addr) = self.socket.recv_from(&mut self.buf)?;
        if self.client.is_none() {
            println!("client connected: {addr}");
            self.client = Some(addr)
        }
        if self.client != Some(addr) {
            eprintln!("datagram from a different client received: ignored");
            return Err(ProtoError::ConflictingClient(addr));
        }
        if size == 1 && self.buf[0] == 0 {
            // Disconnect MSG
            self.client = None;
            println!("client disconnected: {addr}");
            return Err(ProtoError::Disconnected(addr));
        }
        return Ok(&self.buf[..size]);
    }
}

impl UdpReceiver {
    pub fn new(peer: Ipv4Addr, port: u16, bind: Ipv4Addr) -> anyhow::Result<Self> {
        let socket = if peer.is_multicast() {
            let socket = UdpBuilder::new_v4()?;
            socket.reuse_address(true)?;
            let socket = socket.bind((bind, port))?;
            socket.join_multicast_v4(&peer, &bind)?;
            socket
        } else {
            let socket = UdpSocket::bind(&(bind, port))?;
            socket
        };
        socket.set_read_timeout(Some(Duration::from_secs(1)))?;

        Ok(Self {
            socket,
            buf: vec![0; RECV_BUF].into_boxed_slice(),
            client: None,
        })
    }
}
