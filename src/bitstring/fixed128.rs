use crate::{
    binary::{
        encode_max,
        encode_min,
        FixedBinaryBuf,
    },
    text::ArrayTextBuf,
};

/**
A [128bit decimal number](https://en.wikipedia.org/wiki/Decimal128_floating-point_format).
*/
#[derive(Clone, Copy)]
pub struct Bitstring128(FixedBinaryBuf<16, i32>);

/**
Basic mathematical constants.
*/
impl Bitstring128 {
    /// 0
    pub const ZERO: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 34,
    ]));

    /// 1
    pub const ONE: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 34,
    ]));

    /// -1
    pub const NEG_ONE: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 162,
    ]));

    /// Archimedes' constant (π)
    pub const PI: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        130, 230, 181, 218, 208, 98, 226, 180, 251, 179, 83, 235, 26, 204, 255, 45,
    ]));

    /// The full circle constant (τ)
    ///
    /// Equal to 2π.
    pub const TAU: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        5, 100, 107, 190, 90, 173, 218, 169, 110, 62, 135, 45, 179, 210, 255, 57,
    ]));

    /// π/2
    pub const FRAC_PI_2: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        209, 231, 188, 113, 104, 49, 101, 236, 177, 246, 166, 233, 15, 239, 255, 37,
    ]));

    /// π/3
    pub const FRAC_PI_3: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        231, 236, 16, 38, 69, 212, 24, 191, 175, 62, 209, 238, 115, 196, 255, 37,
    ]));

    /// π/4
    pub const FRAC_PI_4: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        125, 254, 208, 36, 216, 21, 39, 134, 228, 126, 227, 120, 183, 252, 255, 33,
    ]));

    /// π/6
    pub const FRAC_PI_6: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        171, 26, 11, 211, 33, 119, 244, 229, 149, 183, 245, 123, 59, 234, 255, 33,
    ]));

    /// π/8
    pub const FRAC_PI_8: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        189, 191, 34, 34, 15, 13, 83, 67, 186, 215, 11, 124, 173, 219, 255, 33,
    ]));

    /// 1/π
    pub const FRAC_1_PI: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        40, 20, 111, 234, 249, 183, 198, 173, 249, 42, 14, 39, 134, 217, 255, 33,
    ]));

    /// 2/π
    pub const FRAC_2_PI: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        87, 104, 56, 69, 173, 117, 12, 183, 232, 121, 242, 103, 108, 243, 255, 33,
    ]));

    /// 2/sqrt(π)
    pub const FRAC_2_SQRT_PI: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        197, 134, 210, 24, 54, 30, 207, 43, 233, 22, 231, 228, 135, 202, 255, 37,
    ]));

    /// sqrt(2)
    pub const SQRT_2: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        94, 39, 68, 186, 211, 13, 32, 177, 197, 124, 226, 78, 68, 225, 255, 37,
    ]));

    /// 1/sqrt(2)
    pub const FRAC_1_SQRT_2: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        47, 18, 34, 30, 147, 0, 146, 122, 172, 58, 139, 27, 114, 248, 255, 33,
    ]));

    /// Euler's number (e)
    pub const E: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        98, 75, 23, 231, 90, 224, 213, 84, 68, 150, 46, 45, 132, 249, 255, 41,
    ]));

    /// log<sub>2</sub>(10)
    pub const LOG2_10: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        154, 61, 153, 98, 102, 124, 28, 39, 222, 195, 90, 184, 22, 218, 255, 45,
    ]));

    /// log<sub>2</sub>(e)
    pub const LOG2_E: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        30, 5, 176, 48, 171, 217, 29, 216, 158, 27, 64, 108, 45, 228, 255, 37,
    ]));

    /// log<sub>10</sub>(2)
    pub const LOG10_2: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        59, 146, 238, 33, 238, 19, 109, 243, 200, 216, 159, 166, 16, 216, 255, 33,
    ]));

    /// log<sub>10</sub>(e)
    pub const LOG10_E: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        5, 115, 238, 11, 42, 81, 183, 28, 85, 99, 11, 106, 69, 227, 255, 33,
    ]));

    /// ln(2)
    pub const LN_2: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        246, 96, 25, 138, 76, 23, 38, 214, 108, 182, 138, 28, 179, 243, 255, 33,
    ]));

    /// ln(10)
    pub const LN_10: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        228, 41, 77, 229, 39, 23, 40, 93, 132, 167, 58, 44, 43, 216, 255, 41,
    ]));
}

