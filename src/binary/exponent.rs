use crate::{
    binary::BinaryBuf,
    num::Integer,
};

use core::{
    fmt::Debug,
    ops::{
        Add,
        Div,
        Mul,
        Sub,
    },
};

/**
The high-level operators needed to work with binary exponents.
*/
pub trait BinaryExponent: Integer {
    /**
    Account for digits on the integral side of the decimal point by raising the exponent.
    */
    fn raise(&self, by: usize) -> Self;

    /**
    Account for digits on the fractional side of the decimal point by lowering the exponent.
    */
    fn lower(&self, by: usize) -> Self;

    fn bias<D: BinaryBuf>(&self, decimal: &D) -> Self;

    fn unbias<D: BinaryBuf>(&self, decimal: &D) -> Self;

    fn emax<D: BinaryBuf>(decimal: &D) -> Self;

    fn emin<D: BinaryBuf>(decimal: &D) -> Self;
}

/**
The math operators needed to work with binary exponents.

The decimal floating point format still needs binary math for its exponent, which is stored
as a binary integer. The width of these exponents scales much more slowly than the width of the
decimal itself, so you can get quite far without needing arbitrary-precision integers for them.
*/
pub(crate) trait BinaryExponentMath:
    Integer
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Debug
    + Sized
{
    fn abs(self) -> Self;
    fn pow2(e: u32) -> Self;
    fn log2(self) -> usize;
}

macro_rules! impl_binary_exponent {
    ($(($i:ty, $bytes:ty)),*) => {
        $(
            impl BinaryExponent for $i {
                #[must_use]
                fn raise(&self, by: usize) -> Self {
                    *self + (by as $i)
                }

                #[must_use]
                fn lower(&self, by: usize) -> Self {
                    *self - (by as $i)
                }

                #[must_use]
                fn bias<D: BinaryBuf>(&self, decimal: &D) -> Self {
                    add_bias(decimal, *self)
                }

                #[must_use]
                fn unbias<D: BinaryBuf>(&self, decimal: &D) -> Self {
                    sub_bias(decimal, *self)
                }

                #[must_use]
                fn emax<D: BinaryBuf>(decimal: &D) -> Self {
                    emax(decimal.storage_width_bits())
                }

                #[must_use]
                fn emin<D: BinaryBuf>(decimal: &D) -> Self {
                    emin(decimal.storage_width_bits())
                }
            }

            impl BinaryExponentMath for $i {
                fn abs(self) -> Self {
                    <$i>::abs(self)
                }

                fn pow2(e: u32) -> Self {
                    let two: $i = 2;
                    two.pow(e)
                }

                fn log2(self) -> usize {
                    // We can approximate log2(i) for a base2 integer i by taking the bit position
                    // of its most significant non-zero bit.

                    ((<$i>::BITS - self.leading_zeros()) as usize).saturating_sub(1)
                }
            }
        )*
    };
}

impl_binary_exponent!((i32, [u8; 4]), (i64, [u8; 8]), (i128, [u8; 16]));

/**
Apply the bias to an exponent.
*/
pub(crate) fn add_bias<D: BinaryBuf, N: BinaryExponentMath>(decimal: &D, exp: N) -> N {
    bias::<N>(decimal.storage_width_bits(), decimal.precision_digits()) + exp
}

/**
Remove the bias from an exponent.
*/
pub(crate) fn sub_bias<D: BinaryBuf, N: BinaryExponentMath>(decimal: &D, exp: N) -> N {
    exp - bias::<N>(decimal.storage_width_bits(), decimal.precision_digits())
}

// These methods follow the formulas given in the IEEE754-2019 standard.
//
// The standard defines the following parameters for decimal floating points that determine
// the range of exponent and significand values they can encode:
//
// - `k`: storage width in bits.
// - `p`: precision in digits.
// - `emax`: the maximum exponent.
// - `bias`: the value to bias exponents with, so that they're always positive integers.

/**
Calculate the maximum exponent that can be encoded by a decimal with a given bit-width.
*/
pub(crate) fn emax<N: BinaryExponentMath>(storage_width_bits: usize) -> N {
    // emax = 3 * 2.pow(k / 16 + 3)
    N::from_i32(3) * N::pow2((storage_width_bits / 16 + 3) as u32)
}

/**
Calculate the minimum exponent that can be encoded by a decimal with a given bit-width.
*/
pub(crate) fn emin<N: BinaryExponentMath>(storage_width_bits: usize) -> N {
    N::from_i32(1) - emax(storage_width_bits)
}

/**
Calculate the bias value to use for a decimal with a given bit-width and number of digits.
*/
pub(crate) fn bias<N: BinaryExponentMath>(storage_width_bits: usize, precision_digits: usize) -> N {
    // bias = emax + p - 2
    emax::<N>(storage_width_bits) + N::from_i32(precision_digits as i32) - N::from_i32(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bias_32() {
        assert_eq!(101, bias::<i32>(32, 7));
    }

    #[test]
    fn bias_64() {
        assert_eq!(398, bias::<i32>(64, 16));
    }

    #[test]
    fn bias_96() {
        assert_eq!(1559, bias::<i32>(96, 25));
    }

    #[test]
    fn bias_128() {
        assert_eq!(6176, bias::<i32>(128, 34));
    }

    #[test]
    fn bias_160() {
        assert_eq!(24617, bias::<i32>(160, 43));
    }

    #[test]
    fn bias_256() {
        assert_eq!(1572932, bias::<i64>(256, 70));
    }

    #[test]
    fn emax_32() {
        assert_eq!(96, emax::<i32>(32));
    }

    #[test]
    fn emax_64() {
        assert_eq!(384, emax::<i32>(64));
    }

    #[test]
    fn emax_96() {
        assert_eq!(1536, emax::<i32>(96));
    }

    #[test]
    fn emax_128() {
        assert_eq!(6144, emax::<i32>(128));
    }

    #[test]
    fn emax_160() {
        assert_eq!(24576, emax::<i32>(160));
    }

    #[test]
    fn emax_256() {
        assert_eq!(1572864, emax::<i64>(256));
    }
}
