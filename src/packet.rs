use super::{Error, Header, Identifier, Status};
use std::io::Read;
use std::net::{SocketAddr, UdpSocket};

pub const PROTOCOL_IDENTIFIER: [u8; 10] = *b"ESC/VP.net";

type Result<T> = std::result::Result<T, super::Error>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Packet {
    pub identifier: Identifier,
    pub status: Status,

    pub headers: Option<Vec<Header>>,
}

impl Packet {
    pub fn new(identifier: Identifier, status: Status, headers: Option<Vec<Header>>) -> Self {
        Self {
            identifier,
            status,
            headers,
        }
    }
    pub fn status(&self) -> Status {
        self.status.clone()
    }
}

impl From<Packet> for Vec<u8> {
    fn from(request: Packet) -> Self {
        let mut data = Vec::with_capacity(16);
        data.extend_from_slice(&PROTOCOL_IDENTIFIER);
        data.push(0x10);
        data.push(request.identifier.into());
        data.extend_from_slice(&[0, 0]);
        data.push(request.status.into());
        data.push(match request.headers {
            None => 0,
            Some(ref headers) => headers.len() as u8,
        });
        if let Some(test) = request.headers {
            for header in test {
                data.extend_from_slice(&std::convert::Into::<[u8; 18]>::into(header))
            }
        }
        data
    }
}

impl TryFrom<[u8; 16]> for Packet {
    type Error = Error;
    fn try_from(value: [u8; 16]) -> Result<Self> {
        Ok(Self {
            identifier: Identifier::try_from(value[11])?,
            status: Status::try_from(value[14])?,
            headers: match value.get(15).ok_or(Error::ParseError)? {
                0 => None,
                n => Some(Vec::with_capacity(*n as usize)),
            },
        })
    }
}

impl TryFrom<&[u8]> for Packet {
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self> {
        if value[0..10] != PROTOCOL_IDENTIFIER {
            return Err(Self::Error::ParseError);
        }

        let identifier = Identifier::try_from(value[11])?;
        let status = Status::try_from(value[14])?;
        let num_headers = value[15] as usize;
        let headers: Option<Vec<Header>> = match num_headers {
            0 => None,
            _ => {
                let mut headers = Vec::with_capacity(num_headers);
                for i in 0..num_headers {
                    let data = &value[16 + i * 18..16 + (i + 1) * 18];
                    println!("{:?}", data);
                    headers.push(Header::try_from(data)?);
                }
                Some(headers)
            }
        };
        Ok(Self {
            identifier,
            status,
            headers,
        })
    }
}

pub const HELLO: [u8; 16] = *b"ESC/VP.net\x10\x01\x00\x00\x00\x00";

#[test]
fn packet_from_bytes() {
    let data =
        b"ESC/VP.net\x10\x01\x00\x00\x20\x01\x03\x01Room 1\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";

    assert_eq!(
        Packet::try_from(&data[..]).unwrap(),
        Packet {
            identifier: Identifier::Hello,
            status: Status::Ok,
            headers: Some(vec![Header::ProjectorName(Some(
                "Room 1\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_string()
            ))])
        }
    );

    let data = *b"ESC/VP.net\x10\x01\x00\x00\x00\x00";
    assert_eq!(
        Packet::try_from(data).unwrap(),
        Packet {
            identifier: Identifier::Hello,
            status: Status::Null,
            headers: None
        }
    )
}
pub trait ReadPacket {
    fn read_packet(&mut self) -> Result<Packet>;
}

impl<R: Read> ReadPacket for R {
    fn read_packet(&mut self) -> Result<Packet> {
        let mut buf = [0; 16];

        self.read_exact(&mut buf)?;
        let identifier = Identifier::try_from(buf[11])?;
        let status = Status::try_from(buf[14])?;
        let num_headers = buf[15] as usize;
        Ok(Packet {
            identifier,
            status,
            headers: match num_headers {
                0 => None,
                _ => {
                    let mut headers = Vec::with_capacity(num_headers);
                    for _ in 0..num_headers {
                        let mut header = [0; 18];
                        self.read_exact(&mut header)?;
                        headers.push(Header::try_from(header)?);
                    }
                    Some(headers)
                }
            },
        })
    }
}
pub trait ReceivePacket {
    fn recv_packet_from(&self) -> Result<(Packet, SocketAddr)>;
    fn recv_packet(&self) -> Result<Packet>;
}

const BUF_SIZE: usize = 512;

impl ReceivePacket for UdpSocket {
    fn recv_packet_from(&self) -> Result<(Packet, SocketAddr)> {
        let mut buf = [0; BUF_SIZE];
        let (_, addr) = self.recv_from(&mut buf)?;

        let identifier = Identifier::try_from(buf[11])?;
        let status = Status::try_from(buf[14])?;
        let num_headers = buf[15];
        Ok((
            Packet {
                identifier,
                status,
                headers: match num_headers {
                    0 => None,
                    _ => {
                        let mut headers = Vec::with_capacity(num_headers as usize);
                        for data in buf[16..].chunks_exact(18) {
                            headers.push(Header::try_from(data)?);
                        }
                        Some(headers)
                    }
                },
            },
            addr,
        ))
    }

    fn recv_packet(&self) -> Result<Packet> {
        let mut buf = [0; BUF_SIZE];
        self.recv(&mut buf)?;
        let identifier = Identifier::try_from(buf[11])?;
        let status = Status::try_from(buf[14])?;
        let num_headers = buf[15];
        Ok(Packet {
            identifier,
            status,
            headers: match num_headers {
                0 => None,
                _ => {
                    let mut headers = Vec::with_capacity(num_headers as usize);
                    for data in buf[16..].chunks_exact(18) {
                        headers.push(Header::try_from(data)?);
                    }
                    Some(headers)
                }
            },
        })
    }
}
