/*!
Conversions between text-encoded and binary-encoded decimal numbers.
*/

use crate::{
    binary::{
        decode_combination_finite,
        decode_significand_trailing_declets,
        encode_combination_finite,
        encode_combination_infinity,
        encode_combination_nan,
        encode_significand_trailing_digits,
        is_finite,
        is_infinite,
        is_nan,
        is_quiet_nan,
        is_sign_negative,
        BinaryBuf,
        BinaryExponent,
    },
    num::Integer,
    text::{
        ParsedDecimal,
        ParsedDecimalPoint,
        ParsedFinite,
        ParsedInfinity,
        ParsedNan,
        ParsedNanHeader,
        ParsedSignificand,
        TextBuf,
    },
    OverflowError,
};
use core::{
    fmt,
    str,
};

mod from_binary_float;
mod from_int;
mod from_str;

pub(crate) use self::{
    from_binary_float::*,
    from_int::*,
    from_str::*,
};

/**
Convert a decimal parsed from text into its binary form.
*/
pub(crate) fn decimal_from_parsed<D: BinaryBuf, T: TextBuf>(
    parsed: ParsedDecimal<T>,
) -> Result<D, OverflowError> {
    match parsed {
        // ±1.234e±5
        ParsedDecimal::Finite(ParsedFinite {
            finite_buf,
            finite_significand:
                ParsedSignificand {
                    significand_is_negative,
                    significand_range,
                    decimal_point,
                },
            finite_exponent,
        }) => {
            let buf = finite_buf.get_ascii();

            // First, get a starting point for the exponent.
            let unbiased_exponent = match finite_exponent {
                // If the number has an explicit exponent, like `1.23e4`, then parse it.
                //
                // The value will already have been validated, so this is more of a conversion
                // than a regular parse.
                Some(exponent) => D::try_exponent_from_ascii(
                    exponent.exponent_is_negative,
                    buf[exponent.exponent_range].iter().copied(),
                )?,
                // If the number doesn't have an explicit exponent, like `1.23`, then use 0.
                None => D::default_exponent(),
            };

            match decimal_point {
                // ±1.234e5
                Some(ParsedDecimalPoint {
                    decimal_point_range,
                }) => {
                    let integer_range = significand_range.start..decimal_point_range.start;
                    let fractional_range = decimal_point_range.end..significand_range.end;

                    let integer_digits = &buf[integer_range];
                    let fractional_digits = &buf[fractional_range];

                    // Account for the fractional part of the number
                    let unbiased_integer_exponent =
                        unbiased_exponent.lower(fractional_digits.len());

                    // Get a decimal buffer with enough space to fit all the digits
                    // and the exponent
                    let mut buf = D::try_with_at_least_precision(
                        integer_digits.len() + fractional_digits.len(),
                        Some(&unbiased_integer_exponent),
                    )?;

                    let msd = encode_significand_trailing_digits(
                        &mut buf,
                        [integer_digits, fractional_digits],
                    );

                    encode_combination_finite(
                        &mut buf,
                        significand_is_negative,
                        unbiased_integer_exponent,
                        msd,
                    );

                    Ok(buf)
                }
                // ±123e4
                None => {
                    let integer_range = significand_range;
                    let integer_digits = &buf[integer_range];

                    // Get a decimal buffer with enough space to fit all the digits
                    // and the exponent
                    let mut buf = D::try_with_at_least_precision(
                        integer_digits.len(),
                        Some(&unbiased_exponent),
                    )?;

                    let msd = encode_significand_trailing_digits(&mut buf, [integer_digits]);

                    encode_combination_finite(
                        &mut buf,
                        significand_is_negative,
                        unbiased_exponent,
                        msd,
                    );

                    Ok(buf)
                }
            }
        }
        // ±inf
        ParsedDecimal::Infinity(ParsedInfinity {
            is_infinity_negative,
        }) => {
            // Infinity doesn't encode any special information, so we can ask for a buffer
            // with the minimum size supported
            let mut buf = D::try_with_at_least_storage_width_bytes(4)
                .expect("infinity will always fit in the minimal sized buffer");

            encode_combination_infinity(&mut buf, is_infinity_negative);

            Ok(buf)
        }
        // ±nan(123)
        ParsedDecimal::Nan(ParsedNan {
            nan_buf,
            nan_header:
                ParsedNanHeader {
                    is_nan_signaling,
                    is_nan_negative,
                },
            nan_payload,
        }) => {
            // If the NaN was parsed with a payload then encode it.
            //
            // This process is the same as finite integers.
            if let Some(ParsedSignificand {
                significand_range, ..
            }) = nan_payload
            {
                let payload_buf = nan_buf.get_ascii();

                let mut buf = D::try_with_at_least_precision(significand_range.len() + 1, None)?;

                encode_significand_trailing_digits(&mut buf, [&payload_buf[significand_range]]);

                encode_combination_nan(&mut buf, is_nan_negative, is_nan_signaling);

                Ok(buf)
            }
            // If the NaN doesn't have a payload then just ask for the minimum size buffer,
            // just like we do for infinities.
            else {
                let mut buf = D::try_with_at_least_storage_width_bytes(4)
                    .expect("a NaN with no payload will always fit in the minimal sized buffer");

                encode_combination_nan(&mut buf, is_nan_negative, is_nan_signaling);

                Ok(buf)
            }
        }
    }
}

