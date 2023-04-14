/*!
An IEEE754-2019 compatible decimal interchange encoding.

This module implements an arbitrary precision binary format that's stored in a contiguous byte buffer.
Compared to the text format, the binary format is compact. The number can be classified by examining
a single byte. These buffers can be persisted or sent over networks to other processes consistently.
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
    fn decode() {
        // Ensure we don't panic reading potentially nonsense encodings
        for b in 0..255 {
            let buf = Decimal128Buf([b; 16]);

            let _ = decode_combination_finite(&buf);
            let _ = decode_significand_trailing_declets(&buf).count();
        }
    }
}
