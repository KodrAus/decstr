/*!
A format for exchanging arbitrary precision [decimal floating-point](https://en.wikipedia.org/wiki/Decimal_floating_point) numbers.

This library converts text-based numbers like `-123.456e7` into bitstrings like `01010110_10001110_10010010_10100010`.

Specifically, it implements support for an **[IEEE 754](https://en.wikipedia.org/wiki/IEEE_754) compatible
[decimal floating-point](https://en.wikipedia.org/wiki/Decimal_floating_point) bitstring,
using [densely-packed-decimal](https://en.wikipedia.org/wiki/Densely_packed_decimal) encoding, in
[little-endian byteorder](https://en.wikipedia.org/wiki/Endianness)**. Along with the bitstring encoding
there is an equivalent text-based one that can represent all the same values. High-level types that convert
between these two formats and standard Rust numeric types are provided.

# Why decimal bitstrings?

Decimal bitstrings are a nice way to encode and exchange numbers between systems.

Decimal bitstrings can be better for exchanging numbers than text because:

- They're more compact than text. Instead of encoding 1 digit per byte (8 bits), you get 3 digits per 10 bits.
- It's quick to tell if a number is positive, negative, whole, infinity, or NaN. You don't need to reparse the number.

Decimal bitstrings can be better for exchanging numbers than binary bitstrings because:

- They're a fairly direct translation from ASCII. You don't need arbitrary-precision arithmetic to encode a human-readable
number into a decimal bitstring.
- They can exactly encode base-10 numbers, which is the base most modern number systems use.
- They're a newer standard, so they avoid some ambiguities around NaN payloads and signaling that affect the portability of binary bitstrings.

# Features and limitations

This library _only_ does conversions between Rust's primitive number types, numbers encoded as text, and decimal bitstrings. It's
not an implementation of decimal arithmetic. It also doesn't do rounding. If a number can't be encoded in a decimal bitstring of a given width
then you'll get `None`s instead of infinities or rounded values.

This library does support very high precision in no-std, and can work with arbitrary precision when the `arbitrary-precision` feature is enabled.
*/

#![cfg_attr(not(any(feature = "std", test)), no_std)]

extern crate core;

/*
If you're exploring the source, there are 3 root modules to look at:

- `text`: Implements the text-based format. This module is a no-surprises parser that produces
ranges that cover features of the number, such as the sign, integer digits, decimal point, exponent.
- `binary`: Implements the IEEE 754 compatible decimal floating-point bitstring encoding. This module
is where you'll find the densely-packed-decimal encoding and arbitrary-precision arithmetic for the
exponent. There's some bit-twiddling involved to encode the bits for a number across multiple bytes,
but it's all explained along the way.
- `convert`: Combines the `text` and `binary` modules to convert between strings and Rust primitive
numbers and encoded bitstrings.

There is no special handling for decimal numbers of specific precisions. This is a trade-off between
simplicity and performance. The same implementation handles encoding decimal32 up to decimal256 and beyond.
*/

#[macro_use]
mod error;

mod binary;
mod convert;
mod num;
mod text;

use crate::{
    binary::{
        is_finite,
        is_infinite,
        is_nan,
        is_quiet_nan,
        is_sign_negative,
        is_signaling_nan,
        BinaryBuf,
        DynamicBinaryBuf,
    },
    text::{
        FixedSizeTextBuf,
        ParsedDecimal,
        TextBuf,
    },
};
use core::fmt;

pub use crate::error::*;

const UP_TO_160_BYTE_BUF: usize = 20;
const UP_TO_160_TEXT_BUF: usize = 128;

/**
A dynamically sized decimal number with enough precision to fit any Rust primitive number.
*/
pub struct Bitstring(DynamicBinaryBuf<UP_TO_160_BYTE_BUF>);

impl Bitstring {
    pub fn try_parse_str(n: &str) -> Result<Self, Error> {
        Ok(Bitstring(convert::decimal_from_str(n)?))
    }

    pub fn try_parse(n: impl fmt::Display) -> Result<Self, Error> {
        Ok(Bitstring(convert::decimal_from_fmt(
            n,
            FixedSizeTextBuf::<UP_TO_160_TEXT_BUF>::default(),
        )?))
    }

    pub fn try_from_le_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() == 0 || bytes.len() % 4 != 0 {
            Err(OverflowError::exact_size_mismatch(
                bytes.len(),
                bytes.len() + 4 - (bytes.len() % 4),
                "decimals must be a multiple of 32 bits (4 bytes)",
            ))?;
        }

