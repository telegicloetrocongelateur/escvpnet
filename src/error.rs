use super::Status;
#[derive(Debug)]
pub enum Error {
    IO,
    ParseError,
    Status(Status),
}

impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Self::IO
    }
} /*
  impl From<std::array::TryFromSliceError> for Error {
      fn from(_: std::array::TryFromSliceError) -> Self {
          Self::ParseError
      }
  }
  */
