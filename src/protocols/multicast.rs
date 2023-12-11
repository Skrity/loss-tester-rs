use anyhow::{Error, Result};
use net2::UdpBuilder;
use std::net::{Ipv4Addr, UdpSocket};

use super::{Receiver, Sender};

pub struct MulticastSender {
    socket: UdpSocket,
}

impl MulticastSender {
    pub fn new(grp: Ipv4Addr, port: u16, bind: Ipv4Addr) -> Result<Self> {
        if !grp.is_multicast() {
            return Err(Error::msg(format!("not a multicast address: {grp}")));
        }
        let socket = UdpBuilder::new_v4()?;
        // https://stackoverflow.com/questions/14388706/how-do-so-reuseaddr-and-so-reuseport-differ/14388707#14388707
        socket.reuse_address(true)?;
        let socket = socket.bind((bind, 0))?;
        socket.set_multicast_ttl_v4(1)?;
        socket.connect((grp, port))?;
        Ok(Self { socket })
    }
}

impl Sender for MulticastSender {
    fn send(&mut self, data: &[u8]) -> Result<()> {
        self.socket.send(data)?;
        Ok(())
    }
}

pub struct MulticastReceiver {
    socket: UdpSocket,
}

impl Receiver for MulticastReceiver {
    fn recv<'a>(&mut self, buf: &'a mut [u8]) -> Result<&'a [u8]> {
        let (size, _addr) = self.socket.recv_from(buf)?;
        return Ok(&buf[..size]);
    }
}

impl MulticastReceiver {
    pub fn new(grp: Ipv4Addr, port: u16, bind: Ipv4Addr) -> Result<Self> {
        if !grp.is_multicast() {
            return Err(Error::msg(format!("not a multicast address: {grp}")));
        }
        let socket = UdpBuilder::new_v4()?;
        socket.reuse_address(true)?;
        let socket = socket.bind((bind, port))?;
        socket.join_multicast_v4(&grp, &bind)?;
        Ok(Self { socket })
    }
}
