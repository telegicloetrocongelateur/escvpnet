pub trait Length {
    const LENGTH: usize;
}
pub trait Decode: Sized + Length {
    type Error;
    fn decode(data: [u8; Self::LENGTH]) -> Result<Self, Self::Error>;
}

pub trait Encode: Length {
    type Error;
    fn encode(self) -> Result<[u8; Self::LENGTH], Self::Error>;
}

pub trait DecodeFrom<R>: Sized {
    type Error;
    fn decode_from(reader: &mut R) -> Result<Self, Self::Error>;
}

impl<R: std::io::Read, D: Decode> DecodeFrom<R> for D
where
    D::Error: From<std::io::Error>,
    [(); Self::LENGTH]:,
{
    type Error = D::Error;
    fn decode_from(reader: &mut R) -> Result<Self, Self::Error> {
        let mut buf = std::array::from_fn(|_| 0);
        reader.read_exact(&mut buf)?;
        D::decode(buf)
    }
}

pub trait EncodeTo<W>: Sized {
    type Error;
    fn encode_to(self, writer: &mut W) -> Result<usize, Self::Error>;
}

impl<R: std::io::Write, E: Encode> EncodeTo<R> for E
where
    E::Error: From<std::io::Error>,
    [(); Self::LENGTH]:,
{
    type Error = E::Error;
    fn encode_to(self, writer: &mut R) -> Result<usize, Self::Error> {
        writer.write_all(&self.encode()?)?;
        Ok(E::LENGTH)
    }
}

impl<const N: usize> Decode for [u8; N] {
    type Error = crate::Error;
    fn decode(data: [u8; Self::LENGTH]) -> Result<Self, Self::Error> {
        Ok(data[..N].try_into().unwrap())
    }
}

impl<T: Decode, const N: usize> Length for [T; N] {
    const LENGTH: usize = T::LENGTH * N;
}

#[macro_export]
macro_rules! number_io {
    ($($type: ty),+) => {
        $(

        impl Length for $type {
            const LENGTH: usize = (Self::BITS / 8) as usize;
        }
        impl Decode for $type {
            type Error = $crate::error::Error;
            fn decode(data: [u8; Self::LENGTH]) -> Result<Self, Self::Error> {
                Ok(Self::from_be_bytes(data))
            }
        }
        impl Encode for $type {
            type Error = $crate::error::Error;
            fn encode(self) -> Result<[u8;Self::LENGTH], Self::Error> {
                Ok(self.to_be_bytes())
            }
        }
                )*
    };
}
number_io!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize);
