use core::{
    fmt,
    ops::Index,
    str,
};

use crate::text::{
    ArrayTextBuf,
    FiniteParser,
    TextBuf,
    TextWriter,
};

/**
Generic binary integers.
*/
pub trait Integer {
    /**
    The integer represented as a little-endian buffer.
    */
    type Bytes: Index<usize, Output = u8>;

    /**
    Parse the integer from ASCII digits.
    */
    fn try_from_ascii<I: Iterator<Item = u8>>(is_negative: bool, ascii: I) -> Option<Self>
    where
        Self: Sized;

    /**
    Convert from a stream of bytes in little-endian byte-order.
    */
    fn from_le_bytes<I: Iterator<Item = u8>>(bytes: I) -> Self;

    /**
    Get an instance of the value `0`.
    */
    fn zero() -> Self
    where
        Self: Sized,
    {
        Self::from_i32(0)
    }

    /**
    Convert from a 32-bit signed integer.
    */
    fn from_i32(n: i32) -> Self;

    /**
    Try convert into a 32-bit signed integer.
    */
    fn to_i32(&self) -> Option<i32>;

    /**
    Whether or not this integer is negative.
    */
    fn is_negative(&self) -> bool;

    /**
    Convert this integer into a little-endian buffer.
    */
    fn to_le_bytes(&self) -> Self::Bytes;

    /**
    Write this integer as ASCII with an optional leading sign.
    */
    fn to_fmt<W: fmt::Write>(&self, out: W) -> fmt::Result;

    /**
    An adapter that can display this integer using its `to_fmt` implementation.
    */
    fn as_display(&self) -> AsDisplay<&Self> {
        AsDisplay(self)
    }
}

pub struct AsDisplay<T>(T);

impl<'a, T: Integer> fmt::Display for AsDisplay<&'a T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.to_fmt(f)
    }
}

/**
Generic binary floating points.
*/
pub(crate) trait Float {
    /**
    A text writer that can buffer any valid instance of this number.
    */
    type TextWriter: TextBuf + TextWriter + Default;

    /**
    An integer that can represent any valid payload on this number.
    */
    type NanPayload: Integer;

    /**
    Parse the number using the same text format as decimals.
    */
    fn try_finite_from_ascii<I: Iterator<Item = u8>, E: Integer>(
        is_negative: bool,
        ascii: I,
        exponent: E,
    ) -> Option<Self>
    where
        Self: Sized;

    /**
    Get an instance of an infinity.
    */
    fn infinity(is_negative: bool) -> Self;

    /**
    Get an instance of a NaN with the given payload.
    */
    fn nan(is_negative: bool, is_signaling: bool, payload: Self::NanPayload) -> Self;

    /**
    Whether or not the number is negative.
    */
    fn is_sign_negative(&self) -> bool;

    /**
    Whether or not the number is finite.
    */
    fn is_finite(&self) -> bool;

    /**
    Whether or not the number is infinity.
    */
    fn is_infinite(&self) -> bool;

    /**
    Whether or not the number is NaN.
    */
    fn is_nan(&self) -> bool;

    /**
    Whether or not the number is sNaN.

    Note that IEEE 754 didn't originally specify how to interpret the signaling bit, so older
    MIPS architectures interpret qNaN and sNaN differently to more recent ones.
    */
    fn is_nan_signaling(&self) -> bool;

    /**
    Get the payload with a NaN if there is one.
    */
    fn nan_payload(&self) -> Option<Self::NanPayload>;
}

