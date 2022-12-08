pub use super::error;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Identifier {
    Hello,
    Password,
    Connect,
}

impl From<Identifier> for u8 {
    fn from(identifier: Identifier) -> Self {
        match identifier {
            Identifier::Hello => 1,
            Identifier::Password => 2,
            Identifier::Connect => 3,
        }
    }
}
impl TryFrom<u8> for Identifier {
    type Error = error::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Identifier::Hello),
            2 => Ok(Identifier::Password),
            3 => Ok(Identifier::Connect),
            _ => Err(Self::Error::ParseError),
        }
    }
}
