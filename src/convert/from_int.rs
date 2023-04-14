/*!
Encode binary integers to decimal.
*/

use core::iter;

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
    OverflowError,
};

pub(crate) fn decimal_to_int<D: BinaryBuf, I: Integer>(decimal: &D) -> Option<I> {
    let (exp, msd) = decode_combination_finite(decimal);

    match exp.to_i32() {
        // ±123
        Some(0) => {
            let trailing_significand = decode_significand_trailing_declets(decimal);

            let digits = Some(msd.get_ascii())
                .into_iter()
                .chain(trailing_significand.flatten());

            I::try_from_ascii(is_sign_negative(decimal), digits)
        }
        // ±123e1
        Some(exponent) if exponent > 0 => {
            let trailing_significand = decode_significand_trailing_declets(decimal);

            let digits = Some(msd.get_ascii())
                .into_iter()
                .chain(trailing_significand.flatten())
                .chain(iter::repeat(b'0').take(exponent as usize));

            I::try_from_ascii(is_sign_negative(decimal), digits)
        }
        // ±1230e-1
        Some(exponent) if (exponent.abs() as usize) < decimal.precision_digits() => {
            let trailing_significand = decode_significand_trailing_declets(decimal);

            let mut digits = Some(msd.get_ascii())
                .into_iter()
                .chain(trailing_significand.flatten());

            // First, try get the integer part of the decimal
            let i = I::try_from_ascii(
                is_sign_negative(decimal),
                digits
                    .by_ref()
                    .take(decimal.precision_digits() - (exponent.abs() as usize)),
            )?;

            // If the rest of the digits are zero then the number is an integer
            if digits.all(|d| d == b'0') {
                Some(i)
            }
            // If there are any non-zero digits then the number is a decimal
            else {
                None
            }
        }
        // If the exponent is very large or small then it can't be represented as an integer
        _ => None,
    }
}

pub(crate) fn decimal_from_int<D: BinaryBuf, I: itoa::Integer>(int: I) -> Result<D, OverflowError> {
    let mut buf = itoa::Buffer::new();
    let int = buf.format(int);

    decimal_from_parsed(ParsedDecimal::Finite(
        FiniteParser::parse_str(int).expect("i8 can always be parsed"),
    ))
}