macro_rules! impl_binary_integer {
    ($(($i:ty, $bytes:ty)),*) => {
        $(
            impl Integer for $i {
                type Bytes = $bytes;

                fn try_from_ascii<I: Iterator<Item = u8>>(is_negative: bool, ascii: I) -> Option<Self> {
                    let mut i: $i = 0;

                    if is_negative {
                        for b in ascii {
                            i = i.checked_mul(10)?;
                            i = i.checked_sub((b - b'0') as $i)?;
                        }
                    } else {
                        for b in ascii {
                            i = i.checked_mul(10)?;
                            i = i.checked_add((b - b'0') as $i)?;
                        }
                    }

                    Some(i)
                }

                fn from_le_bytes<I: Iterator<Item = u8>>(bytes: I) -> Self {
                    let mut buf = [0; (<$i>::BITS / 8) as usize];

                    for (i, b) in bytes.enumerate() {
                        buf[i] = b;
                    }

                    Self::from_le_bytes(buf)
                }

                fn from_i32(exp: i32) -> Self {
                    exp as $i
                }

                fn to_i32(&self) -> Option<i32> {
                    (*self).try_into().ok()
                }

                fn is_negative(&self) -> bool {
                    <$i>::is_negative(*self)
                }

                fn to_le_bytes(&self) -> Self::Bytes {
                    <$i>::to_le_bytes(*self)
                }

                fn to_fmt<W: fmt::Write>(&self, mut out: W) -> fmt::Result {
                    write!(out, "{}", self)
                }
            }
        )*
    };
}

macro_rules! impl_binary_unsigned_integer {
    ($(($i:ty, $bytes:ty)),*) => {
        $(
            impl Integer for $i {
                type Bytes = $bytes;

                fn try_from_ascii<I: Iterator<Item = u8>>(is_negative: bool, ascii: I) -> Option<Self> {
                    let mut i: $i = 0;

                    if is_negative {
                        return None;
                    } else {
                        for b in ascii {
                            i = i.checked_mul(10)?;
                            i = i.checked_add((b - b'0') as $i)?;
                        }
                    }

                    Some(i)
                }

                fn from_le_bytes<I: Iterator<Item = u8>>(bytes: I) -> Self {
                    let mut buf = [0; (<$i>::BITS / 8) as usize];

                    for (i, b) in bytes.enumerate() {
                        buf[i] = b;
                    }

                    Self::from_le_bytes(buf)
                }

                fn from_i32(exp: i32) -> Self {
                    exp as $i
                }

                fn to_i32(&self) -> Option<i32> {
                    (*self).try_into().ok()
                }

                fn is_negative(&self) -> bool {
                    false
                }

                fn to_le_bytes(&self) -> Self::Bytes {
                    <$i>::to_le_bytes(*self)
                }

                fn to_fmt<W: fmt::Write>(&self, mut out: W) -> fmt::Result {
                    write!(out, "{}", self)
                }
            }
        )*
    };
}

impl_binary_integer!(
    (i8, [u8; 1]),
    (i16, [u8; 2]),
    (i32, [u8; 4]),
    (i64, [u8; 8]),
    (i128, [u8; 16])
);
impl_binary_unsigned_integer!(
    (u8, [u8; 1]),
    (u16, [u8; 2]),
    (u32, [u8; 4]),
    (u64, [u8; 8]),
    (u128, [u8; 16])
);

// 2f32.powi(23 + 1).log10().ceil() + 1f32
const F32_MAX_MANTISSA_DIGITS: usize = 9;
const F32_MAX_EXPONENT_DIGITS: usize = 3;
const F32_BUF_SIZE: usize = F32_MAX_MANTISSA_DIGITS + F32_MAX_EXPONENT_DIGITS + 4;

// The payload for a NaN is the significand bits, except for the most significant,
// which is used to identify signaling vs quiet NaNs
const F32_NAN_PAYLOAD_MASK: u32 = 0b0000_0000_0111_1111_1111_1111_1111_1111u32;
const F32_SIGNALING_MASK: u32 = 0b0000_0000_1000_0000_0000_0000_0000_0000u32;

// 2f64.powi(52 + 1).log10().ceil() + 1f64
const F64_MAX_MANTISSA_DIGITS: usize = 17;
const F64_MAX_EXPONENT_DIGITS: usize = 4;
const F64_BUF_SIZE: usize = F64_MAX_MANTISSA_DIGITS + F64_MAX_EXPONENT_DIGITS + 4;

