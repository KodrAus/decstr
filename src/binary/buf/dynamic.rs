use core::{
    fmt,
    ops::Index,
};

use crate::{
    binary::{
        exponent::BinaryExponent,
        try_with_at_least_precision,
        BinaryBuf,
    },
    num::Integer,
    OverflowError,
};

// Decimal
#[derive(Debug, Clone, Copy)]
pub(crate) struct DynamicBinaryBuf<const N: usize> {
    buf: [u8; N],
    len: u32,
}

impl<const N: usize> DynamicBinaryBuf<N> {
    pub(crate) const ZERO: Self = DynamicBinaryBuf {
        buf: [0; N],
        len: N as u32,
    };
}

pub(crate) struct DynamicBinaryExponent(i32);

pub(crate) struct DynamicBinaryExponentBytes([u8; 4]);

impl<const N: usize> BinaryBuf for DynamicBinaryBuf<N> {
    type Exponent = DynamicBinaryExponent;

    fn try_exponent_from_ascii<I: Iterator<Item = u8>>(
        is_negative: bool,
        ascii: I,
    ) -> Result<DynamicBinaryExponent, OverflowError>
    where
        Self::Exponent: Sized,
    {
        DynamicBinaryExponent::try_from_ascii(is_negative, ascii)
            .ok_or_else(|| OverflowError::exponent_out_of_range(4, "the exponent would overflow"))
    }

    fn try_with_at_least_storage_width_bytes(bytes: usize) -> Result<Self, OverflowError> {
        if bytes > N {
            Err(OverflowError::would_overflow(N, bytes))
        } else {
            Ok(DynamicBinaryBuf {
                buf: [0; N],
                len: bytes as u32,
            })
        }
    }

    fn try_with_at_least_precision(
        integer_digits: usize,
        integer_exponent: Option<&Self::Exponent>,
    ) -> Result<Self, OverflowError>
    where
        Self: Sized,
    {
        try_with_at_least_precision(integer_digits, integer_exponent.map(|e| e.0))
    }

    fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.buf[..self.len as usize]
    }

    fn bytes(&self) -> &[u8] {
        &self.buf[..self.len as usize]
    }
}

impl Integer for DynamicBinaryExponent {
    type Bytes = DynamicBinaryExponentBytes;

    fn try_from_ascii<I: Iterator<Item = u8>>(is_negative: bool, ascii: I) -> Option<Self> {
        Some(DynamicBinaryExponent(i32::try_from_ascii(
            is_negative,
            ascii,
        )?))
    }

    fn from_le_bytes<I: Iterator<Item = u8>>(bytes: I) -> Self {
        DynamicBinaryExponent(Integer::from_le_bytes(bytes))
    }

    fn from_i32(exp: i32) -> Self {
        DynamicBinaryExponent(i32::from_i32(exp))
    }

    fn to_i32(&self) -> Option<i32> {
        (self.0).try_into().ok()
    }

    fn is_negative(&self) -> bool {
        self.0.is_negative()
    }

    fn to_le_bytes(&self) -> Self::Bytes {
        DynamicBinaryExponentBytes(self.0.to_le_bytes())
    }

    fn to_fmt<W: fmt::Write>(&self, mut out: W) -> fmt::Result {
        write!(out, "{}", self.0)
    }
}

impl BinaryExponent for DynamicBinaryExponent {
    #[must_use]
    fn raise(&self, integer_digits: usize) -> Self {
        DynamicBinaryExponent(self.0.raise(integer_digits))
    }

    #[must_use]
    fn lower(&self, fractional_digits: usize) -> Self {
        DynamicBinaryExponent(self.0.lower(fractional_digits))
    }

    #[must_use]
    fn bias<D: BinaryBuf>(&self, decimal: &D) -> Self {
        DynamicBinaryExponent(self.0.bias(decimal))
    }

    #[must_use]
    fn unbias<D: BinaryBuf>(&self, decimal: &D) -> Self {
        DynamicBinaryExponent(self.0.unbias(decimal))
    }

    #[must_use]
    fn emax<D: BinaryBuf>(decimal: &D) -> Self {
        DynamicBinaryExponent(i32::emax(decimal))
    }

    #[must_use]
    fn emin<D: BinaryBuf>(decimal: &D) -> Self {
        DynamicBinaryExponent(i32::emin(decimal))
    }
}

impl fmt::Display for DynamicBinaryExponent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for DynamicBinaryExponent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl Index<usize> for DynamicBinaryExponentBytes {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        // Since the exponent is dynamically sized, it might not actually have as
        // many bytes as a decimal is expecting. If we don't have one, then return a `0`
        // byte instead
        if let Some(b) = self.0.get(index) {
            b
        } else {
            &0
        }
    }
}
