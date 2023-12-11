use anyhow::Result;
use std::io::{Read, Write};
use std::net::Ipv4Addr;

use std::net::{TcpListener, TcpStream};

use super::{Receiver, Sender};

pub struct TcpSender {
    socket: TcpStream,
}

/// Tcp sometimes incur losses on Windows. (investigate?)
impl TcpSender {
    pub fn new(peer: Ipv4Addr, port: u16, _bind: Ipv4Addr) -> Result<Self> {
        let socket = TcpStream::connect((peer, port))?;
        socket.set_nodelay(true)?; // might not be needed?
        println!("Connected to {peer}:{port}");
        Ok(Self { socket })
    }
}

impl Sender for TcpSender {
    fn send(&mut self, data: &[u8]) -> Result<()> {
        let _res = self.socket.write(data)?;
        self.socket.flush()?; // might not be needed?
        Ok(())
    }
}

pub struct TcpReceiver {
    socket: TcpListener,
    connection: Option<TcpStream>,
}

impl Receiver for TcpReceiver {
    fn recv<'a>(&mut self, buf: &'a mut [u8]) -> Result<&'a [u8]> {
        if self.connection.is_none() {
            let (conn, addr) = self.socket.accept()?;
            self.connection = Some(conn);
            println!("client connected: {addr}")
        }
        let mut conn = self.connection.take().unwrap();
        let size = conn.read(buf);
        self.connection = if size.is_err() {
            println!("client disconnected");
            None
        } else {
            Some(conn)
        };
        return Ok(&buf[..size?]);
    }
}

impl TcpReceiver {
    pub fn new(bind: Ipv4Addr, port: u16) -> Result<Self> {
        let listener = TcpListener::bind((bind, port))?;

        Ok(Self {
            socket: listener,
            connection: None,
        })
    }
}
