use std::io;
#[derive(Debug)]
pub enum ProtocolError {
    PasswordRequired,
    WrongPassword,
    Busy,
    BadResponse,
    WrongProtocol,
    BadCommand,
}
#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Protocol(ProtocolError),
    Parse,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IO(e)
    }
}
impl From<ProtocolError> for Error {
    fn from(e: ProtocolError) -> Self {
        Self::Protocol(e)
    }
}