/**
Convert a decimal in its binary form into text.
*/
pub(crate) fn decimal_to_fmt<D: BinaryBuf>(
    decimal: &D,
    mut out: impl fmt::Write,
) -> Result<(), fmt::Error> {
    // Write the sign
    if is_sign_negative(decimal) {
        out.write_char('-')?;
    }

    // ±1.234e±5
    if is_finite(decimal) {
        let mut written = 0;

        let (exponent, msd) = decode_combination_finite(decimal);
        let msd = msd.get_ascii();

        // The formatter works in declets, which are 3 digits at a time,
        // in order from most to least significant.
        //
        // The precision of a decimal is always some number of declets plus
        // the lonely most-significant-digit. We create a dummy declet for it here.
        // Calculations that depend on the precision of the number need to have these
        // extra two zeroes accounted for.
        let mut declets = decode_significand_trailing_declets(decimal);

        match exponent.to_i32() {
            // ±123
            Some(0) => {
                write_all_as_integer(
                    skip_leading_zeroes(msd, &mut declets),
                    declets,
                    &mut written,
                    &mut out,
                )?;
            }
            // ±123.456
            Some(exponent) if exponent.is_negative() => {
                // Skip leading zeroes
                let skipped = skip_leading_zeroes(msd, &mut declets);

                // Work out how many digits are remaining after skipping leading zeroes.
                //
                // The extra two precision accounts for the dummy declet created for the
                // most-significant-digit.
                let non_zero_digits =
                    adjusted_precision_digits_with_msd_declet(decimal) - skipped.skipped;

                // Work out where the exponent falls in the decimal
                // It might either be somewhere in the middle, as in `123.456`, or before it, as in `0.00123456`.
                match non_zero_digits
                    .try_into()
                    .map(|non_zero_digits: i32| {
                        // The `exponent` is negative, so adding it to the non-zero number of digits will work out how
                        // many digits we need to write before the decimal point.
                        //
                        // This value itself may end up negative. If that happens it means there aren't enough
                        // digits to put the decimal point between. In that case, we'll write `0.` and then
                        // some number of leading zeroes first.
                        non_zero_digits + exponent
                    })
                    .ok()
                {
                    // ±123.456
                    Some(integer_digits) if integer_digits > 0 => {
                        let total_integer_digits = integer_digits as usize;

                        // Write digits up to the decimal point
                        let mut written_decimal_point = false;

                        if let Some((declet, idx)) = skipped.partial_declet {
                            written_decimal_point = write_decimal_digits(
                                &declet[idx..],
                                total_integer_digits,
                                &mut written,
                                &mut out,
                            )?;
                        }

                        while !written_decimal_point {
                            written_decimal_point = write_decimal_digits(
                                &declets
                                    .next()
                                    .expect("ran out of digits before the decimal point"),
                                total_integer_digits,
                                &mut written,
                                &mut out,
                            )?;
                        }

                        // Write any remaining fractional digits
                        for declet in declets {
                            write_declet(declet, &mut written, &mut out)?;
                        }
                    }
                    // ±0.0123
                    Some(leading_zeroes) => {
                        debug_assert!(leading_zeroes == 0 || leading_zeroes.is_negative());

                        // This buffer determines how many leading zeroes we'll try write
                        // before falling back to exponential notation
                        const DECIMAL_ZEROES: &str = "0.00000";

                        let leading_zeroes = leading_zeroes.abs() as usize;

                        // If the decimal point is before the non-zero digits, and there
                        // aren't too many leading zeroes then write them directly.
                        if leading_zeroes + "0.".len() <= DECIMAL_ZEROES.len() {
                            // Write the leading zeroes along with the decimal point
                            write_content(
                                &DECIMAL_ZEROES[..leading_zeroes + "0.".len()],
                                leading_zeroes,
                                &mut written,
                                &mut out,
                            )?;

                            // Write the declets as an integer following the leading fractional zeroes
                            write_all_as_integer(skipped, declets, &mut written, &mut out)?;
                        }
                        // If there are too many leading zeroes then write the number in scientific notation
                        else {
                            write_all_as_scientific(
                                skipped,
                                declets,
                                exponent,
                                &mut written,
                                &mut out,
                            )?;
                        }
                    }
                    // If the exponent is too large for an `i32` then write the number in scientific notation
                    None => {
                        write_all_as_scientific(
                            skipped,
                            declets,
                            exponent,
                            &mut written,
                            &mut out,
                        )?;
                    }
                }
            }
            // ±1.234e±5
            _ => {
                write_all_as_scientific(
                    skip_leading_zeroes(msd, &mut declets),
                    declets,
                    exponent,
                    &mut written,
                    &mut out,
                )?;
            }
        }

        Ok(())
    }
    // ±inf
    else if is_infinite(decimal) {
        out.write_str("inf")
    }
    // ±nan(123)
    else {
        debug_assert!(is_nan(decimal));

        if is_quiet_nan(decimal) {
            out.write_str("nan")?;
        } else {
            out.write_str("snan")?;
        }

        // NaNs may include an integer payload.
        //
        // If the payload is non-zero then it'll also be written to the output.
        let mut payload = decode_significand_trailing_declets(decimal)
            .flatten()
            .peekable();

        // Skip over leading zeroes in the payload
        while let Some(b'0') = payload.peek() {
            let _ = payload.next();
        }

        // If there are any non-zero digits, then write them between braces
        if payload.peek().is_some() {
            out.write_char('(')?;

            for digit in payload {
                out.write_char(digit as char)?;
            }

            out.write_char(')')?;
        }

        Ok(())
    }
}

