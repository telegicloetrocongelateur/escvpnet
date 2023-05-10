use std::{ pin::Pin, str::FromStr};

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncBufReadExt, BufReader, AsyncWriteExt};

use crate::{
    error::ErrorKind,
    io::{DecodeFrom, EncodeTo},
};
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Get { name: String },
    Set { name: String, value: String },
}

impl FromStr for Command {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_end();
        if s.contains("?") {
            let mut parts = s.split('?');
            let name = parts.next().ok_or(crate::Error::new(
                ErrorKind::Decoding,
                "Failed to decode command".to_string(),
            ))?;
            Ok(Self::Get {
                name: name.to_string(),
            })
        } else {
            let mut parts = s.split(' ');
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
#[async_trait]
impl<R: AsyncReadExt+Send> DecodeFrom<R> for Command {
    type Error = crate::Error;
    async fn decode_from(reader: &mut Pin<&mut R>) -> Result<Self, Self::Error> {
        let mut reader = BufReader::new(reader);
        let mut buf = String::new();
        reader.read_line(&mut buf).await?;
        buf.parse()

    }
}
#[async_trait]
impl<W: AsyncWriteExt+Send> EncodeTo<W> for Command {
    type Error = crate::Error;
    async fn encode_to(self, writer: &mut Pin<&mut W>) -> Result<usize, Self::Error> {
        let command = match self {
            Self::Get { name } => {
                format!("{name}?\n")
            }
            Self::Set { name, value } => {
                format!("{name} {value}\n")
            }
        };
        writer.write_all(command.as_bytes()).await?;
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
#[async_trait]
impl<R: AsyncReadExt+ Send> DecodeFrom<R> for Response {
    type Error = crate::Error;
    async fn decode_from(reader: &mut Pin<&mut R>) -> Result<Self, Self::Error> {
        let mut buf_reader = BufReader::new(reader);
        let mut buf = String::new();
        buf_reader.read_line(&mut buf).await?;
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
#[cfg(test)]
mod tests {
use super::*;
    #[test]
    fn command() {
        let data = "PWR?\n";
        let command = Command::Get { name: "PWR".to_string() };
        let decoded_command:Command = data.parse().unwrap();
        assert_eq!(decoded_command, command);

        let data = "PWR ON\n";
        let command = Command::Set { name: "PWR".to_string(), value: "ON".to_string() } ;
        let decoded_command:Command = data.parse().unwrap();
        assert_eq!(decoded_command, command)

    }
}