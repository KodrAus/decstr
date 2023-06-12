use crate::{
    binary::{
        encode_max,
        encode_min,
        FixedBinaryBuf,
    },
    text::ArrayTextBuf,
};

/**
A 64bit decimal number.
*/
#[derive(Clone, Copy)]
pub struct Bitstring64(FixedBinaryBuf<8, i32>);

/**
Basic mathematical constants.
*/
impl Bitstring64 {
    /// 0
    pub const ZERO: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([0, 0, 0, 0, 0, 0, 56, 34]));

    /// 1
    pub const ONE: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([1, 0, 0, 0, 0, 0, 56, 34]));

    /// -1
    pub const NEG_ONE: Self =
        Bitstring64(FixedBinaryBuf::from_le_bytes([1, 0, 0, 0, 0, 0, 56, 162]));

    /// Archimedes' constant (π)
    pub const PI: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([
        187, 63, 59, 181, 174, 193, 252, 45,
    ]));

    /// The full circle constant (τ)
    ///
    /// Equal to 2π.
    pub const TAU: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([
        234, 230, 115, 216, 50, 43, 253, 57,
    ]));

    /// π/2
    pub const FRAC_PI_2: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([
        30, 107, 111, 154, 254, 240, 254, 37,
    ]));

    /// π/3
    pub const FRAC_PI_3: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([
        251, 234, 19, 237, 62, 71, 252, 37,
    ]));

    /// π/4
    pub const FRAC_PI_4: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([
        72, 238, 55, 142, 119, 203, 255, 33,
    ]));

    /// π/6
    pub const FRAC_PI_6: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([
        94, 121, 91, 191, 183, 163, 254, 33,
    ]));

    /// π/8
    pub const FRAC_PI_8: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([
        164, 123, 189, 192, 215, 186, 253, 33,
    ]));

    /// 1/π
    pub const FRAC_1_PI: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([
        154, 175, 226, 112, 98, 152, 253, 33,
    ]));

    /// 2/π
    pub const FRAC_2_PI: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([
        139, 158, 39, 127, 198, 54, 255, 33,
    ]));

    /// 2/sqrt(π)
    pub const FRAC_2_SQRT_PI: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([
        146, 110, 113, 78, 126, 168, 252, 37,
    ]));

    /// sqrt(2)
    pub const SQRT_2: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([
        91, 204, 39, 238, 68, 20, 254, 37,
    ]));

    /// 1/sqrt(2)
    pub const FRAC_1_SQRT_2: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([
        199, 170, 179, 184, 33, 135, 255, 33,
    ]));

    /// Euler's number (e)
    pub const E: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([
        69, 100, 233, 210, 66, 152, 255, 41,
    ]));

    /// log<sub>2</sub>(10)
    pub const LOG2_10: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([
        226, 61, 172, 133, 107, 161, 253, 45,
    ]));

    /// log<sub>2</sub>(e)
    pub const LOG2_E: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([
        237, 185, 1, 196, 214, 66, 254, 37,
    ]));

    /// log<sub>10</sub>(2)
    pub const LOG10_2: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([
        143, 140, 253, 105, 10, 129, 253, 33,
    ]));

    /// log<sub>10</sub>(e)
    pub const LOG10_E: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([
        81, 53, 182, 160, 86, 52, 254, 33,
    ]));

    /// ln(2)
    pub const LN_2: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([
        205, 102, 171, 200, 49, 59, 255, 33,
    ]));

    /// ln(10)
    pub const LN_10: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([
        69, 120, 170, 195, 178, 130, 253, 41,
    ]));
}

impl Bitstring64 {
    /// The radix or base of the internal representation.
    pub const RADIX: u32 = 10;

    /**
    The number of digits in base 10 that can be represented without loss of precision.

    This constant indicates the total count of significant decimal digits in the
    significand, regardless of the decimal point's position. For instance
    1234567 and 123.4567, both contain `DIGITS` digits.
    */
    pub const DIGITS: u32 = 16;

