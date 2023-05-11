use std::{
    pin::Pin,
};

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{error::ErrorKind, io::*, Result};
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Header {
    identifier: HeaderIdentifier,
    attribute: u8,
    information: String,
}

impl Header {
    pub fn new(identifier: HeaderIdentifier, attribute: u8, information: String) -> Result<Self> {
        if information.len() > 16 {
            return Err(crate::Error::new(
                ErrorKind::Encoding,
                "Header length is too big".to_string(),
            ));
        }
        Ok(Self {
            identifier,
            attribute,
            information,
        })
    }

    pub fn identifier(&self) -> &HeaderIdentifier {
        &self.identifier
    }

    pub fn attribute(&self) -> u8 {
        self.attribute
    }

    pub fn information(&self) -> &str {
        self.information.as_ref()
    }
}

impl Length for Header {
    const LENGTH: usize = 18;
}

impl Decode for Header {
    type Error = crate::Error;
    fn decode(data: [u8; Self::LENGTH]) -> Result<Self> {
        let identifier = HeaderIdentifier::decode(data[0..1].try_into().unwrap())?;
        let attribute = data[1];
        let information = String::from_utf8(data[2..].to_vec())?;
        Ok(Self {
            identifier,
            attribute,
            information,
        })
    }
}
#[async_trait]
impl<R: AsyncReadExt + Send> DecodeFrom<R> for Vec<Header> {
    type Error = crate::Error;
    async fn decode_from(reader: &mut Pin<&mut R>) -> Result<Self> {
        let len = u8::decode_from(reader).await? as usize;
        let mut packet_categories = Vec::with_capacity(len);
        for _ in 0..len {
            packet_categories.push(Header::decode_from(reader).await?)
        }
        Ok(packet_categories)
    }
}
#[async_trait]

impl<W: AsyncWriteExt + Send> EncodeTo<W> for Vec<Header> {
    type Error = crate::Error;
    async fn encode_to(self, writer: &mut Pin<&mut W>) -> Result<usize> {
        let len: u8 = self.len().try_into()?;
        len.encode_to(writer).await?;

        for header in self {
            header.encode_to(writer).await?;
        }
        Ok(1 + len as usize * Header::LENGTH)
    }
}

impl Encode for Header {
    type Error = crate::Error;
    fn encode(self) -> Result<[u8; Self::LENGTH]> {
        let mut data = [0; Self::LENGTH];
        data[0] = self.identifier.encode()?[0];
        data[1] = self.attribute;
        data[2..Self::LENGTH].copy_from_slice(self.information.as_bytes());
        Ok(data)
    }
}
#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum HeaderIdentifier {
    Null = 0, // Reserved
    Password = 1,
    NewPassword = 2,
    ProjectorName = 3,
    ImType = 4,
    ProjectorCommandType = 5,
}

impl Length for HeaderIdentifier {
    const LENGTH: usize = 1;
}

impl Decode for HeaderIdentifier {
    type Error = crate::Error;
    fn decode(data: [u8; Self::LENGTH]) -> Result<Self> {
        use HeaderIdentifier::*;
        match data[0] {
            0 => Ok(Null), // Reserved,
            1 => Ok(Password),
            2 => Ok(NewPassword),
            3 => Ok(ProjectorName),
            4 => Ok(ImType),
            5 => Ok(ProjectorCommandType),
            _ => Err(crate::Error::new(
                crate::error::ErrorKind::Decoding,
                "Failed to decode a header identifier".to_string(),
            )),
        }
    }
}

impl Encode for HeaderIdentifier {
    type Error = crate::Error;
    fn encode(self) -> Result<[u8; Self::LENGTH]> {
        Ok([self as u8])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header() {
        let data = *b"\0\00123456789abcdef";
        let header = Header {
            identifier: HeaderIdentifier::Null,
            attribute: 0,
            information: "0123456789abcdef".to_string(),
        };
        assert_eq!(header, Header::decode(data).unwrap())
    }

    #[test]
    fn header_identifier() {
        let order = [
            HeaderIdentifier::Null, // Reserved
            HeaderIdentifier::Password,
            HeaderIdentifier::NewPassword,
            HeaderIdentifier::ProjectorName,
            HeaderIdentifier::ImType,
            HeaderIdentifier::ProjectorCommandType,
        ];
        for (i, hd) in order.iter().enumerate() {
            assert_eq!(*hd, HeaderIdentifier::decode([i as u8]).unwrap())
        }
    }
}