        let mut buf = DynamicBinaryBuf::try_with_exactly_storage_width_bytes(bytes.len())?;

        buf.bytes_mut().copy_from_slice(bytes);

        Ok(Bitstring(buf))
    }

    pub fn from_f32(f: f32) -> Self {
        Bitstring(convert::decimal_from_binary_float(f).expect("always fits"))
    }

    pub fn from_f64(f: f64) -> Self {
        Bitstring(convert::decimal_from_binary_float(f).expect("always fits"))
    }

    pub fn from_i8(i: i8) -> Self {
        Bitstring(convert::decimal_from_int(i).expect("always fits"))
    }

    pub fn from_i16(i: i16) -> Self {
        Bitstring(convert::decimal_from_int(i).expect("always fits"))
    }

    pub fn from_i32(i: i32) -> Self {
        Bitstring(convert::decimal_from_int(i).expect("always fits"))
    }

    pub fn from_i64(i: i64) -> Self {
        Bitstring(convert::decimal_from_int(i).expect("always fits"))
    }

    pub fn from_i128(i: i128) -> Self {
        Bitstring(convert::decimal_from_int(i).expect("always fits"))
    }

    pub fn from_u8(i: u8) -> Self {
        Bitstring(convert::decimal_from_int(i).expect("always fits"))
    }

    pub fn from_u16(i: u16) -> Self {
        Bitstring(convert::decimal_from_int(i).expect("always fits"))
    }

    pub fn from_u32(i: u32) -> Self {
        Bitstring(convert::decimal_from_int(i).expect("always fits"))
    }

    pub fn from_u64(i: u64) -> Self {
        Bitstring(convert::decimal_from_int(i).expect("always fits"))
    }

    pub fn from_u128(i: u128) -> Self {
        Bitstring(convert::decimal_from_int(i).expect("always fits"))
    }

    pub fn to_i8(&self) -> Option<i8> {
        convert::decimal_to_int(&self.0)
    }

    pub fn to_i16(&self) -> Option<i16> {
        convert::decimal_to_int(&self.0)
    }

    pub fn to_i32(&self) -> Option<i32> {
        convert::decimal_to_int(&self.0)
    }

    pub fn to_i64(&self) -> Option<i64> {
        convert::decimal_to_int(&self.0)
    }

    pub fn to_i128(&self) -> Option<i128> {
        convert::decimal_to_int(&self.0)
    }

    pub fn to_u8(&self) -> Option<u8> {
        convert::decimal_to_int(&self.0)
    }

    pub fn to_u16(&self) -> Option<u16> {
        convert::decimal_to_int(&self.0)
    }

    pub fn to_u32(&self) -> Option<u32> {
        convert::decimal_to_int(&self.0)
    }

    pub fn to_u64(&self) -> Option<u64> {
        convert::decimal_to_int(&self.0)
    }

    pub fn to_u128(&self) -> Option<u128> {
        convert::decimal_to_int(&self.0)
    }

    pub fn to_f32(&self) -> Option<f32> {
        convert::decimal_to_binary_float(&self.0)
    }

    pub fn to_f64(&self) -> Option<f64> {
        convert::decimal_to_binary_float(&self.0)
    }

    pub fn as_le_bytes(&self) -> &[u8] {
        // Even on big-endian platforms we always encode numbers in little-endian order
        self.0.bytes()
    }

    pub fn is_sign_negative(&self) -> bool {
        is_sign_negative(&self.0)
    }

    pub fn is_finite(&self) -> bool {
        is_finite(&self.0)
    }

    pub fn is_infinite(&self) -> bool {
        is_infinite(&self.0)
    }

    pub fn is_nan(&self) -> bool {
        is_nan(&self.0)
    }

    pub fn is_quiet_nan(&self) -> bool {
        is_quiet_nan(&self.0)
    }

    pub fn is_signaling_nan(&self) -> bool {
        is_signaling_nan(&self.0)
    }
}

impl fmt::Debug for Bitstring {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        convert::decimal_to_fmt(&self.0, f)
    }
}

impl fmt::Display for Bitstring {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        convert::decimal_to_fmt(&self.0, f)
    }
}

impl From<u8> for Bitstring {
    fn from(i: u8) -> Bitstring {
        Bitstring::from_u8(i)
    }
}

impl From<u16> for Bitstring {
    fn from(i: u16) -> Bitstring {
        Bitstring::from_u16(i)
    }
}

impl From<u32> for Bitstring {
    fn from(i: u32) -> Bitstring {
        Bitstring::from_u32(i)
    }
}

