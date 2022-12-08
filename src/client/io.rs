use crate::protocol::*;
use std::{
    io::{Cursor, Read, Write},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket},
};
const PROTOCOL_IDENTIFIER: [u8; 10] = *b"ESC/VP.net";

type Result<T> = std::result::Result<T, Error>;

pub trait ReadPacket {
    fn read_packet(&mut self) -> Result<Packet>;
}

impl<R: Read> ReadPacket for R {
    fn read_packet(&mut self) -> Result<Packet> {
        let mut buf = [0; 16];
        self.read_exact(&mut buf)?;

        if buf[..10] != b"ESC/VP.net"[..] {
            return Err(Error::ParseError);
        }

        let identifier = Identifier::try_from(buf[11])?;
        let status = Status::try_from(buf[14])?;

        let num_headers = buf[15] as usize;
        let headers = match num_headers {
            0 => None,
            _ => {
                let mut buf = [0; 18];

                let mut headers = Vec::with_capacity(num_headers);
                for _ in 0..num_headers {
                    self.read_exact(&mut buf)?;
                    println!("test");
                    headers.push(Header::try_from(buf)?);
                }
                Some(headers)
            }
        };

        Ok(Packet::new(identifier, status, headers))
    }
}

pub trait WritePacket {
    fn write_packet(&mut self, packet: Packet) -> Result<()>;
}

impl<W: Write> WritePacket for W {
    fn write_packet(&mut self, packet: Packet) -> Result<()> {
        self.write_all(&PROTOCOL_IDENTIFIER)?;
        self.write_all(&[
            0x10,
            packet.identifier.into(),
            0x00,
            0x00,
            packet.status.into(),
            match &packet.headers {
                Some(headers) => headers.len() as u8,
                None => 0,
            },
        ])?;

        if let Some(headers) = packet.headers {
            for header in headers {
                self.write_all(&(<[u8; 18]>::from(header)))?;
            }
        }
        self.flush()?;
        Ok(())
    }
}
pub trait ReceivePacket {
    fn recv_packet(&self) -> Result<Packet>;
    fn recv_packet_from(&self) -> Result<(Packet, SocketAddr)>;
}

impl ReceivePacket for UdpSocket {
    fn recv_packet_from(&self) -> Result<(Packet, SocketAddr)> {
        let mut buf = [0; 1024];
        let (_, addr) = self.recv_from(&mut buf).unwrap();

        if buf[..10] != b"ESC/VP.net"[..] {
            return Err(Error::ParseError);
        }
        println!("1");

        let identifier = Identifier::try_from(buf[11])?;
        let status = Status::try_from(buf[14])?;
        println!("2");

        let num_headers = buf[15] as usize;
        let mut headers = Vec::with_capacity(num_headers);

        for i in 0..num_headers {
            headers.push(Header::try_from(
                buf.get(16 + i * Header::SIZE..16 + (i + 1) * Header::SIZE)
                    .ok_or(Error::ParseError)?,
            )?)
        }

        Ok((Packet::new(identifier, status, Some(headers)), addr))
    }
    fn recv_packet(&self) -> Result<Packet> {
        let mut buf = [0; 1024];
        self.recv(&mut buf)?;

        if buf[..10] != b"ESC/VP.net"[..] {
            return Err(Error::ParseError);
        }

        let identifier = Identifier::try_from(buf[11])?;
        let status = Status::try_from(buf[14])?;

        let num_headers = buf[14] as usize;
        let mut headers = Vec::with_capacity(num_headers);

        for i in 0..num_headers {
            headers.push(Header::try_from(
                buf.get(16 + i * Header::SIZE..16 + (i + 1) * Header::SIZE)
                    .ok_or(Error::ParseError)?,
            )?)
        }

        Ok(Packet::new(identifier, status, Some(headers)))
    }
}

pub trait SendPacket {
    fn send_packet_to<A: ToSocketAddrs>(&self, packet: Packet, addr: A) -> Result<()>;
    fn send_packet(&self, packet: Packet) -> Result<()>;
}

impl SendPacket for UdpSocket {
    fn send_packet_to<A: ToSocketAddrs>(&self, packet: Packet, addr: A) -> Result<()> {
        self.send_to(&Vec::<u8>::from(packet), addr)?;
        Ok(())
    }

    fn send_packet(&self, packet: Packet) -> Result<()> {
        self.send(&Vec::<u8>::from(packet))?;
        Ok(())
    }
}

#[test]
fn read_packet() {
    let mut data = Cursor::new(b"ESC/VP.net\x10\x01\0\0\0\0".to_vec());
    assert_eq!(
        data.read_packet().unwrap(),
        Packet::new(Identifier::Hello, Status::Null, None)
    );
    let mut data = Cursor::new(b"ESC/VP.net\x10\x01\0\0\0\x05\x01\x010123456789abcdef\x02\x010123456789abcdef\x03\x00\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x04\x00\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x05\x21\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0".to_vec());
    assert_eq!(
        data.read_packet().unwrap(),
        Packet::new(
            Identifier::Hello,
            Status::Null,
            Some(vec![
                Header::Password(Some("0123456789abcdef".to_string())),
                Header::NewPassword(Some("0123456789abcdef".to_string())),
                Header::ProjectorName(None),
                Header::IMType(0),
                Header::ProjectorCommandType,
            ])
        )
    );
}
#[test]
fn write_packet() {
    let mut data = Cursor::new(Vec::<u8>::new());
    data.write_packet(Packet::new(
        Identifier::Hello,
        Status::Null,
        Some(vec![
            Header::Password(Some("0123456789abcdef".to_string())),
            Header::NewPassword(Some("0123456789abcdef".to_string())),
            Header::ProjectorName(None),
            Header::IMType(0),
            Header::ProjectorCommandType,
        ]),
    ))
    .unwrap();

    assert_eq!(data.into_inner(), b"ESC/VP.net\x10\x01\0\0\0\x05\x01\x010123456789abcdef\x02\x010123456789abcdef\x03\x00\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x04\x00\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x05\x21\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0".to_vec())
}
#[test]
fn recv_write_packet() {
    let mut socket = UdpSocket::bind("127.0.0.1:58478").unwrap();
    /*
    let mut buf = [0; 1024];
    let (_, addr) = socket.recv_from(&mut buf).unwrap();
    */

    let packet1 = Packet::new(
        Identifier::Hello,
        Status::Null,
        Some(vec![
            Header::Password(Some("0123456789abcdef".to_string())),
            Header::NewPassword(Some("0123456789abcdef".to_string())),
            Header::ProjectorName(None),
            Header::IMType(0),
            Header::ProjectorCommandType,
        ]),
    );
    socket.set_broadcast(true);
    socket
        .send_packet_to(packet1.clone(), "255.255.255.255:58478")
        .unwrap();

    let (packet2, addr) = socket.recv_packet_from().unwrap();

    assert_eq!(packet1, packet2);
    assert_eq!(
        addr,
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 58478))
    )
}
