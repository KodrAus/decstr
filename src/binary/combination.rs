use crate::{
    binary::{
        BinaryExponent,
        MostSignificantDigit,
    },
    num::Integer,
    BinaryBuf,
};
use core::iter;

// The sign is encoded into the most significant bit of the decimal number
// For negative numbers, the bit is set to `1`
// For positive numbers, the bit is left `0`
const SIGN_NEGATIVE: u8 = 0b1000_0000u8;

// The bits that identify a decimal as encoding an infinity.
const INFINITY: u8 = 0b0111_1000;

// All bits needed to determine whether a decimal is an infinity.
const INFINITY_COMBINATION: u8 = 0b0111_1110u8;

// Whether or not the NaN is "signaling".
//
// This flag determines whether or not the observance of a NaN should
// immediately trap in hardware or raise an exception in software. They're
// not treated consistently so are kind of niche.
const SIGNALING: u8 = 0b0000_0010u8;

// The bits that identify a decimal as encoding a quiet (non-signaling) NaN.
const NAN: u8 = 0b0111_1100u8;

// All bits needed to determine whether a decimal is a NaN, and whether
// it's quiet or signaling.
const NAN_COMBINATION: u8 = NAN | SIGNALING;

// All bits needed to determine whether a decimal is finite.
const FINITE_COMBINATION: u8 = 0b0111_1000u8;

pub fn encode_combination_infinity<D: BinaryBuf>(decimal: &mut D, is_infinity_negative: bool) {
    // Infinity encoding only ever needs to touch the most significant byte of the decimal.
    //
    // The sign is retained, and the combination field is set to the magic value `11110`.
    // The rest of the number is left full of zeroes.
    let buf = decimal.bytes_mut();

    if is_infinity_negative {
        buf[buf.len() - 1] = INFINITY | SIGN_NEGATIVE;
    } else {
        buf[buf.len() - 1] = INFINITY;
    }
}

pub fn encode_combination_nan<D: BinaryBuf>(
    decimal: &mut D,
    is_nan_negative: bool,
    is_nan_signaling: bool,
) {
    let buf = decimal.bytes_mut();

    let sign_bit = if is_nan_negative { SIGN_NEGATIVE } else { 0 };

    let signaling_bit = if is_nan_signaling { SIGNALING } else { 0 };

    buf[buf.len() - 1] = NAN | sign_bit | signaling_bit;
}

