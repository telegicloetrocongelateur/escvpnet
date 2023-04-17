use std::io::{BufRead, BufReader, Read, Write};

use crate::{
    error::ErrorKind,
    io::{DecodeFrom, EncodeTo},
};

pub enum Command {
    Get { name: String },
    Set { name: String, value: String },
}

impl<R: Read> DecodeFrom<R> for Command {
    type Error = crate::Error;
    fn decode_from(reader: &mut R) -> Result<Self, Self::Error> {
        let mut reader = BufReader::new(reader);
        let mut buf = String::new();
        reader.read_line(&mut buf)?;

        if buf.contains("?") {
            let mut parts = buf.split('?');
            let name = parts.next().ok_or(crate::Error::new(
                ErrorKind::Decoding,
                "Failed to decode command".to_string(),
            ))?;
            Ok(Self::Get {
                name: name.to_string(),
            })
        } else {
            let mut parts = buf.split(' ');
            let name = parts.next().ok_or(crate::Error::new(
                ErrorKind::Decoding,
                "Failed to decode command".to_string(),
            ))?;
            let value = parts.next().ok_or(crate::Error::new(
                ErrorKind::Decoding,
                "Failed to decode command".to_string(),
            ))?;
            Ok(Self::Set {
                name: name.to_string(),
                value: value.to_string(),
            })
        }
    }
}

impl<W: Write> EncodeTo<W> for Command {
    type Error = crate::Error;
    fn encode_to(self, writer: &mut W) -> Result<usize, Self::Error> {
        let command = match self {
            Self::Get { name } => {
                format!("{name}?\n")
            }
            Self::Set { name, value } => {
                format!("{name} {value}\n")
            }
        };
        writer.write_all(command.as_bytes())?;
        Ok(command.len())
    }
}

pub struct Response {
    name: String,
    value: String,
}

impl Response {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn value(&self) -> &str {
        self.value.as_ref()
    }
}

impl<R: Read> DecodeFrom<R> for Response {
    type Error = crate::Error;
    fn decode_from(reader: &mut R) -> Result<Self, Self::Error> {
        let mut buf_reader = BufReader::new(reader);
        let mut buf = String::new();
        buf_reader.read_line(&mut buf)?;
        let mut parts = buf.split("=");
        let name = parts
            .next()
            .ok_or(crate::Error::new(
                ErrorKind::Decoding,
                "Failed to decode response".to_string(),
            ))?
            .to_string();
        let value = parts
            .next()
            .ok_or(crate::Error::new(
                ErrorKind::Decoding,
                "Failed to decode response".to_string(),
            ))?
            .to_string();

        Ok(Self { name, value })
    }
}
