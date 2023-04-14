use core::{
    fmt,
    ops::Index,
};

use crate::{
    binary::{
        emax,
        emin,
        exponent::{
            add_bias,
            sub_bias,
            BinaryExponent,
        },
        try_with_at_least_precision,
        BinaryBuf,
        BinaryExponentMath,
    },
    num::Integer,
    OverflowError,
};

use num_bigint::BigInt;
use num_traits::{
    Signed,
    ToPrimitive,
};

// Big Decimal
#[derive(Debug, Clone)]
pub(crate) struct ArbitrarySizedBinaryBuf(Vec<u8>);

pub(crate) struct ArbitrarySizedBinaryExponent(BigInt);

pub(crate) struct ArbitrarySizedBinaryExponentBytes(Vec<u8>);

impl BinaryBuf for ArbitrarySizedBinaryBuf {
    type Exponent = ArbitrarySizedBinaryExponent;

    fn try_exponent_from_ascii<I: Iterator<Item = u8>>(
        is_negative: bool,
        ascii: I,
    ) -> Result<ArbitrarySizedBinaryExponent, OverflowError>
    where
        Self::Exponent: Sized,
    {
        ArbitrarySizedBinaryExponent::try_from_ascii(is_negative, ascii)
            .ok_or_else(|| OverflowError::exponent_out_of_range(4, "the exponent would overflow"))
    }

    fn try_with_at_least_storage_width_bytes(bytes: usize) -> Result<Self, OverflowError> {
        Ok(ArbitrarySizedBinaryBuf(vec![0; bytes]))
    }

    fn try_with_at_least_precision(
        integer_digits: usize,
        integer_exponent: Option<&Self::Exponent>,
    ) -> Result<Self, OverflowError>
    where
        Self: Sized,
    {
        try_with_at_least_precision(integer_digits, integer_exponent.map(|e| e.0.clone()))
    }

    fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }

    fn bytes(&self) -> &[u8] {
        &self.0
    }
}

impl Integer for ArbitrarySizedBinaryExponent {
    type Bytes = ArbitrarySizedBinaryExponentBytes;

    fn try_from_ascii<I: Iterator<Item = u8>>(is_negative: bool, ascii: I) -> Option<Self> {
        Some(ArbitrarySizedBinaryExponent(Integer::try_from_ascii(
            is_negative,
            ascii,
        )?))
    }

    fn from_le_bytes<I: Iterator<Item = u8>>(bytes: I) -> Self {
        ArbitrarySizedBinaryExponent(Integer::from_le_bytes(bytes))
    }

    fn from_i32(exp: i32) -> Self {
        ArbitrarySizedBinaryExponent(Integer::from_i32(exp))
    }

    fn to_i32(&self) -> Option<i32> {
        Integer::to_i32(&self.0)
    }

    fn is_negative(&self) -> bool {
        Integer::is_negative(&self.0)
    }

    fn to_le_bytes(&self) -> Self::Bytes {
        ArbitrarySizedBinaryExponentBytes(Integer::to_le_bytes(&self.0))
    }

    fn to_fmt<W: fmt::Write>(&self, out: W) -> fmt::Result {
        Integer::to_fmt(&self.0, out)
    }
}

impl Integer for BigInt {
    type Bytes = Vec<u8>;

    fn try_from_ascii<I: Iterator<Item = u8>>(is_negative: bool, ascii: I) -> Option<Self>
    where
        Self: Sized,
    {
        let ascii = ascii.collect::<Vec<_>>();
        let n = BigInt::parse_bytes(&ascii, 10)?;

        Some(if is_negative { -n } else { n })
    }

    fn from_le_bytes<I: Iterator<Item = u8>>(bytes: I) -> Self {
        let buf = bytes.collect::<Vec<_>>();

        BigInt::from_signed_bytes_le(&buf)
    }

    fn from_i32(n: i32) -> Self {
        BigInt::from(n)
    }

    fn to_i32(&self) -> Option<i32> {
        ToPrimitive::to_i32(self)
    }

    fn is_negative(&self) -> bool {
        Signed::is_negative(self)
    }

    fn to_le_bytes(&self) -> Self::Bytes {
        self.to_signed_bytes_le()
    }

    fn to_fmt<W: fmt::Write>(&self, mut out: W) -> fmt::Result {
        write!(out, "{}", self)
    }
}

impl BinaryExponent for ArbitrarySizedBinaryExponent {
    #[must_use]
    fn raise(&self, integer_digits: usize) -> Self {
        ArbitrarySizedBinaryExponent(&self.0 + integer_digits)
    }

    #[must_use]
    fn lower(&self, fractional_digits: usize) -> Self {
        ArbitrarySizedBinaryExponent(&self.0 - fractional_digits)
    }

    #[must_use]
    fn bias<D: BinaryBuf>(&self, decimal: &D) -> Self {
        ArbitrarySizedBinaryExponent(add_bias(decimal, self.0.clone()))
    }

    #[must_use]
    fn unbias<D: BinaryBuf>(&self, decimal: &D) -> Self {
        ArbitrarySizedBinaryExponent(sub_bias(decimal, self.0.clone()))
    }

    #[must_use]
    fn emax<D: BinaryBuf>(decimal: &D) -> Self {
        ArbitrarySizedBinaryExponent(emax(decimal.storage_width_bits()))
    }

    #[must_use]
    fn emin<D: BinaryBuf>(decimal: &D) -> Self {
        ArbitrarySizedBinaryExponent(emin(decimal.storage_width_bits()))
    }
}

impl fmt::Display for ArbitrarySizedBinaryExponent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for ArbitrarySizedBinaryExponent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl Index<usize> for ArbitrarySizedBinaryExponentBytes {
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

impl BinaryExponentMath for BigInt {
    fn abs(self) -> Self {
        Signed::abs(&self)
    }

    fn pow2(e: u32) -> Self {
        BigInt::from(2).pow(e)
    }

    fn log2(self) -> usize {
        // We can approximate log2(i) for a base2 integer i by taking the bit position
        // of its most significant non-zero bit. The `BigInt` type doesn't use base2
        // digits though, it uses u32 as digits instead (which are called "limbs"), so we convert
        // the number into a base2 buffer and then look at the bits in the most significant byte

        let i_base2 = self.to_signed_bytes_le();

        let mut significant_byte_index = i_base2.len() - 1;

        // Find the most significant non-zero byte
        // We expect `BigInt` not to give us a lot of pointless empty bytes
        // but it's reasonable to assume it might pad them up to the nearest limb
        while significant_byte_index > 0 {
            if i_base2[significant_byte_index] != 0 {
                break;
            }

            significant_byte_index -= 1;
        }

        let significant_bit_index = 8 - i_base2[significant_byte_index].leading_zeros() as usize;

        let log2 = (significant_bit_index + (significant_byte_index * 8)).saturating_sub(1);

        log2
    }
}