pub fn encode_combination_finite<D: BinaryBuf>(
    decimal: &mut D,
    significand_is_negative: bool,
    unbiased_integer_exponent: D::Exponent,
    most_significant_digit: MostSignificantDigit,
) {
    // First, we bias the exponent.
    //
    // This means adding a number called the "bias" to it so all possible exponents are positive.
    // The value used for the bias is determined by the bit width of the decimal according to
    // IEEE754-2019.

    let biased_exponent = unbiased_integer_exponent.bias(&*decimal);

    debug_assert!(
        !biased_exponent.is_negative(),
        "biased exponents must always be positive"
    );

    let exponent = biased_exponent.to_le_bytes();

    // The point we need to start writing the exponent from is the end of the trailing significand digits
    let exponent_bits = decimal.exponent_width_bits();
    let decimal_bit_index = decimal.trailing_significand_width_bits();

    let buf = decimal.bytes_mut();

    // Second, we write the trailing bits of the exponent.
    //
    // This code is very similar to the way we write the DPD digits into the buffer.
    // Have a look at the bottom of `encode_bcd_to_dpd` for a walkthrough.
    //
    // The difference is that the number of bits we write is the same width as a byte.
    // That means we don't need to compute these offsets each time, they're the same for
    // every pair of bytes we write into.
    let decimal_byte_shift = (decimal_bit_index % 8) as u32;

    let mut decimal_byte_index = decimal_bit_index / 8;
    let mut exponent_byte_index = 0;

    // The trailing exponent bits we're writing always run up to the last byte of the decimal.
    //
    // We always expect to run out of decimal bytes before we run out of exponent bytes.
    // This will leave some trailing junk in that final byte, but it will be cleaned up when
    // we write the combination field shortly.
    let max_decimal_byte_index = buf.len() - 1;

    // Encode trailing exponent bits up to the most significant byte in the decimal.
    //
    // The final byte of the decimal is handled differently than the rest; it includes the
    // combination field which separates the 2 most-significant-bits of the exponent.

    // If the bytes of the exponent is aligned with the bytes in the decimal then
    // assign them without doing any offsetting.
    if decimal_byte_shift == 0 {
        while decimal_byte_index < max_decimal_byte_index {
            buf[decimal_byte_index] = exponent[exponent_byte_index];

            decimal_byte_index += 1;
            exponent_byte_index += 1;
        }
    }
    // If the bytes of the exponent are not aligned with the bytes in the decimal then
    // they'll need to be shifted into position.
    else {
        let decimal_byte_plus_1_shift = (8 - decimal_byte_shift) as u32;

        while decimal_byte_index < max_decimal_byte_index {
            buf[decimal_byte_index] |= exponent[exponent_byte_index] << decimal_byte_shift;
            buf[decimal_byte_index + 1] |=
                exponent[exponent_byte_index] >> decimal_byte_plus_1_shift;

            decimal_byte_index += 1;
            exponent_byte_index += 1;
        }
    }

    // Encode the final 2 non-most-significant bits into the final byte
    buf[decimal_byte_index] |= exponent[exponent_byte_index] << decimal_byte_shift;

    // Finally, we encode the 2 most significant bits of the exponent and most significant digit
    // together into a 5 bit combination field.
    //
    // The most significant digit doesn't need all 4 of its bits to be encoded so we jam it together
    // with the top 2 bits of the exponent. This compresses those 6 bits down to 5.

    let (most_significant_exponent_offset, most_significant_exponent_index) =
        most_significant_exponent_offset(exponent_bits);

    let most_significant_exponent =
        exponent[most_significant_exponent_index] >> most_significant_exponent_offset - 2;

    // Write the final bits of the exponent
    // These will be shifted and masked by the combination field

    let most_significant_digit = most_significant_digit.get_bcd();

    debug_assert_ne!(
        0b0000_0011, most_significant_exponent,
        "an in-range exponent should never have `11` as its most significant bits ({}bit decimal with exponent {})",
        decimal.storage_width_bits(),
        unbiased_integer_exponent.as_display(),
    );

    const C0: u8 = 0b0000_0001;
    const C1: u8 = 0b0000_0010;
    const C2: u8 = 0b0000_0100;
    const C3: u8 = 0b0000_1000;
    const COMBINATION_MASK: u8 = 0b1000_0011;

    let combination = match most_significant_digit & C3 {
        // The most significant digit is small
        // exp: 000000ab
        // sig: 00000cde
        // dec: xabcdexx
        0 => {
            let a = (most_significant_exponent & C1) << 5;
            let b = (most_significant_exponent & C0) << 5;
            let c = (most_significant_digit & C2) << 2;
            let d = (most_significant_digit & C1) << 2;
            let e = (most_significant_digit & C0) << 2;

            a | b | c | d | e
        }
        // The most significant digit is large
        // exp: 000000ab
        // sig: 0000100e
        // dec: x11abexx
        C3 => {
            let a = C0 << 6;
            let b = C0 << 5;
            let c = (most_significant_exponent & C1) << 3;
            let d = (most_significant_exponent & C0) << 3;
            let e = (most_significant_digit & C0) << 2;

            a | b | c | d | e
        }
        _ => unreachable!(),
    };

    // The combination field always fits in the last byte of the decimal
    // We may already have some bits set in this field, so we clear them along the way
    buf[decimal_byte_index] = (buf[decimal_byte_index] & COMBINATION_MASK) | combination;

    if significand_is_negative {
        buf[decimal_byte_index] |= SIGN_NEGATIVE;
    }
}

