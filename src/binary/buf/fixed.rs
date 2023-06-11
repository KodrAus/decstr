use core::marker::PhantomData;

use crate::{
    binary::{
        try_with_at_least_precision,
        BinaryBuf,
        BinaryExponent,
        BinaryExponentMath,
    },
    OverflowError,
};

/**
A fixed-size array that always encodes a decimal with the same precision.
*/
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct FixedBinaryBuf<const N: usize, E>([u8; N], PhantomData<E>);

impl<const N: usize, E> FixedBinaryBuf<N, E> {
    #[inline]
    pub(crate) const fn from_le_bytes(buf: [u8; N]) -> Self {
        FixedBinaryBuf(buf, PhantomData)
    }
    #[inline]
    pub(crate) const fn as_le_bytes(&self) -> [u8; N] {
        self.0
    }
}

impl<const N: usize, E> From<[u8; N]> for FixedBinaryBuf<N, E> {
    #[inline]
    fn from(buf: [u8; N]) -> FixedBinaryBuf<N, E> {
        Self::from_le_bytes(buf)
    }
}

impl<const N: usize, E> From<FixedBinaryBuf<N, E>> for [u8; N] {
    fn from(value: FixedBinaryBuf<N, E>) -> Self {
        value.0
    }
}

impl<const N: usize, E> AsRef<[u8; N]> for FixedBinaryBuf<N, E> {
    fn as_ref(&self) -> &[u8; N] {
        &self.0
    }
}

impl<const N: usize, E> AsMut<[u8; N]> for FixedBinaryBuf<N, E> {
    fn as_mut(&mut self) -> &mut [u8; N] {
        &mut self.0
    }
}

impl<const N: usize, E> FixedBinaryBuf<N, E> {
    pub(crate) const ZERO: Self = FixedBinaryBuf([0; N], PhantomData);
}

// Decimal{32,64,128}
impl<const N: usize, E: BinaryExponent + BinaryExponentMath> BinaryBuf for FixedBinaryBuf<N, E> {
    type Exponent = E;

    fn try_exponent_from_ascii<I: Iterator<Item = u8>>(
        is_negative: bool,
        ascii: I,
    ) -> Result<E, OverflowError>
    where
        Self::Exponent: Sized,
    {
        E::try_from_ascii(is_negative, ascii).ok_or_else(|| {
            OverflowError::exponent_out_of_range(4, "the exponent would overflow an `i32`")
        })
    }

    fn try_with_at_least_storage_width_bytes(bytes: usize) -> Result<Self, OverflowError> {
        if bytes > N {
            Err(OverflowError::would_overflow(N, bytes))
        } else {
            Ok(FixedBinaryBuf([0; N], PhantomData))
        }
    }

    fn try_with_at_least_precision(
        integer_digits: usize,
        integer_exponent: Option<&Self::Exponent>,
    ) -> Result<Self, OverflowError>
    where
        Self: Sized,
    {
        try_with_at_least_precision(integer_digits, integer_exponent.cloned())
    }

    fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }

    fn bytes(&self) -> &[u8] {
        &self.0
    }
}
