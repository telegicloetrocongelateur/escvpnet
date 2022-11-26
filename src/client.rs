#![allow(dead_code)]

use super::errors::*;
use super::protocol::*;
use std::io::{BufRead, BufReader};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::net::ToSocketAddrs;
const BUF_SIZE: usize = 1024;

/// ESC/VP.net client
/// After creating a client, ESC/VP.net commands ([`Command`]) can be sent and response ([`Response`]) received.
/// The communication session will terminate 10 minutes after the last sent command, to prevent that the keep_alive() can be used.
/// # Example
///
/// ```
/// use escvpnet::{Client, commands::LAMP, Command};
/// let mut client = Client::connect("192.168.0.1:3629").expect("Failed to connect to projector");
/// let command = Command::Get { command: LAMP };
/// client.send(command).expect("Failed to send command");
///
/// ```
pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self, Error> {
        let mut stream = TcpStream::connect(addr)?;
        stream.write_all(b"ESC/VP.net\x10\x03\x00\x00\x00\x00")?; // protocol header
        let mut buf = [0; 16];
        stream.read_exact(&mut buf)?;
        if buf[0..10] != b"ESC/VP.net"[..] {
            return Err(ProtocolError::WrongProtocol.into());
        }
        match buf[14] {
            0x20 => Ok(Self { stream }),
            0x41 => Err(ProtocolError::PasswordRequired.into()),
            0x43 => Err(ProtocolError::WrongPassword.into()),
            0x53 => Err(ProtocolError::Busy.into()),
            _ => Err(ProtocolError::BadResponse.into()),
        }
    }

    pub fn send(&mut self, command: Command) -> Result<Response, Error> {
        self.stream.write_all(&Into::<Vec<u8>>::into(command))?;

        let mut buf = Vec::new();
        let mut reader = BufReader::new(&self.stream);
        reader.read_until(b':', &mut buf)?;
        Response::try_from(&buf[..])
    }

    pub fn keep_alive(&mut self) -> Result<(), Error> {
        self.stream
            .write_all(&Into::<Vec<u8>>::into(Command::Null))?;
        Ok(())
    }
}
