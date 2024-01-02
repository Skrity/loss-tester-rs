use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream, SocketAddr};

use super::{ProtoError, Receiver, Sender, RECV_BUF};

pub struct TcpSender {
    socket: BufWriter<TcpStream>,
}

impl TcpSender {
    pub fn new(peer: Ipv4Addr, port: u16, _bind: Ipv4Addr) -> anyhow::Result<Self> {
        let socket = TcpStream::connect((peer, port))?;
        println!("Connected to server {peer}:{port}");
        Ok(Self {
            socket: BufWriter::new(socket),
        })
    }
}

impl Sender for TcpSender {
    fn send(&mut self, data: &[u8]) -> Result<(), ProtoError> {
        if let Err(e) = self.socket.write_all(data) {
            eprintln!("Disconnected from server. Reason: {e}");
            Err(ProtoError::Disconnected(self.socket.get_ref().peer_addr()?))
        } else {
            Ok(())
        }
    }
}

impl Drop for TcpSender {
    fn drop(&mut self) {
        let _x = self.socket.write_all(&[0]);
    }
}

pub struct TcpReceiver {
    socket: TcpListener,
    connection: Option<(BufReader<TcpStream>, SocketAddr)>,
    buf: Vec<u8>,
}

impl Receiver for TcpReceiver {
    fn recv<'a>(&mut self) -> Result<&[u8], ProtoError> {
        self.buf.clear();
        if let Some((mut conn, addr)) = self.connection.take() {
            if conn.read_until(0, &mut self.buf).is_err() || (self.buf.len() == 1 && self.buf[0] == 0) {
                return Err(ProtoError::Disconnected(addr))
            } else {
                self.connection = Some((conn, addr));
                return Ok(&self.buf[..]);
            };
        } else {
            let (conn, addr) = self.socket.accept()?;
            self.connection = Some((BufReader::new(conn), addr));
            return Err(ProtoError::Connected(addr));
        }
    }
}

impl TcpReceiver {
    pub fn new(bind: Ipv4Addr, port: u16) -> anyhow::Result<Self> {
        let listener = TcpListener::bind((bind, port))?;

        Ok(Self {
            socket: listener,
            connection: None,
            buf: Vec::with_capacity(RECV_BUF),
        })
    }
}