// The payload for a NaN is the significand bits, except for the most significant,
// which is used to identify signaling vs quiet NaNs
const F64_NAN_PAYLOAD_MASK: u64 =
    0b0000_0000_0000_0111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111u64;
const F64_SIGNALING_MASK: u64 =
    0b0000_0000_0000_1000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000u64;

macro_rules! impl_binary_float {
    ($(($f:ty, $i:ty, $u:ty, $text_writer:ty, $nan_mask:ident, $signaling_mask:ident)),*) => {
        $(
            impl Float for $f {
                type TextWriter = $text_writer;

                type NanPayload = $i;

                fn try_finite_from_ascii<I: Iterator<Item = u8>, E: Integer>(
                        is_negative: bool,
                        ascii: I,
                        exponent: E,
                ) -> Option<Self> {
                    parse_ascii(is_negative, ascii, exponent)
                }

                fn infinity(is_negative: bool) -> Self {
                    if is_negative {
                        <$f>::NEG_INFINITY
                    } else {
                        <$f>::INFINITY
                    }
                }

                fn nan(is_negative: bool, is_signaling: bool, payload: Self::NanPayload) -> Self {
                    let nan = if is_signaling {
                        <$f>::NAN.to_bits() & !$signaling_mask
                    } else {
                        <$f>::NAN.to_bits()
                    };

                    let f = if payload == 0 {
                        <$f>::from_bits(nan)
                    } else {
                        <$f>::from_bits(nan | ((payload as $u) & $nan_mask))
                    };

                    if is_negative {
                        -f
                    } else {
                        f
                    }
                }

                fn is_sign_negative(&self) -> bool {
                    <$f>::is_sign_negative(*self)
                }

                fn is_finite(&self) -> bool {
                    <$f>::is_finite(*self)
                }

                fn is_infinite(&self) -> bool {
                    <$f>::is_infinite(*self)
                }

                fn is_nan(&self) -> bool {
                    <$f>::is_nan(*self)
                }

                fn is_nan_signaling(&self) -> bool {
                    <$f>::is_nan(*self) && (self.to_bits() & $signaling_mask == 0)
                }

                fn nan_payload(&self) -> Option<Self::NanPayload> {
                    // Rust doesn't guarantee any particular NaN payload for
                    // the constants `f32::NAN` and `f64::NAN`, so we just
                    // unconditionally ignore the payload. This means information
                    // could be lost encoding a binary floating point through a
                    // decimal, but NaN payloads on binary floating points have
                    // rather sketchy portability as it is.
                    None
                }
            }
        )*
    };
}

impl_binary_float!(
    (
        f32,
        i32,
        u32,
        ArrayTextBuf<F32_BUF_SIZE>,
        F32_NAN_PAYLOAD_MASK,
        F32_SIGNALING_MASK
    ),
    (
        f64,
        i64,
        u64,
        ArrayTextBuf<F64_BUF_SIZE>,
        F64_NAN_PAYLOAD_MASK,
        F64_SIGNALING_MASK
    )
);

/**
Parse a binary floating point number from text.
*/
fn parse_ascii<F: Float + str::FromStr>(
    is_negative: bool,
    digits: impl Iterator<Item = u8>,
    exponent: impl Integer,
) -> Option<F>
where
    F::Err: fmt::Debug,
{
    let mut parser = FiniteParser::begin(F::TextWriter::default());

    // Parse the significand
    if is_negative {
        parser.checked_significand_is_negative().ok()?;
    }

    let mut written = 0;
    for digit in digits.skip_while(|d| *d == b'0') {
        parser.checked_push_significand_digit(digit).ok()?;
        written += 1;
    }
    if written == 0 {
        parser.checked_push_significand_digit(b'0').ok()?;
    }

    // Parse the exponent
    parser.checked_begin_exponent().ok()?;

    parser
        .parse_fmt(exponent.as_display())
        .expect("failed to parse exponent");

    let parsed = parser.end().expect("failed to finish parsing");

    // Convert the parsed value into a float
    str::from_utf8(parsed.finite_buf.get_ascii())
        .ok()?
        .parse()
        .ok()
}
