use std::{
    io::Read,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

use crate::Error;
type Result<T> = std::result::Result<T, Error>;

enum Command<'a> {
    Set {
        command: &'a str,
        parameter: &'a str,
    },
    Get {
        command: &'a str,
    },
    Null,
}

struct Response {
    command: String,
    parameter: String,
}

trait SendCommand {
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

        Ok(Response {
            command: command.to_string(),
            parameter: parameter.to_string(),
        })
    }
}