impl Bitstring128 {
    /// The radix or base of the internal representation.
    pub const RADIX: u32 = 10;

    /**
    The number of digits in base 10 that can be represented without loss of precision.

    This constant indicates the total count of significant decimal digits in the
    significand, regardless of the decimal point's position. For instance
    1234567 and 123.4567, both contain `DIGITS` digits.
    */
    pub const DIGITS: u32 = 34;

    /**
    [Machine epsilon] value.

    This is the difference between `1.0` and the next larger representable number.

    [Machine epsilon]: https://en.wikipedia.org/wiki/Machine_epsilon
    */
    pub const EPSILON: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 34,
    ]));

    /// Smallest finite value.
    pub const MIN: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        255, 252, 243, 207, 63, 255, 252, 243, 207, 63, 255, 252, 243, 143, 255, 247,
    ]));

    /// Smallest positive normal value.
    pub const MIN_POSITIVE: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]));

    /// Largest finite value.
    pub const MAX: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        255, 252, 243, 207, 63, 255, 252, 243, 207, 63, 255, 252, 243, 143, 255, 119,
    ]));

    /// Not a Number (NaN), with a zero payload.
    pub const NAN: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 124,
    ]));

    /// Infinity (∞).
    pub const INFINITY: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 120,
    ]));

    /// Negative infinity (−∞).
    pub const NEG_INFINITY: Self = Bitstring128(FixedBinaryBuf::from_le_bytes([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 248,
    ]));
}

#[test]
#[cfg(test)]
fn consts_128() {
    use core::str::FromStr;

    // helper fn
    fn is_eq(a: Bitstring128, b: Bitstring128) {
        assert_eq!(a.as_le_bytes(), b.as_le_bytes());
    }
    // helper fn
    fn is_eq_f(a: Bitstring128, s: &str) {
        assert_eq!(
            a.to_string(),
            s.chars()
                .take((Bitstring128::DIGITS + 1) as usize)
                .collect::<String>()
        );
    }

    is_eq(Bitstring128::ZERO, Bitstring128::from_str("0").unwrap());
    is_eq(Bitstring128::ONE, Bitstring128::from_str("1").unwrap());
    is_eq(Bitstring128::NEG_ONE, Bitstring128::from_str("-1").unwrap());

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

    is_eq_f(Bitstring128::PI, PI);
    is_eq_f(Bitstring128::TAU, TAU);
    is_eq_f(Bitstring128::FRAC_PI_2, FRAC_PI_2);
    is_eq_f(Bitstring128::FRAC_PI_3, FRAC_PI_3);
    is_eq_f(Bitstring128::FRAC_PI_4, FRAC_PI_4);
    is_eq_f(Bitstring128::FRAC_PI_6, FRAC_PI_6);
    is_eq_f(Bitstring128::FRAC_PI_8, FRAC_PI_8);
    is_eq_f(Bitstring128::FRAC_1_PI, FRAC_1_PI);
    is_eq_f(Bitstring128::FRAC_2_PI, FRAC_2_PI);
    is_eq_f(Bitstring128::FRAC_2_SQRT_PI, FRAC_2_SQRT_PI);
    is_eq_f(Bitstring128::SQRT_2, SQRT_2);
    is_eq_f(Bitstring128::FRAC_1_SQRT_2, FRAC_1_SQRT_2);
    is_eq_f(Bitstring128::E, E);
    is_eq_f(Bitstring128::LOG2_10, LOG2_10);
    is_eq_f(Bitstring128::LOG2_E, LOG2_E);
    is_eq_f(Bitstring128::LOG10_2, LOG10_2);
    is_eq_f(Bitstring128::LOG10_E, LOG10_E);
    is_eq_f(Bitstring128::LN_2, LN_2);
    is_eq_f(Bitstring128::LN_10, LN_10);

    // NOTE: 10e-33 according to https://en.wikipedia.org/wiki/Machine_epsilon#cite_note-2
    is_eq(
        Bitstring128::EPSILON,
        Bitstring128::from_str("0.00000000000000000000000000000001").unwrap(),
    );
    is_eq(Bitstring128::MIN, Bitstring128::min());
    is_eq(Bitstring128::MIN_POSITIVE, Bitstring128::min_positive());
    is_eq(Bitstring128::MAX, Bitstring128::max());
    is_eq(Bitstring128::NAN, Bitstring128::from_str("nan").unwrap());
    is_eq(
        Bitstring128::INFINITY,
        Bitstring128::from_str("inf").unwrap(),
    );
    is_eq(
        Bitstring128::NEG_INFINITY,
        Bitstring128::from_str("-inf").unwrap(),
    );
}

