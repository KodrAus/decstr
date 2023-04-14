use crate::{
    binary::{
        exponent::{
            BinaryExponent,
            BinaryMath,
        },
        significand::precision_digits,
    },
    num::Integer,
    OverflowError,
};
use core::cmp;

#[cfg(feature = "arbitrary-precision")]
mod arbitrary_size;
mod dynamic_size;
mod fixed_size;

pub(crate) use self::{
    dynamic_size::*,
    fixed_size::*,
};

#[cfg(feature = "arbitrary-precision")]
pub(crate) use self::arbitrary_size::*;

/**
A buffer for an IEEE754-2019 compatible decimal-interchange-formatted number.

The buffer will have a particular size, which is always a multiple of 32 bits. The size of the buffer
also determines the size of the exponent it can encode. Since the exponent needs arithmetic support
it's treated as a generic parameter rather than fixed to a particular type.
*/
pub trait BinaryBuf {
    /**
    The kind of exponent this buffer needs.

    The exponent scales with the width of the decimal, but more slowly. This type will
    always have enough precision to represent any exponent value that can be encoded in
    this binary buffer.
    */
    type Exponent: BinaryExponent;

    /**
    Try convert a pre-validated stream of ASCII digits into a binary exponent.

    This method may fail if the exponent would overflow.
    */
    fn try_exponent_from_ascii<I: Iterator<Item = u8>>(
        is_negative: bool,
        ascii: I,
    ) -> Result<Self::Exponent, OverflowError>
    where
        Self::Exponent: Sized;

    /**
    Get the default value for an exponent when it's otherwise unspecified.
    */
    fn default_exponent() -> Self::Exponent
    where
        Self::Exponent: Sized,
    {
        Self::Exponent::zero()
    }

    /**
    Try get a buffer with at least enough precision for a `bits`-width decimal.

    If this method returns `Some`, then the buffer will have at least `bits`-width, but may have more.
    */
    fn try_with_at_least_storage_width_bytes(bytes: usize) -> Result<Self, OverflowError>
    where
        Self: Sized;

    fn try_with_exactly_storage_width_bytes(bytes: usize) -> Result<Self, OverflowError>
    where
        Self: Sized,
    {
        let buf = Self::try_with_at_least_storage_width_bytes(bytes)?;

        let got_len = buf.bytes().len();

        if got_len != bytes {
            Err(OverflowError::exact_size_mismatch(
                got_len,
                bytes,
                "the decimal didn't produce the exact size needed",
            ))
        } else {
            Ok(buf)
        }
    }

    /**
    Try get a buffer with at least enough precision to fit a number of digits and exponent.

    The number of digits and the maximum/minimum exponent values are calculated from the width
    of the decimal. If this method returns `Some` then the buffer is guaranteed to have at least `digits`
    of precision and be able to fit `exponent`, if specified.
    */
    fn try_with_at_least_precision(
        integer_digits: usize,
        integer_exponent: Option<&Self::Exponent>,
    ) -> Result<Self, OverflowError>
    where
        Self: Sized;

    /**
    The bit-width of this buffer.

    The width is expected to remain constant.
    */
    fn storage_width_bits(&self) -> usize {
        self.bytes().len() * 8
    }

    /**
    The number of significant digits this buffer can fit.
    */
    fn precision_digits(&self) -> usize {
        precision_digits(self.storage_width_bits())
    }

    /**
    The number of trailing significant digits this buffer can fit.

    The most significant digit is encoded differently to the others. This value determines the
    number of contiguous digits that are encoded using densely-packed-decimal encoding.
    */
    fn trailing_significand_digits(&self) -> usize {
        self.precision_digits() - 1
    }

    /**
    The number of bits dedicated to the trailing significand digits.
    */
    fn trailing_significand_width_bits(&self) -> usize {
        let bit_width = self.storage_width_bits();

        15 * bit_width / 16 - 10
    }

    /**
    The number of bits dedicated to the combination field.

    This field identifies the decimal as being either finite, infinite, or NaN. For finite
    numbers, it also encodes the most significant digit of the significand and the value of the exponent.
    */
    fn combination_width_bits(&self) -> usize {
        let bit_width = self.storage_width_bits();

        bit_width / 16 + 9
    }

    /**
    The bit-width of an exponent that can be encoded by this buffer.

    Not all exponents up to `exponent_width_bits` can be encoded in a buffer. This field just
    determines how many bits of an exponent should be used.
    */
    fn exponent_width_bits(&self) -> usize {
        self.combination_width_bits() - 3
    }

