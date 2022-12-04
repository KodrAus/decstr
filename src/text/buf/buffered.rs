use crate::text::{
    ParsedDecimalPoint,
    ParsedExponent,
    ParsedSignificand,
    TextBuf,
    TextWriter,
};

/**
A buffer that splits between the significand and exponent so they can use different buffers.
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedSizeTextBuf<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> Default for FixedSizeTextBuf<N> {
    fn default() -> Self {
        FixedSizeTextBuf {
            buf: [b'0'; N],
            len: 0,
        }
    }
}

impl<const N: usize> FixedSizeTextBuf<N> {
    #[cfg(test)]
    pub fn at_end(at_end: &[u8]) -> Self {
        let mut buf = [b'0'; N];

        buf[..at_end.len()].copy_from_slice(at_end);

        FixedSizeTextBuf {
            buf,
            len: at_end.len(),
        }
    }
}

impl<const N: usize> TextBuf for FixedSizeTextBuf<N> {
    fn get_ascii(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl<const N: usize> TextWriter for FixedSizeTextBuf<N> {
    fn remaining_capacity(&self) -> Option<usize> {
        Some(N - self.len)
    }

    fn begin_significand(&mut self) -> ParsedSignificand {
        ParsedSignificand {
            significand_is_negative: false,
            significand_range: self.len..self.len,
            decimal_point: None,
        }
    }

    fn advance_significand(&mut self, b: u8) {
        self.buf[self.len] = b;
        self.len += 1;
    }

    fn push_significand_digit(&mut self, significand: &mut ParsedSignificand, digit: u8) {
        self.buf[self.len] = digit;

        self.len += 1;

        significand.significand_range.end += 1;
    }

    fn push_significand_decimal_point(&mut self, significand: &mut ParsedSignificand) {
        self.buf[self.len] = b'.';

        significand.decimal_point = Some(ParsedDecimalPoint {
            decimal_point_range: self.len..self.len + 1,
        });

        self.len += 1;

        significand.significand_range.end += 1;
    }

    fn significand_is_negative(&mut self, significand: &mut ParsedSignificand) {
        self.buf[self.len] = b'-';

        self.len += 1;

        significand.significand_is_negative = true;
        significand.significand_range.start += 1;
        significand.significand_range.end += 1;
    }

    fn begin_exponent(&mut self) -> ParsedExponent {
        self.buf[self.len] = b'e';

        self.len += 1;

        ParsedExponent {
            exponent_is_negative: false,
            exponent_range: self.len..self.len,
        }
    }

    fn push_exponent_digit(&mut self, exponent: &mut ParsedExponent, digit: u8) {
        self.buf[self.len] = digit;

        self.len += 1;

        exponent.exponent_range.end += 1;
    }

    fn exponent_is_negative(&mut self, exponent: &mut ParsedExponent) {
        self.buf[self.len] = b'-';

        self.len += 1;

        exponent.exponent_is_negative = true;
        exponent.exponent_range.start += 1;
        exponent.exponent_range.end += 1;
    }
}
