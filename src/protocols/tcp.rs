use anyhow::{Error, Result};

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};

use super::{Receiver, Sender, RECV_BUF};

pub struct TcpSender {
    socket: BufWriter<TcpStream>,
}

impl TcpSender {
    pub fn new(peer: Ipv4Addr, port: u16, _bind: Ipv4Addr) -> Result<Self> {
        let socket = TcpStream::connect((peer, port))?;
        println!("Connected to server {peer}:{port}");
        Ok(Self {
            socket: BufWriter::new(socket),
        })
    }
}

impl Sender for TcpSender {
    fn send(&mut self, data: &[u8]) -> Result<()> {
        if let Err(e) = self.socket.write_all(data) {
            println!("Disconnected from server. Reason: {e}");
            Err(Error::from(e))
        } else {
            Ok(())
        }
    }
}

pub struct TcpReceiver {
    socket: TcpListener,
    connection: Option<BufReader<TcpStream>>,
    buf: Vec<u8>,
}

impl Receiver for TcpReceiver {
    fn recv<'a>(&mut self) -> Result<&[u8]> {
        let mut conn = if let Some(conn) = self.connection.take() {
            conn
        } else {
            let (conn, addr) = self.socket.accept()?;
            println!("client connected: {addr}");
            BufReader::new(conn)
        };
        self.buf.clear();
        let res = conn.read_until(0, &mut self.buf);
        self.connection = match &res {
            Err(e) => {
                println!("client disconnected: {}", e);
                None
            }
            Ok(_) => Some(conn),
        };
        res?;
        return Ok(&self.buf[..]);
    }
}

impl TcpReceiver {
    pub fn new(bind: Ipv4Addr, port: u16) -> Result<Self> {
        let listener = TcpListener::bind((bind, port))?;

        Ok(Self {
            socket: listener,
            connection: None,
            buf: Vec::with_capacity(RECV_BUF),
        })
    }
}