pub fn decode_combination_finite<D: BinaryBuf>(decimal: &D) -> (D::Exponent, MostSignificantDigit) {
    // The point we need to start writing the exponent from is the end of the trailing significand digits
    let exponent_bits = decimal.exponent_width_bits();
    let decimal_bit_index = decimal.trailing_significand_width_bits();

    let buf = decimal.bytes();

    // We follow the same process as encoding when working out the starting point and offsets.
    //
    // Instead of writing an exponent into the bytes we find, we read the exponent back from them.
    let decimal_byte_shift = (decimal_bit_index % 8) as u32;

    let mut decimal_byte_index = decimal_bit_index / 8;
    let mut exponent_byte_index = 0;
    let max_decimal_byte_index = buf.len() - 1;

    // Work out what the two most significant bits of the exponent and the most significant digit are.
    //
    // It's convenient to do this once here than once we've read to the end of the number, because
    // the results may be needed in multiple branches.
    let (most_significant_exponent, max_exponent_byte_index, most_significant_digit) = {
        // Reading the combination field requires the same offsets that were used to write it
        let (most_significant_exponent_offset, most_significant_exponent_index) =
            most_significant_exponent_offset(exponent_bits);

        const C0: u8 = 0b0100_0000;
        const C1: u8 = 0b0010_0000;
        const C2: u8 = 0b0001_0000;
        const C3: u8 = 0b0000_1000;
        const C4: u8 = 0b0000_0100;
        const COMBINATION_MASK: u8 = 0b0110_0000;

        let combination = buf[max_decimal_byte_index];

        let (most_significant_exponent, most_significant_digit_bcd) =
            match combination & COMBINATION_MASK {
                // The most significant digit is large
                // exp: 000000ab
                // sig: 0000100e
                // dec: x11abexx
                COMBINATION_MASK => {
                    let e0 = (combination & C2) >> 3;
                    let e1 = (combination & C3) >> 3;

                    let d0 = C3;
                    let d1 = 0;
                    let d2 = 0;
                    let d3 = (combination & C4) >> 2;

                    let e = e0 | e1;
                    let d = d0 | d1 | d2 | d3;

                    (e, d)
                }
                // The most significant digit is small
                // exp: 000000ab
                // sig: 00000cde
                // dec: xabcdexx
                _ => {
                    let e0 = (combination & C0) >> 5;
                    let e1 = (combination & C1) >> 5;

                    let d0 = 0;
                    let d1 = (combination & C2) >> 2;
                    let d2 = (combination & C3) >> 2;
                    let d3 = (combination & C4) >> 2;

                    let e = e0 | e1;
                    let d = d0 | d1 | d2 | d3;

                    (e, d)
                }
            };

        (
            most_significant_exponent << most_significant_exponent_offset - 2,
            most_significant_exponent_index,
            most_significant_digit_bcd,
        )
    };

    // Now we iterate through the bytes in the decimal from the start of the exponent
    // through to the combination field.
    //
    // This uses a standard iterator that shifts each byte back into position, then
    // masks out the combination and inserts the 2 most significant bits we decoded earlier.

    const COMBINATION_MASK: u8 = 0b0000_0011;

    // If the bytes of the exponent are already aligned with the bytes of the decimal then
    // they can be read directly from the decimal buffer.
    let biased_exponent = if decimal_byte_shift == 0 {
        D::Exponent::from_binary(iter::from_fn(|| {
            // If there's more bytes in the decimal then read the next one as the next byte
            // of the exponent
            if decimal_byte_index < max_decimal_byte_index {
                let e0 = buf[decimal_byte_index];

                decimal_byte_index += 1;

                Some(e0)
            }
            // If we have reached the last byte of the decimal then mask out the combination field
            // and append the most significant bits of the exponent
            else if decimal_byte_index == max_decimal_byte_index {
                let e0 = buf[decimal_byte_index] & COMBINATION_MASK;

                // Mask in the most significant bits of the exponent.
                //
                // The most significant bits have already been shifted into the right place.
                let e1 = most_significant_exponent;

                decimal_byte_index += 1;

                Some(e0 | e1)
            }
            // If the decimal is at the end then we're finished.
            else {
                None
            }
        }))
    }
    // If the bytes of the exponent are not already aligned with the bytes of the decimal
    // then they'll need to be offset.
    //
    // There's a special case for sizes where the 2 most significant bits are in their own
    // final byte of the exponent.
    else {
        let decimal_byte_plus_1_shift = (8 - decimal_byte_shift) as u32;

        D::Exponent::from_binary(iter::from_fn(|| {
            // If there are more than 2 bytes left then squash them into the next byte of the exponent.
            if decimal_byte_index + 1 < max_decimal_byte_index {
                // Read the first part of the next byte in the exponent
                let e0 = buf[decimal_byte_index] >> decimal_byte_shift;

                // Read the second part of the next byte in the exponent

                // If the second part of the next byte comes from the last byte in the decimal then
                // mask out the combination field. Only the last 2 bits of this byte are needed
                let e1 = buf[decimal_byte_index + 1] << decimal_byte_plus_1_shift;

                decimal_byte_index += 1;
                exponent_byte_index += 1;

                Some(e0 | e1)
            }
            // If there are exactly 2 bytes left in the decimal then mask out the combination field
            // when sqauashing the next byte.
            else if decimal_byte_index + 1 == max_decimal_byte_index {
                // Read the first part of the next byte in the exponent
                let e0 = buf[decimal_byte_index] >> decimal_byte_shift;

                // Read the second part of the next byte in the exponent

                // If the second part of the next byte comes from the last byte in the decimal then
                // mask out the combination field. Only the last 2 bits of this byte are needed
                let mut e1 =
                    (buf[decimal_byte_index + 1] & COMBINATION_MASK) << decimal_byte_plus_1_shift;

                // If this byte is also the last byte of the exponent then include the 2
                // most significant bits from the combination field.
                if exponent_byte_index == max_exponent_byte_index {
                    e1 |= most_significant_exponent;
                }

                decimal_byte_index += 1;
                exponent_byte_index += 1;

                Some(e0 | e1)
            }
            // If there's only 1 byte left but the exponent isn't finished then
            // use the 2 most significant bits from the combination field as the
            // final byte.
            else if exponent_byte_index == max_exponent_byte_index {
                exponent_byte_index += 1;

                Some(most_significant_exponent)
            }
            // If both the decimal and exponent are at the end then we're finished.
            else {
                None
            }
        }))
    };

    debug_assert!(
        !biased_exponent.is_negative(),
        "biased exponents must always be positive"
    );

    (
        biased_exponent.unbias(decimal),
        MostSignificantDigit::from_bcd(most_significant_digit),
    )
}

