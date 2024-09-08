use super::{AsyncReceiver, AsyncSender, ProtoError, RECV_BUF};
use net2::UdpBuilder;
use smol::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use smol::{future::yield_now, net::UdpSocket as async_socket};
use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    time::Duration,
};

pub struct UdpSender(async_socket);

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
            let socket = smol::block_on(async_socket::bind((bind, 0)))?;
            smol::block_on(socket.connect((peer, port)))?;
            return Ok(Self(socket));
        };
        Ok(Self(socket.try_into()?))
    }
}

impl AsyncSender for UdpSender {
    async fn send(&mut self, data: &[u8]) -> Result<(), ProtoError> {
        self.0.send(data).await?;
        Ok(())
    }
}

impl Drop for UdpSender {
    fn drop(&mut self) {
        let _ = smol::block_on(self.0.send(&[0]));
    }
}

pub struct UdpReceiver {
    socket: async_socket,
    buf: Box<[u8]>,
    client: Option<SocketAddr>,
}

impl AsyncReceiver for UdpReceiver {
    async fn recv(&mut self) -> Result<&[u8], ProtoError> {
        if let Some(client) = self.client {
            let (size, addr) = self.socket.recv_from(&mut self.buf).await?;
            if client != addr {
                return Err(ProtoError::ConflictingClient(addr));
            }
            if size == 1 && self.buf[0] == 0 {
                self.client = None;
                return Err(ProtoError::Disconnected(addr));
            }
            Ok(&self.buf[..size])
        } else {
            let (_size, addr) = self.socket.peek_from(&mut self.buf).await?;
            self.client = Some(addr);
            Err(ProtoError::Connected(addr))
        }
    }
}

impl UdpReceiver {
    pub fn new(peer: Ipv4Addr, port: u16, bind: Ipv4Addr) -> anyhow::Result<Self> {
        let socket = if peer.is_multicast() {
            let socket = UdpBuilder::new_v4()?;
            socket.reuse_address(true)?;
            let socket = socket.bind((bind, port))?;
            socket.join_multicast_v4(&peer, &bind)?;
            socket.try_into()?
        } else {
            smol::block_on(async_socket::bind((bind, port)))?
        };
        Ok(Self {
            socket,
            buf: vec![0; RECV_BUF].into_boxed_slice(),
            client: None,
        })
    }
}
