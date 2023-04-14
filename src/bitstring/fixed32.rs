use core::fmt;

use crate::{
    binary::{
        encode_max,
        encode_min,
        Decimal32Buf,
    },
    convert,
    text::FixedSizeTextBuf,
    Error,
};

/**
A 32bit decimal number.
*/
#[derive(Clone, Copy)]
pub struct Bitstring32(Decimal32Buf);

impl Bitstring32 {
    pub fn from_le_bytes(bytes: [u8; 4]) -> Self {
        Self(Decimal32Buf(bytes))
    }

    pub fn as_le_bytes(&self) -> &[u8; 4] {
        // Even on big-endian platforms we always encode numbers in little-endian order
        &(self.0).0
    }

    pub fn zero() -> Self {
        Self::from(0u8)
    }

    pub fn max() -> Self {
        let mut buf = Decimal32Buf([0; 4]);

        encode_max(&mut buf, false);

        Self(buf)
    }

    pub fn min() -> Self {
        let mut buf = Decimal32Buf([0; 4]);

        encode_max(&mut buf, true);

        Self(buf)
    }

    pub fn min_positive() -> Self {
        let mut buf = Decimal32Buf([0; 4]);

        encode_min(&mut buf, false);

        Self(buf)
    }
}

classify!(Bitstring32);

try_s2d!(FixedSizeTextBuf::<16> => Bitstring32);
d2s!(Bitstring32);

try_f2d!(f32 => from_f32 => Bitstring32);
try_f2d!(f64 => from_f64 => Bitstring32);

try_d2f!(Bitstring32 => to_f32 => f32);
d2f!(Bitstring32 => to_f64 => f64);

i2d!(i8 => from_i8 => Bitstring32);
i2d!(i16 => from_i16 => Bitstring32);
try_i2d!(i32 => from_i32 => Bitstring32);
try_i2d!(i64 => from_i64 => Bitstring32);
try_i2d!(i128 => from_i128 => Bitstring32);

try_d2i!(Bitstring32 => to_i8 => i8);
try_d2i!(Bitstring32 => to_i16 => i16);
try_d2i!(Bitstring32 => to_i32 => i32);
try_d2i!(Bitstring32 => to_i64 => i64);
try_d2i!(Bitstring32 => to_i128 => i128);

i2d!(u8 => from_u8 => Bitstring32);
i2d!(u16 => from_u16 => Bitstring32);
try_i2d!(u32 => from_u32 => Bitstring32);
try_i2d!(u64 => from_u64 => Bitstring32);
try_i2d!(u128 => from_u128 => Bitstring32);

try_d2i!(Bitstring32 => to_u8 => u8);
try_d2i!(Bitstring32 => to_u16 => u16);
try_d2i!(Bitstring32 => to_u32 => u32);
try_d2i!(Bitstring32 => to_u64 => u64);
try_d2i!(Bitstring32 => to_u128 => u128);
