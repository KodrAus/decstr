/*!
Encoding binary floating point to decimal.
*/

use core::any::type_name;

use crate::{
    binary::{
        decode_combination_finite,
        decode_significand_trailing_declets,
        is_finite,
        is_infinite,
        is_nan,
        is_sign_negative,
        is_signaling_nan,
        BinaryBuf,
    },
    convert::decimal_from_parsed,
    num::{
        Float,
        Integer,
    },
    text::{
        FiniteParser,
        ParsedDecimal,
        ParsedInfinity,
        ParsedNan,
        ParsedNanHeader,
        PreFormattedTextBuf,
    },
    ConvertError,
    OverflowError,
};

pub(crate) fn decimal_to_binary_float<F: Float, D: BinaryBuf>(
    decimal: &D,
) -> Result<F, ConvertError> {
    if is_finite(decimal) {
        let (exp, msd) = decode_combination_finite(decimal);

        let trailing_significand = decode_significand_trailing_declets(decimal);

        let f = F::try_finite_from_ascii(
            is_sign_negative(decimal),
            Some(msd.get_ascii())
                .into_iter()
                .chain(trailing_significand.flatten()),
            exp,
        )
        .ok_or_else(|| ConvertError::would_overflow(type_name::<F>()))?;

        // If the parsed floating point is finite then return it
        if f.is_finite() {
            Ok(f)
        }
        // If the parsed floating point is non-finite then it's probably
        // infinity, meaning it doesn't fit in the given format. In this
        // case we return `None` rather than infinity to signal overflow.
        else {
            Err(ConvertError::would_overflow(type_name::<F>()))
        }
    } else if is_infinite(decimal) {
        Ok(F::infinity(is_sign_negative(decimal)))
    } else {
        debug_assert!(is_nan(decimal));

        let payload = decode_significand_trailing_declets(decimal);

        let payload = F::NanPayload::try_from_ascii(false, payload.flatten())
            .ok_or_else(|| ConvertError::would_overflow(type_name::<F>()))?;

        Ok(F::nan(
            is_sign_negative(decimal),
            is_signaling_nan(decimal),
            payload,
        ))
    }
}

pub(crate) fn decimal_from_binary_float<D: BinaryBuf, F: Float + ryu::Float>(
    float: F,
) -> Result<D, OverflowError> {
    if float.is_finite() {
        // The value is a finite number, like `-123.456e-7`.
        //
        // A finite number consists of a sign, an exponent, and a significand.

        let mut buf = ryu::Buffer::new();
        let f = buf.format_finite(float);

        // ryū picks a reasonable representation for floating points, picking
        // between scientific and regular formatting automatically
        decimal_from_parsed(ParsedDecimal::Finite(
            FiniteParser::parse_str(f).expect("f64 can always be parsed"),
        ))
    } else if float.is_infinite() {
        // The value is infinity.
        //
        // Infinities represent overflow conditions. They can be either positive or negative.
        // Unlike NaNs, infinities don't carry diagnostic payloads.

        decimal_from_parsed(ParsedDecimal::<PreFormattedTextBuf>::Infinity(
            ParsedInfinity {
                is_infinity_negative: float.is_sign_negative(),
            },
        ))
    } else {
        // The value is a NaN (not a number).
        //
        // NaNs are a strange sort of value encoded in a floating point number that
        // represents an exception (such as the result of 1/0).

        debug_assert!(float.is_nan());

        // NaN payloads are treated like integers, so we use the same infrastructure
        // for parsing them as regular finite numbers. Payloads don't support scientific
        // notation, and have slightly less precision than finite numbers, but it's fine to
        // use the same buffer size here.
        let buf = F::TextWriter::default();

        let (nan_buf, nan_payload) = if let Some(payload) = float.nan_payload() {
            let payload = FiniteParser::parse(buf, payload.as_display())
                .expect("integers can always be parsed");

            (payload.finite_buf, Some(payload.finite_significand))
        } else {
            (buf, None)
        };

        decimal_from_parsed(ParsedDecimal::Nan(ParsedNan {
            nan_buf,
            nan_header: ParsedNanHeader {
                is_nan_signaling: float.is_nan_signaling(),
                is_nan_negative: float.is_sign_negative(),
            },
            nan_payload,
        }))
    }
}