fn adjusted_precision_digits_with_msd_declet(decimal: &impl BinaryBuf) -> usize {
    decimal.precision_digits() + 2
}

fn skip_leading_zeroes(msd_ascii: u8, mut declets: impl Iterator<Item = [u8; 3]>) -> LeadingZeroes {
    let mut skipped = 0;

    // Check the most-significant-digit
    if msd_ascii == b'0' {
        skipped += 3;
    } else {
        return LeadingZeroes {
            skipped: 2,
            partial_declet: Some(([b'0', b'0', msd_ascii], 2)),
        };
    }

    while let Some(declet) = declets.next() {
        // If the declet contains just zeroes then skip them entirely
        if declet == [b'0', b'0', b'0'] {
            skipped += 3;
            continue;
        }
        // Find the first non-zero slice of the declet
        else {
            let mut i = 0;
            for digit in declet {
                if digit != b'0' {
                    break;
                }

                i += 1;
            }

            skipped += i;

            return LeadingZeroes {
                skipped,
                partial_declet: Some((declet, i)),
            };
        }
    }

    LeadingZeroes {
        skipped,
        partial_declet: None,
    }
}

fn write_content(
    content: &str,
    digits: usize,
    written: &mut usize,
    mut out: impl fmt::Write,
) -> Result<(), fmt::Error> {
    *written += digits;
    out.write_str(content)
}

