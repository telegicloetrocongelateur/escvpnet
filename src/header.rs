use super::Error;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]

pub enum Header {
    Password(Option<String>),
    NewPassword(Option<String>),
    ProjectorName(Option<String>),
    IMType(u8),
    ProjectorCommandType,
}

impl Header {
    pub const SIZE: usize = 18;
}

impl From<Header> for [u8; 18] {
    fn from(header: Header) -> Self {
        match header {
            Header::Password(password) => match password {
                None => [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                Some(password) => {
                    let mut array = [1; 18];
                    array[2..].copy_from_slice(password.as_bytes());
                    array
                }
            },
            Header::NewPassword(name) => match name {
                None => [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                Some(name) => {
                    let mut array = [2; 18];
                    array[1] = 1;
                    array[2..].copy_from_slice(name.as_bytes());
                    array
                }
            },
            Header::ProjectorName(name) => match name {
                None => [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                Some(name) => {
                    let mut array = [3; 18];
                    array[1] = 1;
                    array[2..].copy_from_slice(name.as_bytes());
                    array
                }
            },
            Header::IMType(n) => [4, n, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            Header::ProjectorCommandType => [5, 33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        }
    }
}
impl TryFrom<[u8; 18]> for Header {
    type Error = Error;
    fn try_from(value: [u8; 18]) -> Result<Self, Self::Error> {
        match value[0] {
            1 => match value[1] {
                0 => Ok(Self::Password(None)),
                1 => Ok(Self::Password(Some(
                    String::from_utf8_lossy(&value[2..18]).to_string(),
                ))),
                _ => Err(Error::ParseError),
            },
            2 => match value[1] {
                0 => Ok(Self::NewPassword(None)),
                1 => Ok(Self::NewPassword(Some(
                    String::from_utf8_lossy(&value[2..18]).to_string(),
                ))),
                _ => Err(Error::ParseError),
            },
            3 => match value[1] {
                0 => Ok(Self::ProjectorName(None)),
                1 => Ok(Self::ProjectorName(Some(
                    String::from_utf8_lossy(&value[2..18]).to_string(),
                ))),
                _ => Err(Error::ParseError),
            },
            4 => Ok(Self::IMType(value[1])),
            5 => Ok(Self::ProjectorCommandType),
            _ => Err(Self::Error::ParseError),
        }
    }
}
impl TryFrom<&[u8]> for Header {
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let attr_value: u8 = *value.get(1).ok_or(Self::Error::ParseError)?;
        let information = value.get(2..18).ok_or(Self::Error::ParseError)?;
        match value.first().ok_or(Self::Error::ParseError)? {
            1 => match attr_value {
                0 => Ok(Self::Password(None)),
                1 => Ok(Self::Password(Some(
                    String::from_utf8_lossy(information).to_string(),
                ))),
                _ => Err(Error::ParseError),
            },
            2 => match attr_value {
                0 => Ok(Self::NewPassword(None)),
                1 => Ok(Self::NewPassword(Some(
                    String::from_utf8_lossy(information).to_string(),
                ))),
                _ => Err(Error::ParseError),
            },
            3 => match attr_value {
                0 => Ok(Self::ProjectorName(None)),
                1 => Ok(Self::ProjectorName(Some(
                    String::from_utf8_lossy(information).to_string(),
                ))),
                _ => Err(Error::ParseError),
            },
            4 => Ok(Self::IMType(value[1])),
            5 => Ok(Self::ProjectorCommandType),
            _ => Err(Self::Error::ParseError),
        }
    }
}

#[test]
fn bytes_to_header() {
    assert_eq!(
        Header::try_from(*b"\x01\x010123456789ABCDEF").unwrap(),
        Header::Password(Some("0123456789ABCDEF".to_string()))
    );
    assert_eq!(
        Header::try_from(*b"\x01\x00\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0").unwrap(),
        Header::Password(None)
    );

    assert_eq!(
        Header::try_from(*b"\x02\x00\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0").unwrap(),
        Header::NewPassword(None)
    );

    assert_eq!(
        Header::try_from(*b"\x04\x0a\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0").unwrap(),
        Header::IMType(10)
    );

    assert_eq!(
        Header::try_from(*b"\x05\x21\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0").unwrap(),
        Header::ProjectorCommandType
    )
}
