/*!
Encode binary integers to decimal.
*/

use core::{
    any::type_name,
    iter,
};

use crate::{
    binary::{
        decode_combination_finite,
        decode_significand_trailing_declets,
        is_sign_negative,
        BinaryBuf,
    },
    convert::decimal_from_parsed,
    num::Integer,
    text::{
        FiniteParser,
        ParsedDecimal,
    },
    ConvertError,
    OverflowError,
};

pub(crate) fn decimal_to_int<D: BinaryBuf, I: Integer>(decimal: &D) -> Result<I, ConvertError> {
    let (exp, msd) = decode_combination_finite(decimal);

    match exp.to_i32() {
        // ±123
        Some(0) => {
            let trailing_significand = decode_significand_trailing_declets(decimal);

            let digits = Some(msd.get_ascii())
                .into_iter()
                .chain(trailing_significand.flatten());

            I::try_from_ascii(is_sign_negative(decimal), digits)
                .ok_or_else(|| ConvertError::would_overflow(type_name::<I>()))
        }
        // ±123e1
        Some(exponent) if exponent > 0 => {
            let trailing_significand = decode_significand_trailing_declets(decimal);

            let digits = Some(msd.get_ascii())
                .into_iter()
                .chain(trailing_significand.flatten())
                .chain(iter::repeat(b'0').take(exponent as usize));

            I::try_from_ascii(is_sign_negative(decimal), digits)
                .ok_or_else(|| ConvertError::would_overflow(type_name::<I>()))
        }
        // ±1230e-1
        Some(exponent) if (exponent.unsigned_abs() as usize) < decimal.precision_digits() => {
            let trailing_significand = decode_significand_trailing_declets(decimal);

            let mut digits = Some(msd.get_ascii())
                .into_iter()
                .chain(trailing_significand.flatten());

            // First, try get the integer part of the decimal
            let i = I::try_from_ascii(
                is_sign_negative(decimal),
                digits
                    .by_ref()
                    .take(decimal.precision_digits() - (exponent.unsigned_abs() as usize)),
            )
            .ok_or_else(|| ConvertError::would_overflow(type_name::<I>()))?;

            // If the rest of the digits are zero then the number is an integer
            if digits.all(|d| d == b'0') {
                Ok(i)
            }
            // If there are any non-zero digits then the number is a decimal
            else {
                Err(ConvertError::non_integer(type_name::<I>()))
            }
        }
        // If the exponent is very large or small then it can't be represented as an integer
        _ => Err(ConvertError::would_overflow(type_name::<I>())),
    }
}

pub(crate) fn decimal_from_int<D: BinaryBuf, I: itoa::Integer>(int: I) -> Result<D, OverflowError> {
    let mut buf = itoa::Buffer::new();
    let int = buf.format(int);

    decimal_from_parsed(ParsedDecimal::Finite(
        FiniteParser::parse_str(int).expect("primitive integers can always be parsed"),
    ))
}
