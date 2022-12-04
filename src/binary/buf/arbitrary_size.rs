use core::{
    cmp,
    fmt,
    ops::Index,
};

use crate::binary::{
    buf::BinaryExponentMath,
    exponent::{
        add_bias,
        sub_bias,
        BinaryExponent,
    },
    try_with_at_least_precision,
    BinaryBuf,
    Overflow,
};

use num_bigint::BigInt;
use num_traits::{
    Signed,
    ToPrimitive,
};

// BigDecimal
pub(crate) struct ArbitrarySizedBinaryBuf(Vec<u8>);

pub(crate) struct ArbitrarySizedBinaryExponent(BigInt);

pub(crate) struct ArbitrarySizedBinaryExponentBytes(Vec<u8>);

impl BinaryBuf for ArbitrarySizedBinaryBuf {
    type Exponent = ArbitrarySizedBinaryExponent;

    fn try_with_at_least_storage_width_bits(bits: usize) -> Result<Self, Overflow> {
        let bytes = cmp::max(4, bits / 8);

        Ok(ArbitrarySizedBinaryBuf(vec![0; bytes]))
    }

    fn try_with_at_least_precision(
        integer_digits: usize,
        fractional_digits: usize,
        exponent: Option<&Self::Exponent>,
    ) -> Result<Self, Overflow>
    where
        Self: Sized,
    {
        try_with_at_least_precision(
            integer_digits,
            fractional_digits,
            exponent.map(|e| e.0.clone()),
        )
    }

    fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }

    fn bytes(&self) -> &[u8] {
        &self.0
    }
}

impl BinaryExponent for ArbitrarySizedBinaryExponent {
    type Bytes = ArbitrarySizedBinaryExponentBytes;

    fn from_ascii(is_negative: bool, digits: &[u8]) -> Self {
        let mut exp = BigInt::parse_bytes(digits, 10).expect("pre-validated digits");
        if is_negative {
            exp = -exp;
        }

        ArbitrarySizedBinaryExponent(exp)
    }

    fn from_binary<I: Iterator<Item = u8>>(bytes: I) -> Self {
        let bytes = bytes.collect::<Vec<_>>();

        ArbitrarySizedBinaryExponent(BigInt::from_signed_bytes_le(&bytes))
    }

    fn from_i32(exp: i32) -> Self {
        ArbitrarySizedBinaryExponent(exp.into())
    }

    fn to_i32(&self) -> Option<i32> {
        BigInt::to_i32(&self.0)
    }

    fn to_i64(&self) -> Option<i64> {
        BigInt::to_i64(&self.0)
    }

    fn to_i128(&self) -> Option<i128> {
        BigInt::to_i128(&self.0)
    }

    #[must_use]
    fn raise(&self, integer_digits: usize) -> Self {
        ArbitrarySizedBinaryExponent((&self.0) + integer_digits)
    }

    #[must_use]
    fn lower(&self, fractional_digits: usize) -> Self {
        ArbitrarySizedBinaryExponent((&self.0) - fractional_digits)
    }

    #[must_use]
    fn bias<D: BinaryBuf>(&self, decimal: &D) -> Self {
        ArbitrarySizedBinaryExponent(add_bias(decimal, self.0.clone()))
    }

    #[must_use]
    fn unbias<D: BinaryBuf>(&self, decimal: &D) -> Self {
        ArbitrarySizedBinaryExponent(sub_bias(decimal, self.0.clone()))
    }

    fn to_le_bytes(&self) -> Self::Bytes {
        ArbitrarySizedBinaryExponentBytes(self.0.to_signed_bytes_le())
    }

    fn to_fmt<W: fmt::Write>(&self, mut out: W) -> fmt::Result {
        write!(out, "{}", self.0)
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

impl BinaryExponentMath for BigInt {
    fn new(n: usize) -> Self {
        BigInt::from(n)
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

    fn is_negative(&self) -> bool {
        Signed::is_negative(self)
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
