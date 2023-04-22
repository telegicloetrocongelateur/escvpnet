use std::pin::Pin;

use async_trait::async_trait;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

use crate::io::*;
pub const PROTOCOL_IDENTIFIER: [u8; 10] = *b"ESC/VP.net";

pub const VERSION_IDENTIFIER: u8 = 0x10;
use crate::header::*;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Packet {
    pub category: PacketCategory,
    pub status: Status,
    pub headers: Vec<Header>,
}

impl Packet {
    pub fn new(category: PacketCategory, status: Status, headers: Vec<Header>) -> Self {
        Self {
            category,
            status,
            headers,
        }
    }

    pub fn new_request(category: PacketCategory, headers: Vec<Header>) -> Self {
        Self {
            category,
            status: Status::Null,
            headers,
        }
    }

    pub fn status_as_result(self) -> Result<Self, crate::Error> {
        match self.status {
            Status::Ok | Status::Null => Ok(self),
            _ => Err(self.status.into()),
        }
    }

    pub fn headers(&self) -> &[Header] {
        self.headers.as_ref()
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn category(&self) -> &PacketCategory {
        &self.category
    }
}
#[async_trait]

impl<R: AsyncReadExt + Send> DecodeFrom<R> for Packet {
    type Error = crate::Error;
    async fn decode_from(reader: &mut Pin<&mut R>) -> Result<Self, Self::Error> {
        let protocol_identifier = <[u8; 10]>::decode_from(reader).await?;

        if protocol_identifier != PROTOCOL_IDENTIFIER {
            return Err(crate::Error::new(
                crate::error::ErrorKind::Decoding,
                "Bad protocol identifier".to_string(),
            ));
        }
        let version_identifier = u8::decode_from(reader).await?;

        if version_identifier != VERSION_IDENTIFIER {
            return Err(crate::Error::new(
                crate::error::ErrorKind::Decoding,
                "Bad protocol version".to_string(),
            ));
        }

        let category = PacketCategory::decode_from(reader).await?;

        <[u8; 2]>::decode_from(reader).await?; // Reserved

        let status = Status::decode_from(reader).await?;

        let headers = Vec::<Header>::decode_from(reader).await?;

        Ok(Self {
            category,
            status,
            headers,
        })
    }
}
#[async_trait]

impl<W: AsyncWriteExt + Send> EncodeTo<W> for Packet {
    type Error = crate::Error;
    async fn encode_to(self, writer: &mut Pin<&mut W>) -> Result<usize, Self::Error> {
        let mut len = 13;
        writer.write_all(&PROTOCOL_IDENTIFIER).await?;
        writer.write_all(&[VERSION_IDENTIFIER]).await?;
        len += self.category.encode_to(writer).await?;
        writer.write_all(&[0, 0]).await?;
        len += self.status.encode_to(writer).await?;
        len += self.headers.encode_to(writer).await?;
        Ok(len)
    }
}
#[derive(Debug, PartialEq, PartialOrd, Clone)]

pub enum PacketCategory {
    Null = 0, //Reserved
    Hello = 1,
    Password = 2,
    Connect = 3,
}

impl Length for PacketCategory {
    const LENGTH: usize = 1;
}

impl Decode for PacketCategory {
    type Error = crate::Error;
    fn decode(data: [u8; Self::LENGTH]) -> Result<Self, Self::Error> {
        use PacketCategory::*;

        match data[0] {
            0 => Ok(Null),
            1 => Ok(Hello),
            2 => Ok(Password),
            3 => Ok(Connect),
            _ => Err(crate::Error::new(
                crate::error::ErrorKind::Decoding,
                "Failed to decode a packet category".to_string(),
            )),
        }
    }
}
impl Encode for PacketCategory {
    type Error = crate::Error;
    fn encode(self) -> Result<[u8; Self::LENGTH], Self::Error> {
        Ok([self as u8])
    }
}
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Status {
    Null = 0x00, // For requests
    Ok = 0x20,
    BadRequest = 0x40,
    Unauthorized = 0x41,
    Forbidden = 0x43,
    RequestNotAllowed = 0x45,
    ServiceUnavailable = 0x53,
    VersionNotSupported = 0x55,
}

impl Length for Status {
    const LENGTH: usize = 1;
}

impl Decode for Status {
    type Error = crate::Error;
    fn decode(data: [u8; Self::LENGTH]) -> Result<Self, Self::Error> {
        match data[0] {
            0x00 => Ok(Self::Null),
            0x20 => Ok(Self::Ok),
            0x40 => Ok(Self::BadRequest),
            0x41 => Ok(Self::Unauthorized),
            0x43 => Ok(Self::Forbidden),
            0x45 => Ok(Self::RequestNotAllowed),
            0x53 => Ok(Self::ServiceUnavailable),
            0x55 => Ok(Self::VersionNotSupported),
            _ => Err(crate::Error::new(
                crate::error::ErrorKind::Decoding,
                "Failed to decode a status".to_string(),
            )),
        }
    }
}

impl Encode for Status {
    type Error = crate::Error;
    fn encode(self) -> Result<[u8; Self::LENGTH], Self::Error> {
        Ok([self as u8])
    }
}
#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    #[tokio::test]
    async fn packet() {
        let mut data = &b"ESC/VP.net\x10\x01\0\0\0\0"[..];
        let mut reader = Pin::new(&mut data);
        let decoded_packet = Packet::decode_from(&mut reader).await.unwrap();
        let packet = Packet::new_request(PacketCategory::Hello, vec![]);
        let mut buf = Vec::new();
        let mut encoded_packet = Pin::new(&mut buf);

        packet.clone().encode_to(&mut encoded_packet).await.unwrap();
        assert_eq!(decoded_packet, packet);
        assert_eq!(data, encoded_packet.as_slice());
    }
    #[tokio::test]
    async fn packet_headers() {
        let data = b"ESC/VP.net\x10\x02\0\0\0\x06\0\x050123456789abcdef\x01\x04123456789abcdef0\x02\x0323456789abcdef01\x03\x023456789abcdef012\x04\x01456789abcdef0123\x05\x0056789abcdef01234";
        let mut cursor = Cursor::new(data);
        let mut reader = Pin::new(&mut cursor);
        use HeaderIdentifier::*;

        let packet = Packet::new_request(
            PacketCategory::Password,
            vec![
                Header::new(Null, 5, "0123456789abcdef".to_string()).unwrap(),
                Header::new(Password, 4, "123456789abcdef0".to_string()).unwrap(),
                Header::new(NewPassword, 3, "23456789abcdef01".to_string()).unwrap(),
                Header::new(ProjectorName, 2, "3456789abcdef012".to_string()).unwrap(),
                Header::new(ImType, 1, "456789abcdef0123".to_string()).unwrap(),
                Header::new(ProjectorCommandType, 0, "56789abcdef01234".to_string()).unwrap(),
            ],
        );
        let decoded_packet = Packet::decode_from(&mut reader).await.unwrap();
        assert_eq!(decoded_packet, packet);
        let mut buf = vec![];
        let mut encoded_packet = Pin::new(&mut buf);
        packet.encode_to(&mut encoded_packet).await.unwrap();
        assert_eq!(encoded_packet.as_slice(), data)
    }
}
