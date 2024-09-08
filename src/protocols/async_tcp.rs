use std::collections::VecDeque;
use std::net::{Ipv4Addr, SocketAddr};

use super::{AsyncReceiver as Receiver, AsyncSender as Sender, ProtoError, RECV_BUF};
use smol::future::{poll_once, yield_now};
use smol::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use smol::net::{TcpListener, TcpStream};
pub struct TcpSender {
    socket: BufWriter<TcpStream>,
}

impl TcpSender {
    pub fn new(peer: Ipv4Addr, port: u16, _bind: Ipv4Addr) -> anyhow::Result<Self> {
        let socket = smol::block_on(TcpStream::connect((peer, port)))?;
        println!("Connected to server {peer}:{port}");
        Ok(Self {
            socket: BufWriter::new(socket),
        })
    }
}

impl Sender for TcpSender {
    async fn send(&mut self, data: &[u8]) -> Result<(), ProtoError> {
        if let Err(e) = self.socket.write_all(data).await {
            eprintln!("Disconnected from server. Reason: {e}");
            Err(ProtoError::Disconnected(self.socket.get_ref().peer_addr()?))
        } else {
            Ok(())
        }
    }
}

impl Drop for TcpSender {
    fn drop(&mut self) {
        let _x = smol::block_on(self.socket.write_all(&[0]));
    }
}

struct TcpClient {
    stream: BufReader<TcpStream>,
    addr: SocketAddr,
}

pub struct TcpReceiver {
    socket: TcpListener,
    connection: Option<TcpClient>, // To support multiple clients use VecDeque or LinkedList
    buf: Vec<u8>,
}

impl Receiver for TcpReceiver {
    async fn recv(&mut self) -> Result<&[u8], ProtoError> {
        if let Some(TcpClient {
            stream: ref mut conn,
            addr,
        }) = self.connection
        {
            self.buf.clear();
            match conn.read_until(0, &mut self.buf).await {
                Ok(_) if (!self.buf.is_empty() && self.buf[0] != 0) => Ok(&self.buf[..]),
                _ => {
                    self.connection = None;
                    Err(ProtoError::Disconnected(addr))
                }
            }
        } else {
            let (conn, addr) = self.socket.accept().await?;
            self.connection = Some(TcpClient {
                stream: BufReader::new(conn),
                addr,
            });
            Err(ProtoError::Connected(addr))
        }
    }
}

impl TcpReceiver {
    pub fn new(bind: Ipv4Addr, port: u16) -> anyhow::Result<Self> {
        let listener = smol::block_on(TcpListener::bind((bind, port)))?;

        Ok(Self {
            socket: listener,
            connection: None,
            buf: Vec::with_capacity(RECV_BUF),
        })
    }
}