    /**
    [Machine epsilon] value.

    This is the difference between `1.0` and the next larger representable number.

    [Machine epsilon]: https://en.wikipedia.org/wiki/Machine_epsilon
    */
    pub const EPSILON: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([1, 0, 0, 0, 0, 0, 0, 34]));

    /// Smallest finite value.
    pub const MIN: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([
        255, 252, 243, 207, 63, 255, 248, 247,
    ]));

    /// Smallest positive normal value.
    pub const MIN_POSITIVE: Self =
        Bitstring64(FixedBinaryBuf::from_le_bytes([1, 0, 0, 0, 0, 0, 0, 0]));

    /// Largest finite value.
    pub const MAX: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([
        255, 252, 243, 207, 63, 255, 248, 119,
    ]));

    /// Not a Number (NaN), with a zero payload.
    pub const NAN: Self = Bitstring64(FixedBinaryBuf::from_le_bytes([0, 0, 0, 0, 0, 0, 0, 124]));

    /// Infinity (∞).
    pub const INFINITY: Self =
        Bitstring64(FixedBinaryBuf::from_le_bytes([0, 0, 0, 0, 0, 0, 0, 120]));

    /// Negative infinity (−∞).
    pub const NEG_INFINITY: Self =
        Bitstring64(FixedBinaryBuf::from_le_bytes([0, 0, 0, 0, 0, 0, 0, 248]));
}

#[test]
#[cfg(test)]
fn consts_64() {
    use core::str::FromStr;

    // helper fn
    fn is_eq(a: Bitstring64, b: Bitstring64) {
        assert_eq!(a.as_le_bytes(), b.as_le_bytes());
    }
    // helper fn
    fn is_eq_f(a: Bitstring64, s: &str) {
        assert_eq!(
            a.to_string(),
            s.chars()
                .take((Bitstring64::DIGITS + 1) as usize)
                .collect::<String>()
        );
    }

    is_eq(Bitstring64::ZERO, Bitstring64::from_str("0").unwrap());
    is_eq(Bitstring64::ONE, Bitstring64::from_str("1").unwrap());
    is_eq(Bitstring64::NEG_ONE, Bitstring64::from_str("-1").unwrap());

    // 37 char strings extracted from https://doc.rust-lang.org/stable/core/f64/consts/index.html
    const PI: &str = "3.14159265358979323846264338327950288";
    const TAU: &str = "6.28318530717958647692528676655900577";
    const FRAC_PI_2: &str = "1.57079632679489661923132169163975144";
    const FRAC_PI_3: &str = "1.04719755119659774615421446109316763";
    const FRAC_PI_4: &str = "0.785398163397448309615660845819875721";
    const FRAC_PI_6: &str = "0.52359877559829887307710723054658381";
    const FRAC_PI_8: &str = "0.39269908169872415480783042290993786";
    const FRAC_1_PI: &str = "0.318309886183790671537767526745028724";
    const FRAC_2_PI: &str = "0.636619772367581343075535053490057448";
    const FRAC_2_SQRT_PI: &str = "1.12837916709551257389615890312154517";
    const SQRT_2: &str = "1.41421356237309504880168872420969808";
    const FRAC_1_SQRT_2: &str = "0.707106781186547524400844362104849039";
    const E: &str = "2.71828182845904523536028747135266250";
    const LOG2_10: &str = "3.32192809488736234787031942948939018";
    const LOG2_E: &str = "1.44269504088896340735992468100189214";
    const LOG10_2: &str = "0.301029995663981195213738894724493027";
    const LOG10_E: &str = "0.434294481903251827651128918916605082";
    const LN_2: &str = "0.693147180559945309417232121458176568";
    const LN_10: &str = "2.30258509299404568401799145468436421";

    is_eq_f(Bitstring64::PI, PI);
    is_eq_f(Bitstring64::TAU, TAU);
    is_eq_f(Bitstring64::FRAC_PI_2, FRAC_PI_2);
    is_eq_f(Bitstring64::FRAC_PI_3, FRAC_PI_3);
    is_eq_f(Bitstring64::FRAC_PI_4, FRAC_PI_4);
    is_eq_f(Bitstring64::FRAC_PI_6, FRAC_PI_6);
    is_eq_f(Bitstring64::FRAC_PI_8, FRAC_PI_8);
    is_eq_f(Bitstring64::FRAC_1_PI, FRAC_1_PI);
    is_eq_f(Bitstring64::FRAC_2_PI, FRAC_2_PI);
    is_eq_f(Bitstring64::FRAC_2_SQRT_PI, FRAC_2_SQRT_PI);
    is_eq_f(Bitstring64::SQRT_2, SQRT_2);
    is_eq_f(Bitstring64::FRAC_1_SQRT_2, FRAC_1_SQRT_2);
    is_eq_f(Bitstring64::E, E);
    is_eq_f(Bitstring64::LOG2_10, LOG2_10);
    is_eq_f(Bitstring64::LOG2_E, LOG2_E);
    is_eq_f(Bitstring64::LOG10_2, LOG10_2);
    is_eq_f(Bitstring64::LOG10_E, LOG10_E);
    is_eq_f(Bitstring64::LN_2, LN_2);
    is_eq_f(Bitstring64::LN_10, LN_10);

    // NOTE: 10e-15 according to https://en.wikipedia.org/wiki/Machine_epsilon#cite_note-2
    is_eq(
        Bitstring64::EPSILON,
        Bitstring64::from_str("0.00000000000001").unwrap(),
    );
    is_eq(Bitstring64::MIN, Bitstring64::min());
    is_eq(Bitstring64::MIN_POSITIVE, Bitstring64::min_positive());
    is_eq(Bitstring64::MAX, Bitstring64::max());
    is_eq(Bitstring64::NAN, Bitstring64::from_str("nan").unwrap());
    is_eq(Bitstring64::INFINITY, Bitstring64::from_str("inf").unwrap());
    is_eq(
        Bitstring64::NEG_INFINITY,
        Bitstring64::from_str("-inf").unwrap(),
    );
}

