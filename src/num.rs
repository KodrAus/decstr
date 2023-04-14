use core::{
    fmt,
    ops::Index,
    str,
};

use crate::text::{
    FiniteParser,
    FixedSizeTextBuf,
    TextBuf,
    TextWriter,
};

/**
Generic binary integers.
*/
pub trait Integer {
    type Bytes: Index<usize, Output = u8>;

    fn try_from_ascii<I: Iterator<Item = u8>>(is_negative: bool, ascii: I) -> Option<Self>
    where
        Self: Sized;

    fn from_le_bytes<I: Iterator<Item = u8>>(bytes: I) -> Self;

    fn zero() -> Self
    where
        Self: Sized,
    {
        Self::from_i32(0)
    }

    fn from_i32(n: i32) -> Self;

    fn to_i32(&self) -> Option<i32>;

    fn is_negative(&self) -> bool;

    fn to_le_bytes(&self) -> Self::Bytes;

    fn to_fmt<W: fmt::Write>(&self, out: W) -> fmt::Result;

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
    type TextWriter: TextBuf + TextWriter + Default;
    type NanPayload: Integer;

    fn try_finite_from_ascii<I: Iterator<Item = u8>, E: Integer>(
        is_negative: bool,
        ascii: I,
        exponent: E,
    ) -> Option<Self>
    where
        Self: Sized;

    fn infinity(is_negative: bool) -> Self;

    fn nan(is_negative: bool, is_signaling: bool, payload: Self::NanPayload) -> Self;

    fn is_sign_negative(&self) -> bool;

    fn is_finite(&self) -> bool;

    fn is_infinite(&self) -> bool;

    fn is_nan(&self) -> bool;
    fn is_nan_signaling(&self) -> bool;
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
                    let bits = self.to_bits() & $nan_mask;

                    if bits == 0 {
                        None
                    } else {
                        Some(bits as $i)
                    }
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
        FixedSizeTextBuf<F32_BUF_SIZE>,
        F32_NAN_PAYLOAD_MASK,
        F32_SIGNALING_MASK
    ),
    (
        f64,
        i64,
        u64,
        FixedSizeTextBuf<F64_BUF_SIZE>,
        F64_NAN_PAYLOAD_MASK,
        F64_SIGNALING_MASK
    )
);

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
        parser.significand_is_negative();
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
    parser.begin_exponent();

    parser
        .parse_fmt(exponent.as_display())
        .expect("failed to parse exponent");

    let parsed = parser.end().expect("failed to finish parsing");

    // Convert the parsed value into a float
    str::from_utf8(parsed.finite_buf.get_ascii())
        .expect("non-UTF8")
        .parse()
        .ok()
}