fn write_digits(
    digits: &[u8],
    written: &mut usize,
    out: impl fmt::Write,
) -> Result<(), fmt::Error> {
    write_content(
        unsafe { str::from_utf8_unchecked(&digits) },
        digits.len(),
        written,
        out,
    )
}

fn write_declet(
    declet: [u8; 3],
    written: &mut usize,
    out: impl fmt::Write,
) -> Result<(), fmt::Error> {
    write_content(
        unsafe { str::from_utf8_unchecked(&declet) },
        declet.len(),
        written,
        out,
    )
}

fn write_decimal_digits(
    digits: &[u8],
    total_integer_digits: usize,
    written: &mut usize,
    mut out: impl fmt::Write,
) -> Result<bool, fmt::Error> {
    // If the decimal point doesn't intersect these digits then write it and continue
    if *written + digits.len() <= total_integer_digits {
        write_digits(digits, written, &mut out)?;

        Ok(false)
    }
    // If the decimal point falls at the start of these digits then print it first
    else if *written == total_integer_digits {
        out.write_char('.')?;
        write_digits(digits, written, &mut out)?;

        Ok(true)
    }
    // If the decimal point falls within these digits then print it in the middle and break
    else {
        let decimal_point = total_integer_digits - *written;

        write_digits(&digits[..decimal_point], written, &mut out)?;
        out.write_char('.')?;
        write_digits(&digits[decimal_point..], written, &mut out)?;

        Ok(true)
    }
}

fn write_all_as_integer(
    leading_zeroes: LeadingZeroes,
    declets: impl Iterator<Item = [u8; 3]>,
    written: &mut usize,
    mut out: impl fmt::Write,
) -> Result<(), fmt::Error> {
    if let LeadingZeroes {
        partial_declet: Some((declet, idx)),
        ..
    } = leading_zeroes
    {
        write_digits(&declet[idx..], written, &mut out)?;
    }

    // Write the remaining digits
    for declet in declets {
        write_declet(declet, written, &mut out)?;
    }

    // If no digits were written then write a zero
    if *written == 0 {
        out.write_char('0')?;
    }

    Ok(())
}

fn write_all_as_scientific(
    leading_zeroes: LeadingZeroes,
    mut declets: impl Iterator<Item = [u8; 3]>,
    exponent: impl BinaryExponent,
    written: &mut usize,
    mut out: impl fmt::Write,
) -> Result<(), fmt::Error> {
    // Write the first declet along with a decimal point
    if let LeadingZeroes {
        partial_declet: Some((declet, idx)),
        ..
    } = leading_zeroes
    {
        write_decimal_digits(&declet[idx..], 1, written, &mut out)?;
    } else {
        if let Some(declet) = declets.next() {
            write_content(
                unsafe { str::from_utf8_unchecked(&[declet[0], b'.', declet[1], declet[2]]) },
                1,
                written,
                &mut out,
            )?;
        }
    }

    // Write the remaining digits
    for declet in declets {
        write_declet(declet, written, &mut out)?;
    }

    // If no digits were written, then write a zero
    if *written == 0 {
        out.write_str("0e")?;
    } else {
        out.write_char('e')?;
    }

    // Adjust the integer exponent to the form `1.23e4`.
    //
    // This means raising it to account for the number of fractional digits written.
    let exponent = exponent.raise(written.saturating_sub(1));

    // Write the exponent into the buffer
    exponent.to_fmt(&mut out)?;

    Ok(())
}

struct LeadingZeroes {
    skipped: usize,
    partial_declet: Option<([u8; 3], usize)>,
}
