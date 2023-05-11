use std::{net::SocketAddr, pin::Pin, time::Duration};

use crate::{
    header::{Header, HeaderIdentifier},
    io::{DecodeFrom, EncodeTo},
    packet::{Packet, PacketCategory, Status},
    Result,
};
use tokio::{
    io::AsyncWriteExt,
    io::{BufReader, BufWriter},
    net::{TcpStream, ToSocketAddrs, UdpSocket},
};

const HELLO_PACKET: [u8; 16] = [
    b'E', b'S', b'C', b'/', b'V', b'P', b'.', b'n', b'e', b't', // Protocol Header
    0x10, // Protocol version
    1,    // Type identifier
    0, 0, // Reserved
    0, //Status code
    0, // Number of headers
];
const BUF_SIZE: usize = 1024;

const CONNECT_PACKET: Packet = Packet {
    category: PacketCategory::Connect,
    status: Status::Null,
    headers: vec![],
};

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub async fn discover<A: ToSocketAddrs>(
        bind_addr: A,
        broadcast_addr: A,
        timeout: Duration,
    ) -> Result<Vec<Projector>> {
        let socket = UdpSocket::bind(bind_addr).await?;

        socket.set_broadcast(true)?;

        socket.send_to(&HELLO_PACKET, broadcast_addr).await?;
        let mut projectors = Vec::new();
        let mut buf = [0; BUF_SIZE];

        while let Ok((n, addr)) =
            match tokio::time::timeout(timeout, socket.recv_from(&mut buf)).await {
                Ok(result) => result,
                Err(_) => return Ok(projectors),
            }
        {
            let packet = Packet::decode_from(&mut Pin::new(&mut &buf[..n])).await?; // handle result
            let name = packet
                .headers
                .iter()
                .find(|h| matches!(h.identifier(), HeaderIdentifier::ProjectorName))
                .map(|h| h.information().to_string());
            projectors.push(Projector { addr, name })
        }
        Ok(projectors)
    }

    pub async fn connect<A: ToSocketAddrs>(
        addr: A,
        password: Option<String>,
        timeout: Duration,
    ) -> Result<Self> {
        let mut stream = tokio::time::timeout(timeout, TcpStream::connect(addr))
            .await
            .map_err(|_| {
                crate::Error::new(
                    crate::error::ErrorKind::IO(std::io::ErrorKind::TimedOut),
                    "Timed out".to_string(),
                )
            })??;
        {
            let (reader, writer) = stream.split();
            let mut writer = BufWriter::new(writer);
            let mut pinned_writer = Pin::new(&mut writer);
            let mut reader = BufReader::new(reader);
            let mut pinned_reader = Pin::new(&mut reader);

            let packet = match password {
                None => CONNECT_PACKET,
                Some(password) => {
                    let mut packet = CONNECT_PACKET;
                    packet
                        .headers
                        .push(Header::new(HeaderIdentifier::Password, 1, password)?);
                    packet
                }
            };

            packet.encode_to(&mut pinned_writer).await?;
            writer.flush().await?;
            Packet::decode_from(&mut pinned_reader)
                .await?
                .status_as_result()?;
        }
        Ok(Self { stream })
    }

    pub async fn send_packet(&mut self, packet: Packet) -> Result<Packet> {
        let (reader, writer) = self.stream.split();
        let mut writer = BufWriter::new(writer);
        let mut pinned_writer = Pin::new(&mut writer);
        let mut reader = BufReader::new(reader);
        let mut pinned_reader = Pin::new(&mut reader);
        packet.encode_to(&mut pinned_writer).await?;

        Packet::decode_from(&mut pinned_reader).await
    }
}

pub struct Projector {
    addr: SocketAddr,
    name: Option<String>,
}

impl Projector {
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }
}
