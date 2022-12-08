use super::Error;
type Result<T> = std::result::Result<T, super::Error>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Status {
    Null = 0x00,
    Ok = 0x20,
    BadRequest = 0x40,
    Unauthorized = 0x41,
    Forbidden = 0x43,
    NotAllowed = 0x45,
    Unavailable = 0x53,
    VersionNotSupported = 0x55,
}

impl Status {
    pub fn as_result(self) -> Result<()> {
        match self {
            Self::Ok => Ok(()),
            status => Err(super::Error::Status(status)),
        }
    }
}

impl From<Status> for u8 {
    fn from(status: Status) -> Self {
        status as u8
    }
}
impl TryFrom<u8> for Status {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self> {
        match value {
            0x00 => Ok(Status::Null),
            0x20 => Ok(Status::Ok),
            0x40 => Ok(Status::BadRequest),
            0x41 => Ok(Status::Unauthorized),
            0x43 => Ok(Status::Forbidden),
            0x45 => Ok(Status::NotAllowed),
            0x53 => Ok(Status::Unavailable),
            0x55 => Ok(Status::VersionNotSupported),
            _ => Err(Self::Error::ParseError),
        }
    }
}

#[test]
fn bytes_to_status() {
    assert_eq!(Status::try_from(0x20).unwrap(), Status::Ok);
}
