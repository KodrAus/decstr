use crate::text::{
    ParsedDecimalPoint,
    ParsedExponent,
    ParsedSignificand,
    TextBuf,
    TextWriter,
};

/**
A buffer that already contains an ASCII-coded decimal.
*/
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct StrTextBuf<'a> {
    ascii: &'a [u8],
    index: usize,
}

impl<'a> StrTextBuf<'a> {
    pub fn new(buf: &'a str) -> Self {
        let ascii = buf.as_bytes();

        StrTextBuf { ascii, index: 0 }
    }

    #[cfg(test)]
    pub fn at_end(buf: &'a str) -> Self {
        let ascii = buf.as_bytes();
        let index = ascii.len();

        StrTextBuf { ascii, index }
    }
}

impl<'a> TextBuf for StrTextBuf<'a> {
    fn get_ascii(&self) -> &[u8] {
        self.ascii
    }
}

impl<'a> TextWriter for StrTextBuf<'a> {
    fn remaining_capacity(&self) -> Option<usize> {
        None
    }

    fn begin_significand(&mut self) -> ParsedSignificand {
        ParsedSignificand {
            significand_is_negative: false,
            significand_range: self.index..self.index,
            decimal_point: None,
        }
    }

    fn advance_significand(&mut self, _: u8) {
        self.index += 1;
    }

    fn push_significand_digit(&mut self, significand: &mut ParsedSignificand, _digit: u8) {
        debug_assert_eq!(_digit, self.ascii[self.index]);

        self.index += 1;

        // The end of the significand range will shift to the index
        significand.significand_range.end = self.index;
    }

    fn push_significand_decimal_point(&mut self, significand: &mut ParsedSignificand) {
        debug_assert_eq!(b'.', self.ascii[self.index]);

        // The decimal point will always mark the position in the buffer that the `.` appears.
        significand.decimal_point = Some(ParsedDecimalPoint {
            decimal_point_range: self.index..self.index + 1,
        });

        self.index += 1;

        significand.significand_range.end = self.index;
    }

    fn significand_is_negative(&mut self, significand: &mut ParsedSignificand) {
        self.index += 1;

        significand.significand_is_negative = true;
        significand.significand_range.start += 1;
        significand.significand_range.end += 1;
    }

    fn significand_is_positive(&mut self, significand: &mut ParsedSignificand) {
        self.index += 1;

        significand.significand_is_negative = false;
        significand.significand_range.start += 1;
        significand.significand_range.end += 1;
    }

    fn begin_exponent(&mut self) -> ParsedExponent {
        self.index += 1;

        ParsedExponent {
            exponent_is_negative: false,
            exponent_range: self.index..self.index,
        }
    }

    fn push_exponent_digit(&mut self, exponent: &mut ParsedExponent, _: u8) {
        self.index += 1;

        exponent.exponent_range.end = self.index;
    }

    fn exponent_is_negative(&mut self, exponent: &mut ParsedExponent) {
        self.index += 1;

        exponent.exponent_is_negative = true;
        exponent.exponent_range.start += 1;
        exponent.exponent_range.end += 1;
    }

    fn exponent_is_positive(&mut self, exponent: &mut ParsedExponent) {
        self.index += 1;

        exponent.exponent_is_negative = false;
        exponent.exponent_range.start += 1;
        exponent.exponent_range.end += 1;
    }
}
