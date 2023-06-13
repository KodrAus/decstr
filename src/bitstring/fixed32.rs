use crate::{
    binary::{
        encode_max,
        encode_min,
        FixedBinaryBuf,
    },
    text::ArrayTextBuf,
};

/**
A [32bit decimal number](https://en.wikipedia.org/wiki/Decimal32_floating-point_format).
*/
#[derive(Clone, Copy)]
pub struct Bitstring32(FixedBinaryBuf<4, i32>);

/**
Basic mathematical constants.
*/
impl Bitstring32 {
    /// 0
    pub const ZERO: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([0, 0, 80, 34]));

    /// 1
    pub const ONE: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([1, 0, 80, 34]));

    /// -1
    pub const NEG_ONE: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([1, 0, 80, 162]));

    /// Archimedes' constant (π)
    pub const PI: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([186, 6, 243, 45]));

    /// The full circle constant (τ)
    ///
    /// Equal to 2π.
    pub const TAU: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([203, 172, 244, 57]));

    /// π/2
    pub const FRAC_PI_2: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([250, 195, 251, 37]));

    /// π/3
    pub const FRAC_PI_3: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([251, 28, 241, 37]));

    /// π/4
    pub const FRAC_PI_4: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([222, 45, 255, 33]));

    /// π/6
    pub const FRAC_PI_6: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([222, 142, 250, 33]));

    /// π/8
    pub const FRAC_PI_8: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([95, 235, 246, 33]));

    /// 1/π
    pub const FRAC_1_PI: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([137, 97, 246, 33]));

    /// 2/π
    pub const FRAC_2_PI: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([25, 219, 252, 33]));

    /// 2/sqrt(π)
    pub const FRAC_2_SQRT_PI: Self =
        Bitstring32(FixedBinaryBuf::from_le_bytes([249, 161, 242, 37]));

    /// sqrt(2)
    pub const SQRT_2: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([19, 81, 248, 37]));

    /// 1/sqrt(2)
    pub const FRAC_1_SQRT_2: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([134, 28, 254, 33]));

    /// Euler's number (e)
    pub const E: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([11, 97, 254, 41]));

    /// log<sub>2</sub>(10)
    pub const LOG2_10: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([174, 133, 246, 45]));

    /// log<sub>2</sub>(e)
    pub const LOG2_E: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([91, 11, 249, 37]));

    /// log<sub>10</sub>(2)
    pub const LOG10_2: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([41, 4, 246, 33]));

    /// log<sub>10</sub>(e)
    pub const LOG10_E: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([90, 209, 248, 33]));

    /// ln(2)
    pub const LN_2: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([199, 236, 252, 33]));

    /// ln(10)
    pub const LN_10: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([203, 10, 246, 41]));
}

impl Bitstring32 {
    /// The radix or base of the internal representation.
    pub const RADIX: u32 = 10;

    /**
    The number of digits in base 10 that can be represented without loss of precision.

    This constant indicates the total count of significant decimal digits in the
    significand, regardless of the decimal point's position. For instance
    1234567 and 123.4567, both contain `DIGITS` digits.
    */
    pub const DIGITS: u32 = 7;

    /**
    [Machine epsilon] value.

    This is the difference between `1.0` and the next larger representable number.

    [Machine epsilon]: https://en.wikipedia.org/wiki/Machine_epsilon
    */
    pub const EPSILON: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([1, 0, 0, 34]));

    /// Smallest finite value.
    pub const MIN: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([255, 252, 227, 247]));

    /// Smallest positive normal value.
    pub const MIN_POSITIVE: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([1, 0, 0, 0]));

    /// Largest finite value.
    pub const MAX: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([255, 252, 227, 119]));

    /// Not a Number (NaN), with a zero payload.
    pub const NAN: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([0, 0, 0, 124]));

    /// Infinity (∞).
    pub const INFINITY: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([0, 0, 0, 120]));

    /// Negative infinity (−∞).
    pub const NEG_INFINITY: Self = Bitstring32(FixedBinaryBuf::from_le_bytes([0, 0, 0, 248]));
}

