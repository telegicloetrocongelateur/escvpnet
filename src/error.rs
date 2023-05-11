use std::{num::TryFromIntError, string::FromUtf8Error};

use crate::packet::Status;

#[derive(Debug)]

pub struct Error {
    kind: ErrorKind,
    message: String,
}
impl Error {
    pub fn new(kind: ErrorKind, message: String) -> Self {
        Self { kind, message }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind.clone()
    }

    pub fn message(&self) -> String {
        self.message.clone()
    }
}
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum ErrorKind {
    Decoding,
    Encoding,
    IO(std::io::ErrorKind),
    Protocol(Status),
}

impl ToString for ErrorKind {
    fn to_string(&self) -> String {
        use ErrorKind::*;
        match self {
            Decoding => "Decoding Error".to_string(),
            Encoding => "Encoding Error".to_string(),
            IO(kind) => format!("I/O Error ({})", kind.to_string()),
            Protocol(status) => format!("Protocol Error (status:{status:?})"),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ESC/VP.net Error: {}, {}",
            self.kind().to_string(),
            self.message()
        )
    }
}
impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::IO(value.kind()),
            message: value.to_string(),
        }
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_: FromUtf8Error) -> Self {
        Self {
            kind: ErrorKind::Decoding,
            message: "Error while decoding string".to_string(),
        }
    }
}

impl From<TryFromIntError> for Error {
    fn from(_: TryFromIntError) -> Self {
        Self {
            kind: ErrorKind::Decoding,
            message: "Collection length too big to be encoded".to_string(),
        }
    }
}

impl From<Status> for Error {
    fn from(value: Status) -> Self {
        use Status::*;
        Self {
            kind: ErrorKind::Protocol(value.clone()),
            message: match value {
                BadRequest => "Bad Request",
                Unauthorized => "Unauthorized, a password header was expected",
                Forbidden => "Forbidden, bad password",
                RequestNotAllowed => "Request not allowed",
                ServiceUnavailable => "Service Unavalaible",
                VersionNotSupported => "Version not supported",
                _ => "This is not supposed to happen",
            }
            .to_string(),
        }
    }
}