impl From<u64> for Bitstring {
    fn from(i: u64) -> Bitstring {
        Bitstring::from_u64(i)
    }
}

impl From<u128> for Bitstring {
    fn from(i: u128) -> Bitstring {
        Bitstring::from_u128(i)
    }
}

impl From<i8> for Bitstring {
    fn from(i: i8) -> Bitstring {
        Bitstring::from_i8(i)
    }
}

impl From<i16> for Bitstring {
    fn from(i: i16) -> Bitstring {
        Bitstring::from_i16(i)
    }
}

impl From<i32> for Bitstring {
    fn from(i: i32) -> Bitstring {
        Bitstring::from_i32(i)
    }
}

impl From<i64> for Bitstring {
    fn from(i: i64) -> Bitstring {
        Bitstring::from_i64(i)
    }
}

impl From<i128> for Bitstring {
    fn from(i: i128) -> Bitstring {
        Bitstring::from_i128(i)
    }
}

impl From<f32> for Bitstring {
    fn from(i: f32) -> Bitstring {
        Bitstring::from_f32(i)
    }
}

impl From<f64> for Bitstring {
    fn from(i: f64) -> Bitstring {
        Bitstring::from_f64(i)
    }
}

/**
An arbitrary precision decimal number.
*/
#[cfg(feature = "arbitrary-precision")]
pub struct BigBitstring(binary::ArbitrarySizedBinaryBuf);

#[cfg(feature = "arbitrary-precision")]
impl BigBitstring {
    pub fn from_str(n: &str) -> Result<Self, Error> {
        Ok(BigBitstring(convert::decimal_from_str(n)?))
    }

    pub fn from_f64(n: f64) -> Self {
        BigBitstring(convert::decimal_from_f64(n).expect("always fits"))
    }

    pub fn from_i64(n: i64) -> Self {
        BigBitstring(convert::decimal_from_i64(n).expect("always fits"))
    }

    pub fn as_le_bytes(&self) -> &[u8] {
        // Even on big-endian platforms we always encode numbers in little-endian order
        self.0.bytes()
    }

    pub fn is_sign_negative(&self) -> bool {
        is_sign_negative(&self.0)
    }

    pub fn is_finite(&self) -> bool {
        is_finite(&self.0)
    }

    pub fn is_infinite(&self) -> bool {
        is_infinite(&self.0)
    }

    pub fn is_nan(&self) -> bool {
        is_nan(&self.0)
    }

    pub fn is_quiet_nan(&self) -> bool {
        is_quiet_nan(&self.0)
    }

    pub fn is_signaling_nan(&self) -> bool {
        is_signaling_nan(&self.0)
    }
}

#[cfg(feature = "arbitrary-precision")]
impl fmt::Display for BigBitstring {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        convert::decimal_to_fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn digits(buf: &mut String, n: usize) {
        for i in 0..n {
            buf.push(((i % 9) as u8 + 1 + b'0') as char);
        }
    }

    pub(crate) fn bitstr(b: &[u8]) -> String {
        use core::fmt::Write;

        let mut s = String::new();

        for b in b {
            if s.len() > 0 {
                s.write_char('_').expect("infallible string write");
            }

            write!(&mut s, "{:>08b}", b).expect("infallible string write");
        }

        s
    }

    fn nan32(payload: u32) -> f32 {
        f32::from_bits(f32::NAN.to_bits() | (payload & 0x7fffff))
    }

    fn snan32(payload: u32) -> f32 {
        f32::from_bits(nan32(payload).to_bits() & !0x800000)
    }

    fn nan64(payload: u64) -> f64 {
        f64::from_bits(f64::NAN.to_bits() | (payload & 0x7ffffffffffff))
    }

    fn snan64(payload: u64) -> f64 {
        f64::from_bits(nan64(payload).to_bits() & !0x8000000000000)
    }

    #[test]
    fn is_sign_negative() {
        for (f, is_negative) in [
            ("0", false),
            ("-0", true),
            ("123", false),
            ("-123", true),
            ("inf", false),
            ("-inf", true),
            ("nan", false),
            ("-nan", true),
            ("snan", false),
            ("-snan", true),
        ] {
            let d = Bitstring::try_parse_str(f).expect("failed to parse decimal");

            assert_eq!(is_negative, d.is_sign_negative(), "{}", f);
        }
    }

