/*!
An IEEE754-2019 compatible decimal interchange encoding.

This module implements an arbitrary precision binary format that's stored in a contiguous byte buffer.
Compared to the text format, the binary format is compact. The number can be classified by examining
a single byte. These buffers can be persisted or sent over networks to other processes consistently.

This module is organized around _features_ of the encoded decimal.
*/

mod buf;
mod combination;
mod exponent;
mod significand;

pub use self::{
    buf::*,
    combination::*,
    exponent::*,
    significand::*,
};

pub(crate) fn encode_max<D: BinaryBuf>(buf: &mut D, is_negative: bool) {
    let bit_width = buf.storage_width_bits();
    let max_digits = precision_digits(bit_width);

    let exp = <D::Exponent>::emax(buf).lower(max_digits);

    let msd = encode_significand_trailing_digits_repeat(buf, b'9');

    debug_assert_eq!(b'9', msd.get_ascii());

    encode_combination_finite(buf, is_negative, exp, msd);
}

pub(crate) fn encode_min<D: BinaryBuf>(buf: &mut D, is_negative: bool) {
    let bit_width = buf.storage_width_bits();
    let max_digits = precision_digits(bit_width);

    let exp = <D::Exponent>::emin(buf).raise(1).lower(max_digits);

    let msd = encode_significand_trailing_digits(buf, [b"1"]);

    encode_combination_finite(buf, is_negative, exp, msd);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_byte_order() {
        let mut buf = Decimal128Buf([0; 16]);

        let is_negative = true;
        let exp = 2;

        let msd = encode_significand_trailing_digits(&mut buf, [b"123456789"]);

        encode_combination_finite(&mut buf, is_negative, exp, msd);

        // Bytes are ordered least to most significant regardless of the platform's endianness
        assert_eq!(
            buf.0,
            [207, 91, 57, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 8, 162]
        );
    }

    #[test]
    fn encode_min_max_32() {
        encode_max(&mut Decimal32Buf([0; 4]), false);
        encode_max(&mut Decimal32Buf([0; 4]), true);

        encode_min(&mut Decimal32Buf([0; 4]), false);
        encode_min(&mut Decimal32Buf([0; 4]), true);
    }

    #[test]
    fn encode_min_max_64() {
        encode_max(&mut Decimal64Buf([0; 8]), false);
        encode_max(&mut Decimal64Buf([0; 8]), true);

        encode_min(&mut Decimal64Buf([0; 8]), false);
        encode_min(&mut Decimal64Buf([0; 8]), true);
    }

    #[test]
    fn encode_min_max_128() {
        encode_max(&mut Decimal128Buf([0; 16]), false);
        encode_max(&mut Decimal128Buf([0; 16]), true);

        encode_min(&mut Decimal128Buf([0; 16]), false);
        encode_min(&mut Decimal128Buf([0; 16]), true);
    }

    #[test]
    #[allow(const_item_mutation)]
    fn encode_min_max_dynamic() {
        encode_max(&mut DynamicBinaryBuf::<20>::EMPTY, false);
        encode_max(&mut DynamicBinaryBuf::<20>::EMPTY, true);

        encode_min(&mut DynamicBinaryBuf::<20>::EMPTY, false);
        encode_min(&mut DynamicBinaryBuf::<20>::EMPTY, true);
    }

    #[test]
    fn decode() {
        // Ensure we don't panic reading potentially nonsense encodings
        for b in 0..255 {
            let buf = Decimal128Buf([b; 16]);

            let _ = decode_combination_finite(&buf);
            let _ = decode_significand_trailing_declets(&buf).count();
        }
    }
}