impl Bitstring128 {
    /**
    Create a decimal from its representation as a byte array in little endian.

    This matches the internal byte representation of the decimal, regardless of the platform.
    */
    #[inline]
    pub const fn from_le_bytes(bytes: [u8; 16]) -> Self {
        Self(FixedBinaryBuf::from_le_bytes(bytes))
    }

    /**
    Create a decimal from its representation as a byte array in big endian.
    */
    #[inline]
    pub const fn from_be_bytes(bytes: [u8; 16]) -> Self {
        Self(FixedBinaryBuf::from_le_bytes([
            bytes[15], bytes[14], bytes[13], bytes[12], bytes[11], bytes[10], bytes[9], bytes[8],
            bytes[7], bytes[6], bytes[5], bytes[4], bytes[3], bytes[2], bytes[1], bytes[0],
        ]))
    }

    /**
    Return the memory representation of this decimal as a byte array in little-endian byte order.

    This matches the internal byte representation of the decimal, regardless of the platform.
    */
    #[inline]
    pub const fn as_le_bytes(&self) -> &[u8; 16] {
        // Even on big-endian platforms we always encode numbers in little-endian order
        self.0.as_le_bytes()
    }

    /**
    Return the memory representation of this decimal as a byte array in big-endian
    (network) byte order.
    */
    #[inline]
    pub const fn to_be_bytes(&self) -> [u8; 16] {
        let b = self.0.as_le_bytes();
        [
            b[15], b[14], b[13], b[12], b[11], b[10], b[9], b[8], b[7], b[6], b[5], b[4], b[3],
            b[2], b[1], b[0],
        ]
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

classify!(Bitstring128);

try_s2d!(ArrayTextBuf::<128> => Bitstring128);
d2s!(Bitstring128);

f2d!(f32 => from_f32 => Bitstring128);
f2d!(f64 => from_f64 => Bitstring128);

try_d2f!(Bitstring128 => to_f32 => f32);
try_d2f!(Bitstring128 => to_f64 => f64);

i2d!(i8 => from_i8 => Bitstring128);
i2d!(i16 => from_i16 => Bitstring128);
i2d!(i32 => from_i32 => Bitstring128);
i2d!(i64 => from_i64 => Bitstring128);
try_i2d!(i128 => from_i128 => Bitstring128);

try_d2i!(Bitstring128 => to_i8 => i8);
try_d2i!(Bitstring128 => to_i16 => i16);
try_d2i!(Bitstring128 => to_i32 => i32);
try_d2i!(Bitstring128 => to_i64 => i64);
try_d2i!(Bitstring128 => to_i128 => i128);

i2d!(u8 => from_u8 => Bitstring128);
i2d!(u16 => from_u16 => Bitstring128);
i2d!(u32 => from_u32 => Bitstring128);
i2d!(u64 => from_u64 => Bitstring128);
try_i2d!(u128 => from_u128 => Bitstring128);

try_d2i!(Bitstring128 => to_u8 => u8);
try_d2i!(Bitstring128 => to_u16 => u16);
try_d2i!(Bitstring128 => to_u32 => u32);
try_d2i!(Bitstring128 => to_u64 => u64);
try_d2i!(Bitstring128 => to_u128 => u128);