    #[test]
    fn is_finite() {
        for (f, is_finite) in [
            ("0", true),
            ("-0", true),
            ("123", true),
            ("-123", true),
            ("inf", false),
            ("-inf", false),
            ("nan", false),
            ("-nan", false),
            ("snan", false),
            ("-snan", false),
        ] {
            let d = Bitstring::try_parse_str(f).expect("failed to parse decimal");

            assert_eq!(is_finite, d.is_finite(), "{}", f);
        }
    }

    #[test]
    fn is_infinite() {
        for (f, is_infinite) in [
            ("123", false),
            ("-123", false),
            ("inf", true),
            ("-inf", true),
            ("nan", false),
            ("snan", false),
        ] {
            let d = Bitstring::try_parse_str(f).expect("failed to parse decimal");

            assert_eq!(is_infinite, d.is_infinite(), "{}", f);
        }
    }

    #[test]
    fn is_nan() {
        for (f, is_nan) in [
            ("123", false),
            ("-123", false),
            ("inf", false),
            ("-inf", false),
            ("nan", true),
            ("-nan", true),
            ("snan", true),
            ("-snan", true),
        ] {
            let d = Bitstring::try_parse_str(f).expect("failed to parse decimal");

            assert_eq!(is_nan, d.is_nan(), "{}", f);
        }
    }

    #[test]
    fn is_quiet_nan() {
        for (f, is_nan) in [
            ("123", false),
            ("-123", false),
            ("inf", false),
            ("-inf", false),
            ("nan", true),
            ("-nan", true),
            ("snan", false),
            ("-snan", false),
        ] {
            let d = Bitstring::try_parse_str(f).expect("failed to parse decimal");

            assert_eq!(is_nan, d.is_quiet_nan(), "{}", f);
        }
    }

    #[test]
    fn is_signaling_nan() {
        for (f, is_nan) in [
            ("123", false),
            ("-123", false),
            ("inf", false),
            ("-inf", false),
            ("nan", false),
            ("-nan", false),
            ("snan", true),
            ("-snan", true),
        ] {
            let d = Bitstring::try_parse_str(f).expect("failed to parse decimal");

            assert_eq!(is_nan, d.is_signaling_nan(), "{}", f);
        }
    }

    #[test]
    fn decimal_roundtrip_i128() {
        for i in [0i128, 42i128, i128::MIN, i128::MAX] {
            let d = Bitstring::from_i128(i);
            let di = d.to_i128().unwrap();

            assert_eq!(i, di);
        }
    }

    #[test]
    fn decimal_roundtrip_u128() {
        for i in [0u128, 42u128, u128::MAX] {
            let d = Bitstring::from_u128(i);
            let di = d.to_u128().unwrap();

            assert_eq!(i, di);
        }
    }

    #[test]
    fn decimal_to_int_with_exponent() {
        for (f, i) in [("17e1", 170i32), ("4e7", 40000000i32), ("170e-1", 17i32)] {
            let d = Bitstring::try_parse_str(f).unwrap();

            assert_eq!(i, d.to_i32().unwrap(), "{}", f);
        }
    }

    #[test]
    fn err_decimal_to_int_exponent_overflow() {
        for f in ["4e618", "17e-1", "1e-1"] {
            let d = Bitstring::try_parse_str(f).unwrap();

            assert!(d.to_i32().is_none(), "{}", f);
        }
    }

    #[test]
    fn decimal_roundtrip_f32() {
        for f in [
            0.0f32,
            17.05e2f32,
            f32::MIN,
            f32::MAX,
            f32::INFINITY,
            f32::NEG_INFINITY,
        ] {
            let d = Bitstring::from_f32(f);
            let df = d.to_f32().unwrap();

            assert_eq!(f, df);
        }
    }

    #[test]
    fn decimal_roundtrip_f32_nan() {
        for f in [nan32(0), nan32(42), snan32(0), snan32(42)] {
            let d = Bitstring::from_f32(f);
            let df = d.to_f32().unwrap();

            assert_eq!(f.to_bits(), df.to_bits());
        }
    }

    #[test]
    fn err_decimal_to_f32_overflow() {
        let d = Bitstring::try_parse_str("1e106").unwrap();

        assert!(d.to_f32().is_none());
    }

    #[test]
    fn decimal_roundtrip_f64() {
        for f in [
            0.0f64,
            17.05e2f64,
            f64::MIN,
            f64::MAX,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ] {
            let d = Bitstring::from_f64(f);
            let df = d.to_f64().unwrap();

            assert_eq!(f, df);
        }
    }

    #[test]
    fn decimal_roundtrip_f64_nan() {
        for f in [nan64(0), nan64(42), snan64(0), snan64(42)] {
            let d = Bitstring::from_f64(f);
            let df = d.to_f64().unwrap();

            assert_eq!(f.to_bits(), df.to_bits());
        }
    }