/**
Calculate an index into the exponent bytes and an offset to find the most significant bits
to encode in the combination field.
*/
fn most_significant_exponent_offset(exponent_bits: usize) -> (u32, usize) {
    let offset = exponent_bits % 8;

    // If the offset is already 8bit-aligned then we treat it slightly differently.
    //
    // Say we have an exponent byte buffer with 2 bytes (so 8 bits each):
    //
    // 0         1
    // [01234567][01234567]
    //
    // If our exponent has 6 bits then the byte we're interested in is `0`:
    //
    // 0         1
    // [01234567][01234567]
    //  --------
    //       ^
    //
    // If our exponent has 8 bits then the byte we're interested is still `0`, not `1`:
    //
    // 0         1
    // [01234567][01234567]
    //  --------
    //         ^
    //
    // But 8 / 8 is 1, not 0, so we need to handle this case specially.
    if offset == 0 {
        (8u32, (exponent_bits / 8) - 1)
    } else {
        (offset as u32, exponent_bits / 8)
    }
}

pub fn is_finite<D: BinaryBuf>(decimal: &D) -> bool {
    let buf = decimal.bytes();

    buf[buf.len() - 1] & FINITE_COMBINATION != FINITE_COMBINATION
}

pub fn is_infinite<D: BinaryBuf>(decimal: &D) -> bool {
    let buf = decimal.bytes();

    buf[buf.len() - 1] & INFINITY_COMBINATION == INFINITY
}

pub fn is_nan<D: BinaryBuf>(decimal: &D) -> bool {
    let buf = decimal.bytes();

    buf[buf.len() - 1] & NAN == NAN
}

pub fn is_quiet_nan<D: BinaryBuf>(decimal: &D) -> bool {
    let buf = decimal.bytes();

    buf[buf.len() - 1] & NAN_COMBINATION == NAN
}

pub fn is_signaling_nan<D: BinaryBuf>(decimal: &D) -> bool {
    let buf = decimal.bytes();

    buf[buf.len() - 1] & NAN_COMBINATION == NAN_COMBINATION
}

pub fn is_sign_negative<D: BinaryBuf>(decimal: &D) -> bool {
    let buf = decimal.bytes();

    buf[buf.len() - 1] & SIGN_NEGATIVE == SIGN_NEGATIVE
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::{
        emax,
        DynamicBinaryBuf,
    };

    fn encode_decode_case<D: BinaryBuf + Clone>(decimal: D) {
        let emin = 1i32
            - emax::<i32>(decimal.precision_digits())
            - (decimal.precision_digits() as i32 - 1);
        let emax = emax::<i32>(decimal.precision_digits()) - (decimal.precision_digits() as i32);

        // Check all possible exponent values for all possible digits
        for exp in emin..=emax {
            for msd in b'0'..=b'9' {
                let mut decimal = decimal.clone();

                encode_combination_finite(
                    &mut decimal,
                    false,
                    D::Exponent::from_i32(exp),
                    MostSignificantDigit::from_ascii(msd),
                );

                let (decoded_exponent, decoded_msd) = decode_combination_finite(&decimal);

                assert_eq!(
                    msd as char,
                    decoded_msd.get_ascii() as char,
                    "most significant digit"
                );

                assert_eq!(
                    exp,
                    decoded_exponent
                        .to_i32()
                        .expect("exponent is outside i32 range"),
                    "exponent"
                );
            }
        }
    }

    #[test]
    fn encode_decode_combination_decimal32_all() {
        encode_decode_case(DynamicBinaryBuf::<4>::EMPTY);
    }

    #[test]
    fn encode_decode_combination_decimal64_all() {
        encode_decode_case(DynamicBinaryBuf::<8>::EMPTY);
    }

    #[test]
    fn encode_decode_combination_decimal96_all() {
        encode_decode_case(DynamicBinaryBuf::<12>::EMPTY);
    }

    #[test]
    fn encode_decode_combination_decimal128_all() {
        encode_decode_case(DynamicBinaryBuf::<16>::EMPTY);
    }

    #[test]
    fn encode_decode_combination_decimal160_all() {
        encode_decode_case(DynamicBinaryBuf::<20>::EMPTY);
    }
}
