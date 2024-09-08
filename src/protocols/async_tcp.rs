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
    buf: Vec<u8>,
}

pub struct TcpReceiver {
    socket: TcpListener,
    connection: VecDeque<TcpClient>, // Can support multiple clients; but currently only accepts one
    buf: Vec<u8>,
}

impl Receiver for TcpReceiver {
    async fn recv(&mut self) -> Result<&[u8], ProtoError> {
        self.buf.clear();
        loop {
            // This loop polls all TcpClients which we accepted, first done will be yielded
            // At high throughput this will skew towards first client
            for (
                index,
                TcpClient {
                    stream: ref mut conn,
                    addr,
                    buf,
                },
            ) in self.connection.iter_mut().enumerate()
            {
                match poll_once(conn.read_until(0, buf)).await {
                    None => {} // try another
                    Some(Ok(_)) if (!buf.is_empty() && buf[0] != 0) => {
                        std::mem::swap(&mut self.buf, buf);
                        return Ok(&self.buf[..]);
                    }
                    _ => {
                        let addr = *addr;
                        self.connection.remove(index);
                        return Err(ProtoError::Disconnected(addr));
                    }
                }
            }
            // Currently we accepts only one client
            // Somehow this helps with bandwidth
            if self.connection.is_empty() {
                if let Some(Ok((conn, addr))) = poll_once(self.socket.accept()).await {
                    self.connection.push_back(TcpClient {
                        stream: BufReader::new(conn),
                        addr,
                        buf: vec![],
                    });
                    return Err(ProtoError::Connected(addr));
                }
            }
            // if no client was ready or no connection was accepted - yield
            yield_now().await;
        }
    }
}

impl TcpReceiver {
    pub fn new(bind: Ipv4Addr, port: u16) -> anyhow::Result<Self> {
        let listener = smol::block_on(TcpListener::bind((bind, port)))?;

        Ok(Self {
            socket: listener,
            connection: VecDeque::new(),
            buf: Vec::with_capacity(RECV_BUF),
        })
    }
}