    /**
    The number of bits dedicated to the trailing exponent.

    For finite numbers, the most significant digit and the 2 most significant bits of the exponent
    are encoded together in the combination field. This value determines how many bits of the exponent
    should be written directly.
    */
    fn trailing_exponent_width_bits(&self) -> usize {
        self.combination_width_bits() - 5
    }

    /**
    Get an exclusive reference to the buffer.

    The buffered returned should have the same length as `storage_width_bits() / 8`.
    */
    fn bytes_mut(&mut self) -> &mut [u8];

    /**
    Get a shared reference to the buffer.

    The buffered returned should have the same length as `storage_width_bits() / 8`.
    */
    fn bytes(&self) -> &[u8];
}

pub(crate) fn try_with_at_least_precision<D: BinaryBuf, N: BinaryMath>(
    integer_digits: usize,
    integer_exponent: Option<N>,
) -> Result<D, OverflowError> {
    debug_assert_ne!(
        0, integer_digits,
        "decimals always have at least 1 integer digit"
    );

    // First, calculate the minimum storage width needed to fit the digits.
    let minimum_digit_precision_width_bytes =
        minimum_storage_width_bits_for_precision_digits(integer_digits) / 8;

    // Next, calculate the minimum storage width needed to fit the exponent.
    //
    // A decimal might have a tiny number of digits but a very large exponent, like `3e435654`.
    if let Some(integer_exponent) = integer_exponent {
        let minimum_exponent_precision_width_bytes =
            minimum_storage_width_bits_for_integer_exponent(integer_exponent) / 8;

        // The minimum storage width needed is the larger of what's needed for the digits and the exponent
        D::try_with_at_least_storage_width_bytes(cmp::max(
            minimum_digit_precision_width_bytes,
            minimum_exponent_precision_width_bytes,
        ))
    } else {
        D::try_with_at_least_storage_width_bytes(minimum_digit_precision_width_bytes)
    }
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
Calculate the minimum bit-width for a decimal that can encode a given exponent.
*/
pub(crate) fn minimum_storage_width_bits_for_integer_exponent<N: BinaryMath>(emax: N) -> usize {
    match emax.to_i32() {
        // If the exponent is small, check some specific bounds directly.
        //
        // Since the general case over-provisions, for these common sizes we just check them
        // rather than computing so they're always exact.
        Some(emax) if emax >= -101 && emax <= 90 => 32,
        Some(emax) if emax >= -398 && emax <= 369 => 64,
        Some(emax) if emax >= -1559 && emax <= 1512 => 96,
        Some(emax) if emax >= -6176 && emax <= 6111 => 128,
        Some(emax) if emax >= -24617 && emax <= 24534 => 160,
        // If the exponent is not small, compute an appropriate width
        _ => calculate_minimum_storage_width_bits_for_integer_exponent(emax),
    }
}

fn calculate_minimum_storage_width_bits_for_integer_exponent<N: BinaryMath>(emax: N) -> usize {
    // emax = 3 * 2.pow(k / 16 + 3)
    // k = 16 * ((log2(emax / 3) / log2(2)) - 3)
    let storage_width_bits =
        16 * ((N::log2(emax.abs() / N::from_i32(3)) / N::log2(N::from_i32(2))).saturating_sub(3));

    // Since the exponent is probably not exactly emin or emax, round the
    // width up to the nearest multiple of 32 bits
    let rem = storage_width_bits % 32;

    // Conservatively add 32 to the calculated width to account for rounding
    // and the need to adjust the exponent by an unknown number of digits based
    // on the chosen precision.
    //
    // This means this function will over-provision by 32 bits, but that might be
    // avoidable in the future if a better way of calculating the minimum width
    // is found.
    if rem == 0 {
        storage_width_bits + 32
    } else {
        storage_width_bits + 64 - rem
    }
}

/**
Calculate the minimum bit-width for a decimal that can encode a number of digits.
*/
pub(crate) fn minimum_storage_width_bits_for_precision_digits(precision_digits: usize) -> usize {
    match precision_digits {
        // If the number of digits is small, check some specific bounds directly.
        //
        // Since the general case over-provisions, for these common sizes we just check them
        // rather than computing so they're always exact.
        precision_digits if precision_digits <= 7 => 32,
        precision_digits if precision_digits <= 16 => 64,
        precision_digits if precision_digits <= 25 => 96,
        precision_digits if precision_digits <= 34 => 128,
        precision_digits if precision_digits <= 43 => 160,
        // If the number of digits is not small, compute an appropriate width
        _ => calculate_minimum_storage_width_bits_for_precision_digits(precision_digits),
    }
}

fn calculate_minimum_storage_width_bits_for_precision_digits(precision_digits: usize) -> usize {
    // d = 9k / 32 – 2
    // k = 32 * (d + 2) / 9
    let storage_width_bits = (precision_digits + 2) * 32 / 9;

    // Add 1 to the result to account for rounding.
    //
    // This means this function may over-provision storage for precisions
    // that would fit in a smaller size, but will never under-provision.
    let storage_width_bits = storage_width_bits + 1;

    // The number of digits requested might not be a multiple of a format
    // So we need to align it up to a multiple of 32 bits
    let rem = storage_width_bits % 32;

    if rem == 0 {
        storage_width_bits
    } else {
        storage_width_bits + 32 - rem
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimum_storage_case(
        target_width: usize,
        precision_digits_lower: Option<usize>,
        emin_lower: Option<i32>,
        emax_lower: Option<i32>,
        precision_digits_upper: usize,
        emin_upper: i32,
        emax_upper: i32,
    ) {
        if let Some(emin_lower) = emin_lower {
            assert_eq!(
                target_width,
                minimum_storage_width_bits_for_integer_exponent(emin_lower)
            );
        }

        if let Some(emax_lower) = emax_lower {
            assert_eq!(
                target_width,
                minimum_storage_width_bits_for_integer_exponent(emax_lower)
            );
        }

        if let Some(precision_digits_lower) = precision_digits_lower {
            assert_eq!(
                target_width,
                minimum_storage_width_bits_for_precision_digits(precision_digits_lower)
            );
        }

        assert_eq!(
            target_width,
            minimum_storage_width_bits_for_integer_exponent(emin_upper)
        );
        assert_eq!(
            target_width,
            minimum_storage_width_bits_for_integer_exponent(emax_upper)
        );
        assert_eq!(
            target_width,
            minimum_storage_width_bits_for_precision_digits(precision_digits_upper)
        );
    }

    #[test]
    fn minimum_storage_32() {
        minimum_storage_case(32, None, None, None, 7, -101, 90);
    }

    #[test]
    fn minimum_storage_64() {
        minimum_storage_case(64, Some(8), Some(-102), Some(91), 16, -398, 369);
    }

    #[test]
    fn minimum_storage_96() {
        minimum_storage_case(96, Some(17), Some(-399), Some(370), 25, -1559, 1512);
    }

    #[test]
    fn minimum_storage_128() {
        minimum_storage_case(128, Some(26), Some(-1560), Some(1513), 34, -6176, 6111);
    }

    #[test]
    fn minimum_storage_160() {
        minimum_storage_case(160, Some(35), Some(-6177), Some(6112), 43, -24617, 24534);
    }

    #[test]
    fn minimum_calculated_storage_from_exponent() {
        assert!(calculate_minimum_storage_width_bits_for_integer_exponent(90i32) >= 32);
        assert!(calculate_minimum_storage_width_bits_for_integer_exponent(-101i32) >= 32);

        assert!(calculate_minimum_storage_width_bits_for_integer_exponent(369i32) >= 64);
        assert!(calculate_minimum_storage_width_bits_for_integer_exponent(-398i32) >= 64);

        assert!(calculate_minimum_storage_width_bits_for_integer_exponent(1512i32) >= 96);
        assert!(calculate_minimum_storage_width_bits_for_integer_exponent(-1559i32) >= 96);

        assert!(calculate_minimum_storage_width_bits_for_integer_exponent(6111i32) >= 128);
        assert!(calculate_minimum_storage_width_bits_for_integer_exponent(-6176i32) >= 128);

        assert!(calculate_minimum_storage_width_bits_for_integer_exponent(24534i32) >= 160);
        assert!(calculate_minimum_storage_width_bits_for_integer_exponent(-24617i32) >= 160);

        assert!(calculate_minimum_storage_width_bits_for_integer_exponent(1572795i32) >= 256);
        assert!(calculate_minimum_storage_width_bits_for_integer_exponent(-1572932i32) >= 256);
    }

    #[test]
    fn minimum_calculated_storage_from_precision() {
        assert!(calculate_minimum_storage_width_bits_for_precision_digits(7) >= 32);
        assert!(calculate_minimum_storage_width_bits_for_precision_digits(16) >= 64);
        assert!(calculate_minimum_storage_width_bits_for_precision_digits(25) >= 96);
        assert!(calculate_minimum_storage_width_bits_for_precision_digits(34) >= 128);
        assert!(calculate_minimum_storage_width_bits_for_precision_digits(43) >= 160);
        assert!(calculate_minimum_storage_width_bits_for_precision_digits(70) >= 256);
    }
}
