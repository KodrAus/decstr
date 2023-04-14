use crate::{
    binary::{
        encode_max,
        encode_min,
        Decimal128Buf,
    },
    text::FixedSizeTextBuf,
};

/**
A 128bit decimal number.
*/
pub struct Bitstring128(Decimal128Buf);

impl Bitstring128 {
    pub fn from_le_bytes(bytes: [u8; 16]) -> Self {
        Self(Decimal128Buf(bytes))
    }

    pub fn as_le_bytes(&self) -> &[u8; 16] {
        // Even on big-endian platforms we always encode numbers in little-endian order
        &(self.0).0
    }

    pub fn zero() -> Self {
        Self::from(0u8)
    }

    pub fn max() -> Self {
        let mut buf = Decimal128Buf::ZERO;

        encode_max(&mut buf, false);

        Self(buf)
    }

    pub fn min() -> Self {
        let mut buf = Decimal128Buf::ZERO;

        encode_max(&mut buf, true);

        Self(buf)
    }

    pub fn min_positive() -> Self {
        let mut buf = Decimal128Buf::ZERO;

        encode_min(&mut buf, false);

        Self(buf)
    }
}

classify!(Bitstring128);

try_s2d!(FixedSizeTextBuf::<128> => Bitstring128);
d2s!(Bitstring128);

f2d!(f32 => from_f32 => Bitstring128);
f2d!(f64 => from_f64 => Bitstring128);

try_d2f!(Bitstring128 => to_f32 => f32);
try_d2f!(Bitstring128 => to_f64 => f64);

i2d!(i8 => from_i8 => Bitstring128);
i2d!(i16 => from_i16 => Bitstring128);
i2d!(i32 => from_i32 => Bitstring128);
i2d!(i64 => from_i64 => Bitstring128);
try_i2d!(i128 => from_i128 => Bitstring128);

try_d2i!(Bitstring128 => to_i8 => i8);
try_d2i!(Bitstring128 => to_i16 => i16);
try_d2i!(Bitstring128 => to_i32 => i32);
try_d2i!(Bitstring128 => to_i64 => i64);
try_d2i!(Bitstring128 => to_i128 => i128);

i2d!(u8 => from_u8 => Bitstring128);
i2d!(u16 => from_u16 => Bitstring128);
i2d!(u32 => from_u32 => Bitstring128);
i2d!(u64 => from_u64 => Bitstring128);
try_i2d!(u128 => from_u128 => Bitstring128);

try_d2i!(Bitstring128 => to_u8 => u8);
try_d2i!(Bitstring128 => to_u16 => u16);
try_d2i!(Bitstring128 => to_u32 => u32);
try_d2i!(Bitstring128 => to_u64 => u64);
try_d2i!(Bitstring128 => to_u128 => u128);
