use super::errors::{Error, ProtocolError};
#[derive(Debug, PartialEq, Eq)]

pub enum Command<'a> {
    Set {
        command: &'a str,
        parameter: Parameter,
    },
    Get {
        command: &'a str,
    },
    Null,
}

impl<'a> TryFrom<&'a str> for Command<'a> {
    type Error = Error;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Ok(Self::Null);
        }
        if value.chars().nth(value.len() - 1).ok_or(Error::Parse)? == '?' {
            return Ok(Self::Get {
                command: &value[..value.len() - 1],
            });
        }
        if value.contains(' ') {
            let mut data = value.split(' ');
            let command = data.next().ok_or(Error::Parse)?;
            let parameter = data.next().ok_or(Error::Parse)?;
            return Ok(Self::Set {
                command,
                parameter: Parameter::try_from(parameter)?,
            });
        }
        Err(Error::Parse)
    }
}

impl<'a> From<Command<'a>> for Vec<u8> {
    fn from(command: Command<'a>) -> Self {
        match command {
            Command::Set { command, parameter } => {
                let mut bytes: Vec<u8> = Vec::new();
                bytes.extend(command.as_bytes());
                bytes.push(b' ');
                bytes.extend(Into::<Vec<u8>>::into(parameter));
                bytes.push(b'\r');
                bytes
            }
            Command::Get { command } => {
                let mut bytes: Vec<u8> = Vec::new();
                bytes.extend(command.as_bytes());
                bytes.extend(b"?\r");
                bytes
            }
            Command::Null => b"\x0d".to_vec(),
        }
    }
}
impl<'a> From<&Command<'a>> for Vec<u8> {
    fn from(command: &Command<'a>) -> Self {
        match command {
            Command::Set { command, parameter } => {
                let mut bytes: Vec<u8> = Vec::new();
                bytes.extend(command.as_bytes());
                bytes.push(b' ');
                bytes.extend(Into::<Vec<u8>>::into(parameter));
                bytes.push(b'\r');
                bytes
            }
            Command::Get { command } => {
                let mut bytes: Vec<u8> = Vec::new();
                bytes.extend(command.as_bytes());
                bytes.extend(b"?\r");
                bytes
            }
            Command::Null => b"\x0d".to_vec(),
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum Parameter {
    On,
    Off,
    Number(u32),
    Increase,
    Decrease,
    Initialize,
}
impl<'a> TryFrom<&'a str> for Parameter {
    type Error = Error;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if let Ok(number) = value.parse::<u32>() {
            return Ok(Self::Number(number));
        }
        match value {
            "ON" => Ok(Self::On),
            "OFF" => Ok(Self::Off),
            "INC" => Ok(Self::Increase),
            "DEC" => Ok(Self::Decrease),
            "INIT" => Ok(Self::Initialize),

            _ => Err(Error::Parse),
        }
    }
}

impl From<&Parameter> for Vec<u8> {
    fn from(parameter: &Parameter) -> Self {
        match parameter {
            Parameter::On => b"ON".to_vec(),
            Parameter::Off => b"OFF".to_vec(),
            Parameter::Number(n) => n.to_string().as_bytes().to_vec(),
            Parameter::Increase => b"INC".to_vec(),
            Parameter::Decrease => b"DEC".to_vec(),
            Parameter::Initialize => b"INIT".to_vec(),
        }
    }
}
impl From<Parameter> for Vec<u8> {
    fn from(parameter: Parameter) -> Self {
        match parameter {
            Parameter::On => b"ON".to_vec(),
            Parameter::Off => b"OFF".to_vec(),
            Parameter::Number(n) => n.to_string().as_bytes().to_vec(),
            Parameter::Increase => b"INC".to_vec(),
            Parameter::Decrease => b"DEC".to_vec(),
            Parameter::Initialize => b"INIT".to_vec(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]

pub struct Response {
    pub command: String,
    pub parameter: String,
}

impl Response {
    fn new(command: String, parameter: String) -> Self {
        Self { command, parameter }
    }
}

impl TryFrom<&[u8]> for Response {
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value == b"ERR\x0d:" {
            return Err(Error::Protocol(ProtocolError::BadCommand));
        }
        let value = String::from_utf8_lossy(value).to_string();
        let mut parts = value.split('=');

        let command = parts.next().ok_or(ProtocolError::BadResponse)?.to_string();
        let parameter = parts
            .next()
            .ok_or(ProtocolError::BadResponse)?
            .replace(':', "")
            .trim_end()
            .to_string();
        Ok(Response {
            command,

            parameter, //TODO: optimize it
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::protocol::{Command, Response};

    use super::Parameter;
    #[test]
    fn parse_response() {
        assert_eq!(
            Response::try_from(&b"SOURCE=21:"[..]).unwrap(),
            Response {
                command: "SOURCE".to_string(),
                parameter: "21".to_string()
            }
        );
        //Response::try_from(&b"ERR\x0d:"[..]).unwrap();
    }
    #[test]
    fn parse_command() {
        assert_eq!(
            Command::try_from("SOURCE 21").unwrap(),
            Command::Set {
                command: "SOURCE",
                parameter: Parameter::Number(21)
            }
        );
        assert_eq!(
            Command::try_from("SOURCE?").unwrap(),
            Command::Get { command: "SOURCE" }
        );

        assert_eq!(Command::try_from("").unwrap(), Command::Null)
    }
    #[test]
    fn command_to_bytes() {
        let data: Vec<u8> = Command::Set {
            command: "SOURCE",
            parameter: Parameter::Number(21),
        }
        .into();
        assert_eq!(&data, b"SOURCE 21\r");
        let data: Vec<u8> = Command::Get { command: "SOURCE" }.into();
        assert_eq!(&data, b"SOURCE?\r");
        let data: Vec<u8> = Command::Null.into();
        assert_eq!(&data, b"\x0d");
    }

    #[test]
    fn parameter_to_bytes() {
        let data: Vec<u8> = Parameter::On.into();
        assert_eq!(data, b"ON".to_vec());
        let data: Vec<u8> = Parameter::Number(12).into();
        assert_eq!(data, b"12".to_vec());
    }
}