#[test]
#[cfg(test)]
fn consts_32() {
    use core::{
        f64::consts as f64const,
        str::FromStr,
    };

    // helper fn
    fn is_eq(a: Bitstring32, b: Bitstring32) {
        assert_eq!(a.as_le_bytes(), b.as_le_bytes());
    }
    // helper fn
    fn is_eq_f(a: Bitstring32, b: f64) {
        assert_eq!(
            a.to_string(),
            b.to_string()
                .chars()
                .take((Bitstring32::DIGITS + 1) as usize)
                .collect::<String>()
        );
    }

    is_eq(Bitstring32::ZERO, Bitstring32::from_str("0").unwrap());
    is_eq(Bitstring32::ONE, Bitstring32::from_str("1").unwrap());
    is_eq(Bitstring32::NEG_ONE, Bitstring32::from_str("-1").unwrap());

    is_eq_f(Bitstring32::PI, f64const::PI);
    is_eq_f(Bitstring32::FRAC_PI_2, f64const::FRAC_PI_2);
    is_eq_f(Bitstring32::FRAC_PI_3, f64const::FRAC_PI_3);
    is_eq_f(Bitstring32::FRAC_PI_6, f64const::FRAC_PI_6);
    is_eq_f(Bitstring32::FRAC_PI_8, f64const::FRAC_PI_8);
    is_eq_f(Bitstring32::FRAC_1_PI, f64const::FRAC_1_PI);
    is_eq_f(Bitstring32::FRAC_2_PI, f64const::FRAC_2_PI);
    is_eq_f(Bitstring32::FRAC_2_SQRT_PI, f64const::FRAC_2_SQRT_PI);
    is_eq_f(Bitstring32::SQRT_2, f64const::SQRT_2);
    is_eq_f(Bitstring32::FRAC_1_SQRT_2, f64const::FRAC_1_SQRT_2);
    is_eq_f(Bitstring32::E, f64const::E);
    is_eq_f(Bitstring32::LOG2_10, f64const::LOG2_10);
    is_eq_f(Bitstring32::LOG2_E, f64const::LOG2_E);
    is_eq_f(Bitstring32::LOG10_2, f64const::LOG10_2);
    is_eq_f(Bitstring32::LOG10_E, f64const::LOG10_E);
    is_eq_f(Bitstring32::LN_2, f64const::LN_2);
    is_eq_f(Bitstring32::LN_10, f64const::LN_10);

    // NOTE: 10e-6 according to https://en.wikipedia.org/wiki/Machine_epsilon#cite_note-2
    is_eq(
        Bitstring32::EPSILON,
        Bitstring32::from_str("0.00001").unwrap(),
    );
    is_eq(Bitstring32::MIN, Bitstring32::min());
    is_eq(Bitstring32::MIN_POSITIVE, Bitstring32::min_positive());
    is_eq(Bitstring32::MAX, Bitstring32::max());
    is_eq(Bitstring32::NAN, Bitstring32::from_str("nan").unwrap());
    is_eq(Bitstring32::INFINITY, Bitstring32::from_str("inf").unwrap());
    is_eq(
        Bitstring32::NEG_INFINITY,
        Bitstring32::from_str("-inf").unwrap(),
    );
}

impl Bitstring32 {
    /**
    Create a decimal from its representation as a byte array in little endian.

    This matches the internal byte representation of the decimal, regardless of the platform.
    */
    #[inline]
    pub const fn from_le_bytes(bytes: [u8; 4]) -> Self {
        Self(FixedBinaryBuf::from_le_bytes(bytes))
    }

    /**
    Create a decimal from its representation as a byte array in big endian.
    */
    #[inline]
    pub const fn from_be_bytes(bytes: [u8; 4]) -> Self {
        Self(FixedBinaryBuf::from_le_bytes([
            bytes[3], bytes[2], bytes[1], bytes[0],
        ]))
    }

    /**
    Return the memory representation of this decimal as a byte array in little-endian byte order.

    This matches the internal byte representation of the decimal, regardless of the platform.
    */
    #[inline]
    pub const fn as_le_bytes(&self) -> [u8; 4] {
        // Even on big-endian platforms we always encode numbers in little-endian order
        self.0.as_le_bytes()
    }

    /**
    Return the memory representation of this decimal as a byte array in big-endian
    (network) byte order.
    */
    #[inline]
    pub const fn to_be_bytes(&self) -> [u8; 4] {
        let b = self.0.as_le_bytes();
        [b[3], b[2], b[1], b[0]]
    }

    /**
    Create a decimal with the finite value zero.
    */
    pub fn zero() -> Self {
        Self::from(0u8)
    }

    /**
    Create a decimal with its maximum finite value.
    */
    pub fn max() -> Self {
        let mut buf = FixedBinaryBuf::ZERO;

        encode_max(&mut buf, false);

        Self(buf)
    }

    /**
    Create a decimal with its minimum finite value.
    */
    pub fn min() -> Self {
        let mut buf = FixedBinaryBuf::ZERO;

        encode_max(&mut buf, true);

        Self(buf)
    }

    /**
    Create a decimal with its minimum positive non-zero value.
    */
    pub fn min_positive() -> Self {
        let mut buf = FixedBinaryBuf::ZERO;

        encode_min(&mut buf, false);

        Self(buf)
    }
}

classify!(Bitstring32);

try_s2d!(ArrayTextBuf::<32> => Bitstring32);
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