    #[test]
    fn err_decimal_to_f64_overflow() {
        let d = Bitstring::try_parse_str("1e4513").unwrap();

        assert!(d.to_f64().is_none());
    }

    #[test]
    fn decimal_roundtrip_str() {
        for f in [
            "0",
            "-0",
            "0.0",
            "-0.0",
            "435",
            "-435",
            "547473436755",
            "-547473436755",
            "354.55",
            "-354.55",
            "3546.8764256732",
            "-3546.8764256732",
            "0.00012532",
            "-0.00012532",
            "0e1",
            "120e2",
            "-120e2",
            "123e456",
            "-123e456",
            "123e-3",
            "-123e-3",
            "1.2354e-7",
            "-1.2354e-7",
            "1e96",
            "1e-95",
            "1e384",
            "1e-383",
            "1e6144",
            "1e-1643",
            "nan",
            "nan(123)",
            "inf",
            "-inf",
            "snan",
            "snan(123)",
        ] {
            println!("f: {}", f);

            let d = Bitstring::try_parse_str(f).expect("failed to parse decimal");

            // Ensure bitstrings roundtrip through from_le_bytes and to_le_bytes
            assert_eq!(
                bitstr(d.as_le_bytes()),
                bitstr(
                    Bitstring::try_from_le_bytes(d.as_le_bytes())
                        .expect("failed to convert bytes to decimal")
                        .as_le_bytes()
                ),
                "{}",
                f
            );

            let s = d.to_string();

            println!("s: {}", s);

            // Ensure bitstrings roundtrip through to_string and try_parse_str
            assert_eq!(
                bitstr(d.as_le_bytes()),
                bitstr(
                    Bitstring::try_parse_str(&s)
                        .expect("failed to parse decimal")
                        .as_le_bytes()
                ),
                "{} -> {}",
                f,
                s
            );
        }
    }

    #[test]
    fn decimal_size_small_significand_large_exponent() {
        for i in ["1e6100", "1e-6100"] {
            // We only have a single digit, but the exponent is too large for anything smaller than 128bit
            let d = Bitstring::try_parse_str(i).expect("failed to parse decimal");

            assert!(d.as_le_bytes().len() * 8 >= 128, "{}", i);
        }
    }

    #[test]
    fn decimal_size_small_significand() {
        let d = Bitstring::try_parse_str("1").expect("failed to parse decimal");

        assert_eq!(32, d.as_le_bytes().len() * 8);
    }

    #[test]
    fn err_decimal_overflow_exponent() {
        for i in ["1e2147483648", "1e1073741823"] {
            assert!(Bitstring::try_parse_str(i).is_err());
            assert!(Bitstring::try_parse(format_args!("{}", i)).is_err());
        }
    }

    #[test]
    fn err_decimal_overflow_digits() {
        for i in [
            "123456789012345678901234567890123456789012345678901234567890",
            "1234567890123456789.1234567890123456789012345678901234567890",
        ] {
            assert!(Bitstring::try_parse_str(i).is_err());
            assert!(Bitstring::try_parse(format_args!("{}", i)).is_err());
        }
    }

    #[test]
    fn err_decimal_from_invalid_byte_count() {
        let err = Bitstring::try_from_le_bytes(&[]).unwrap_err();
        assert_eq!("the value cannot fit into a decimal of `0` bytes; the width needed is `4` bytes; decimals must be a multiple of 32 bits (4 bytes)", &err.to_string());

        let err = Bitstring::try_from_le_bytes(&[0; 3]).unwrap_err();
        assert_eq!("the value cannot fit into a decimal of `3` bytes; the width needed is `4` bytes; decimals must be a multiple of 32 bits (4 bytes)", &err.to_string());

        let err = Bitstring::try_from_le_bytes(&[0; 32]).unwrap_err();
        assert_eq!(
            "the value cannot fit into a decimal of `20` bytes; the width needed is `32` bytes",
            &err.to_string()
        );
    }

    #[test]
    #[cfg(feature = "arbitrary-precision")]
    fn bigdecimal_large() {
        let mut s = String::new();

        // This is just getting a little silly now. But we want to make sure
        // our real limits are effectively unreachable
        digits(&mut s, 16384);
        s.push('e');
        digits(&mut s, 512);

        let d = BigBitstring::try_parse_str(&s).expect("failed to parse decimal");

        assert_eq!(58272, d.as_le_bytes().len() * 8);
    }
}
