use std::{
    io::Cursor,
    net::{SocketAddr, TcpStream, ToSocketAddrs, UdpSocket},
    time::Duration,
};

use crate::{
    header::{Header, HeaderIdentifier},
    io::{DecodeFrom, EncodeTo},
    packet::{Packet, PacketCategory, Status},
    Result,
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
const KEEP_ALIVE_PACKET: Packet = Packet {
    category: PacketCategory::Hello,
    status: Status::Null,
    headers: vec![],
};

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn discover<A: ToSocketAddrs>(
        bind_addr: A,
        broadcast_addr: A,
        timeout: Option<Duration>,
    ) -> Result<Vec<Projector>> {
        let socket = UdpSocket::bind(bind_addr)?;

        socket.set_read_timeout(timeout)?;
        socket.set_write_timeout(timeout)?;
        socket.set_broadcast(true)?;

        socket.send_to(&HELLO_PACKET, broadcast_addr)?;
        let mut projectors = Vec::new();
        let mut buf = [0; BUF_SIZE];
        while let Ok((n, addr)) = socket.recv_from(&mut buf) {
            let packet = Packet::decode_from(&mut Cursor::new(&buf[..n]))?; // handle result
            let name = packet
                .headers
                .iter()
                .find(|h| matches!(h.identifier(), HeaderIdentifier::ProjectorName))
                .map(|h| h.information().to_string());
            projectors.push(Projector { addr, name })
        }
        Ok(projectors)
    }

    pub fn connect<A: ToSocketAddrs>(addr: A, password: Option<String>) -> Result<Self> {
        let mut stream = TcpStream::connect(addr)?;
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

        packet.encode_to(&mut stream)?;

        Packet::decode_from(&mut stream)?.status_as_result()?;
        Ok(Self { stream })
    }

    pub fn keep_alive(&mut self) -> Result<()> {
        KEEP_ALIVE_PACKET.encode_to(&mut self.stream)?;
        Packet::decode_from(&mut self.stream)?.status_as_result()?;
        Ok(())
    }

    pub fn send_packet(&mut self, packet: Packet) -> Result<Packet> {
        packet.encode_to(&mut self.stream)?;
        Packet::decode_from(&mut self.stream)
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

impl ToSocketAddrs for Projector {
    type Iter = std::option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        Ok(Some(self.addr).into_iter())
    }
}
