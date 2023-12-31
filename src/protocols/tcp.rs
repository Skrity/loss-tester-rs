use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};

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
    connection: Option<BufReader<TcpStream>>,
    buf: Vec<u8>,
}

impl Receiver for TcpReceiver {
    fn recv<'a>(&mut self) -> Result<&[u8], ProtoError> {
        let mut conn = if let Some(conn) = self.connection.take() {
            conn
        } else {
            let (conn, addr) = self.socket.accept()?;
            println!("client connected: {addr}");
            BufReader::new(conn)
        };
        self.buf.clear();
        let res = conn.read_until(0, &mut self.buf);
        match &res {
            Err(e) => {
                println!("client disconnected: {}", e);
                self.connection = None;
                return Err(ProtoError::Disconnected(conn.get_ref().peer_addr()?));
            }
            Ok(_) => self.connection = Some(conn),
        };
        res?;
        return Ok(&self.buf[..]);
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
