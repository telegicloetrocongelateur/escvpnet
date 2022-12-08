use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
};

use crate::{
    packet::PROTOCOL_IDENTIFIER, Command, Error, Header, Identifier, Packet, Response, Status,
};
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

pub trait SendCommand {
    fn send_command(&mut self, command: Command) -> Result<Response>;
}

impl<T: Read + Write> SendCommand for T {
    fn send_command(&mut self, command: Command) -> Result<Response> {
        match command {
            Command::Set { command, parameter } => {
                self.write_all(command.as_bytes())?;
                self.write_all(b" ")?;
                self.write_all(parameter.as_bytes())?;
            }
            Command::Get { command } => {
                self.write_all(command.as_bytes())?;
                self.write_all(b"?")?;
            }
            Command::Null => {}
        }
        self.write_all(b"\r")?;
        self.flush()?;

        let mut reader = BufReader::new(self);
        let mut buf = Vec::new();
        reader.read_until(b':', &mut buf)?;
        let data = String::from_utf8_lossy(&buf);
        let mut parts = data.split('=');
        let command = parts.next().ok_or(Error::ParseError)?;
        let parameter = parts.next().ok_or(Error::ParseError)?;

        Ok(Response::new(command.to_string(), parameter.to_string()))
    }
}
