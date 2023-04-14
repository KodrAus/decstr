use crate::{
    binary::{
        try_with_at_least_precision,
        BinaryBuf,
    },
    num::Integer,
    OverflowError,
};

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct Decimal32Buf(pub(crate) [u8; 4]);

impl Decimal32Buf {
    pub(crate) const ZERO: Self = Decimal32Buf([0; 4]);
}

// Decimal32
impl BinaryBuf for Decimal32Buf {
    type Exponent = i32;

    fn try_exponent_from_ascii<I: Iterator<Item = u8>>(
        is_negative: bool,
        ascii: I,
    ) -> Result<i32, OverflowError>
    where
        Self::Exponent: Sized,
    {
        i32::try_from_ascii(is_negative, ascii).ok_or_else(|| {
            OverflowError::exponent_out_of_range(4, "the exponent would overflow an `i32`")
        })
    }

    fn try_with_at_least_storage_width_bytes(bytes: usize) -> Result<Self, OverflowError> {
        if bytes > 4 {
            Err(OverflowError::would_overflow(4, bytes))
        } else {
            Ok(Decimal32Buf([0; 4]))
        }
    }

    fn try_with_at_least_precision(
        integer_digits: usize,
        integer_exponent: Option<&Self::Exponent>,
    ) -> Result<Self, OverflowError>
    where
        Self: Sized,
    {
        try_with_at_least_precision(integer_digits, integer_exponent.copied())
    }

    fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }

    fn bytes(&self) -> &[u8] {
        &self.0
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct Decimal64Buf(pub(crate) [u8; 8]);

impl Decimal64Buf {
    pub(crate) const ZERO: Self = Decimal64Buf([0; 8]);
}

// Decimal64
impl BinaryBuf for Decimal64Buf {
    type Exponent = i32;

    fn try_exponent_from_ascii<I: Iterator<Item = u8>>(
        is_negative: bool,
        ascii: I,
    ) -> Result<i32, OverflowError>
    where
        Self::Exponent: Sized,
    {
        i32::try_from_ascii(is_negative, ascii).ok_or_else(|| {
            OverflowError::exponent_out_of_range(4, "the exponent would overflow an `i32`")
        })
    }

    fn try_with_at_least_storage_width_bytes(bytes: usize) -> Result<Self, OverflowError> {
        if bytes > 8 {
            Err(OverflowError::would_overflow(8, bytes))
        } else {
            Ok(Decimal64Buf([0; 8]))
        }
    }

    fn try_with_at_least_precision(
        integer_digits: usize,
        integer_exponent: Option<&Self::Exponent>,
    ) -> Result<Self, OverflowError>
    where
        Self: Sized,
    {
        try_with_at_least_precision(integer_digits, integer_exponent.copied())
    }

    fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }

    fn bytes(&self) -> &[u8] {
        &self.0
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct Decimal128Buf(pub(crate) [u8; 16]);

impl Decimal128Buf {
    pub(crate) const ZERO: Self = Decimal128Buf([0; 16]);
}

// Decimal128
impl BinaryBuf for Decimal128Buf {
    type Exponent = i32;

    fn try_exponent_from_ascii<I: Iterator<Item = u8>>(
        is_negative: bool,
        ascii: I,
    ) -> Result<i32, OverflowError>
    where
        Self::Exponent: Sized,
    {
        i32::try_from_ascii(is_negative, ascii).ok_or_else(|| {
            OverflowError::exponent_out_of_range(4, "the exponent would overflow an `i32`")
        })
    }

    fn try_with_at_least_storage_width_bytes(bytes: usize) -> Result<Self, OverflowError> {
        if bytes > 16 {
            Err(OverflowError::would_overflow(16, bytes))
        } else {
            Ok(Decimal128Buf([0; 16]))
        }
    }

    fn try_with_at_least_precision(
        integer_digits: usize,
        integer_exponent: Option<&Self::Exponent>,
    ) -> Result<Self, OverflowError>
    where
        Self: Sized,
    {
        try_with_at_least_precision(integer_digits, integer_exponent.copied())
    }

    fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }

    fn bytes(&self) -> &[u8] {
        &self.0
    }
}
