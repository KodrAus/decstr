use crate::{
    binary::{
        BinaryBuf,
        DynamicBinaryBuf,
    },
    text::ArrayTextBuf,
    Error,
    OverflowError,
};

#[cfg(test)]
use crate::binary::encode_max;

/**
A dynamically sized decimal number with enough precision to fit any Rust primitive number.
*/
#[derive(Clone, Copy)]
pub struct Bitstring(DynamicBinaryBuf<20>);

impl Bitstring {
    /**
    Try create a decimal from the given buffer.

    The buffer is assumed to be in little-endian byte-order already.
    This method will fail if the buffer length is not a multiple of 4 bytes, or it's too
    big to fit in a `Bitstring`.
    */
    pub fn try_from_le_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() == 0 || bytes.len() % 4 != 0 {
            Err(OverflowError::exact_size_mismatch(
                bytes.len(),
                bytes.len() + 4 - (bytes.len() % 4),
                "decimals must be a multiple of 32 bits (4 bytes)",
            ))?;
        }

        let mut buf = DynamicBinaryBuf::try_with_exactly_storage_width_bytes(bytes.len())?;

        buf.bytes_mut().copy_from_slice(bytes);

        Ok(Self(buf))
    }

    /**
    Get a reference to the underlying bitstring buffer.

    This buffer is always stored in little-endain byte-order, regardless of the endianness
    of the platform.
    */
    pub fn as_le_bytes(&self) -> &[u8] {
        // Even on big-endian platforms we always encode numbers in little-endian order
        self.0.bytes()
    }

    /**
    Create a decimal with the finite value zero.
    */
    pub fn zero() -> Self {
        Self::from(0u8)
    }

    #[cfg(test)]
    fn max() -> Self {
        let mut buf = DynamicBinaryBuf::ZERO;

        encode_max(&mut buf, false);

        Self(buf)
    }

    #[cfg(test)]
    fn min() -> Self {
        let mut buf = DynamicBinaryBuf::ZERO;

        encode_max(&mut buf, true);

        Self(buf)
    }
}

classify!(Bitstring);

try_s2d!(ArrayTextBuf::<128> => Bitstring);
d2s!(Bitstring);

f2d!(f32 => from_f32 => Bitstring);
f2d!(f64 => from_f64 => Bitstring);

try_d2f!(Bitstring => to_f32 => f32);
try_d2f!(Bitstring => to_f64 => f64);

i2d!(i8 => from_i8 => Bitstring);
i2d!(i16 => from_i16 => Bitstring);
i2d!(i32 => from_i32 => Bitstring);
i2d!(i64 => from_i64 => Bitstring);
i2d!(i128 => from_i128 => Bitstring);

try_d2i!(Bitstring => to_i8 => i8);
try_d2i!(Bitstring => to_i16 => i16);
try_d2i!(Bitstring => to_i32 => i32);
try_d2i!(Bitstring => to_i64 => i64);
try_d2i!(Bitstring => to_i128 => i128);

i2d!(u8 => from_u8 => Bitstring);
i2d!(u16 => from_u16 => Bitstring);
i2d!(u32 => from_u32 => Bitstring);
i2d!(u64 => from_u64 => Bitstring);
i2d!(u128 => from_u128 => Bitstring);

try_d2i!(Bitstring => to_u8 => u8);
try_d2i!(Bitstring => to_u16 => u16);
try_d2i!(Bitstring => to_u32 => u32);
try_d2i!(Bitstring => to_u64 => u64);
try_d2i!(Bitstring => to_u128 => u128);