impl Bitstring64 {
    /**
    Create a decimal from its representation as a byte array in little endian.

    This matches the internal byte representation of the decimal, regardless of the platform.
    */
    #[inline]
    pub const fn from_le_bytes(bytes: [u8; 8]) -> Self {
        Self(FixedBinaryBuf::from_le_bytes(bytes))
    }

    /**
    Create a decimal from its representation as a byte array in big endian.
    */
    #[inline]
    pub const fn from_be_bytes(bytes: [u8; 8]) -> Self {
        Self(FixedBinaryBuf::from_le_bytes([
            bytes[7], bytes[6], bytes[5], bytes[4], bytes[3], bytes[2], bytes[1], bytes[0],
        ]))
    }

    /**
    Return the memory representation of this decimal as a byte array in little-endian byte order.

    This matches the internal byte representation of the decimal, regardless of the platform.
    */
    #[inline]
    pub const fn as_le_bytes(&self) -> [u8; 8] {
        // Even on big-endian platforms we always encode numbers in little-endian order
        self.0.as_le_bytes()
    }

    /**
    Return the memory representation of this decimal as a byte array in big-endian
    (network) byte order.
    */
    #[inline]
    pub const fn to_be_bytes(&self) -> [u8; 8] {
        let b = self.0.as_le_bytes();
        [b[7], b[6], b[5], b[4], b[3], b[2], b[1], b[0]]
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

classify!(Bitstring64);

try_s2d!(ArrayTextBuf::<64> => Bitstring64);
d2s!(Bitstring64);

f2d!(f32 => from_f32 => Bitstring64);
try_f2d!(f64 => from_f64 => Bitstring64);

try_d2f!(Bitstring64 => to_f32 => f32);
try_d2f!(Bitstring64 => to_f64 => f64);

i2d!(i8 => from_i8 => Bitstring64);
i2d!(i16 => from_i16 => Bitstring64);
i2d!(i32 => from_i32 => Bitstring64);
try_i2d!(i64 => from_i64 => Bitstring64);
try_i2d!(i128 => from_i128 => Bitstring64);

try_d2i!(Bitstring64 => to_i8 => i8);
try_d2i!(Bitstring64 => to_i16 => i16);
try_d2i!(Bitstring64 => to_i32 => i32);
try_d2i!(Bitstring64 => to_i64 => i64);
try_d2i!(Bitstring64 => to_i128 => i128);

i2d!(u8 => from_u8 => Bitstring64);
i2d!(u16 => from_u16 => Bitstring64);
i2d!(u32 => from_u32 => Bitstring64);
try_i2d!(u64 => from_u64 => Bitstring64);
try_i2d!(u128 => from_u128 => Bitstring64);

try_d2i!(Bitstring64 => to_u8 => u8);
try_d2i!(Bitstring64 => to_u16 => u16);
try_d2i!(Bitstring64 => to_u32 => u32);
try_d2i!(Bitstring64 => to_u64 => u64);
try_d2i!(Bitstring64 => to_u128 => u128);
