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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VecTextBuf {
    buf: Vec<u8>,
}

impl Default for VecTextBuf {
    fn default() -> Self {
        VecTextBuf { buf: Vec::new() }
    }
}

impl TextBuf for VecTextBuf {
    fn get_ascii(&self) -> &[u8] {
        &self.buf
    }
}

impl TextWriter for VecTextBuf {
    fn remaining_capacity(&self) -> Option<usize> {
        None
    }

    fn begin_significand(&mut self) -> ParsedSignificand {
        ParsedSignificand {
            significand_is_negative: false,
            significand_range: self.buf.len()..self.buf.len(),
            decimal_point: None,
        }
    }

    fn advance_significand(&mut self, b: u8) {
        self.buf.push(b);
    }

    fn push_significand_digit(&mut self, significand: &mut ParsedSignificand, digit: u8) {
        self.buf.push(digit);

        significand.significand_range.end += 1;
    }

    fn push_significand_decimal_point(&mut self, significand: &mut ParsedSignificand) {
        significand.decimal_point = Some(ParsedDecimalPoint {
            decimal_point_range: self.buf.len()..self.buf.len() + 1,
        });

        self.buf.push(b'.');

        significand.significand_range.end += 1;
    }

    fn significand_is_negative(&mut self, significand: &mut ParsedSignificand) {
        self.buf.push(b'-');

        significand.significand_is_negative = true;
        significand.significand_range.start += 1;
        significand.significand_range.end += 1;
    }

    fn significand_is_positive(&mut self, significand: &mut ParsedSignificand) {
        significand.significand_is_negative = false;
    }

    fn begin_exponent(&mut self) -> ParsedExponent {
        self.buf.push(b'e');

        ParsedExponent {
            exponent_is_negative: false,
            exponent_range: self.buf.len()..self.buf.len(),
        }
    }

    fn push_exponent_digit(&mut self, exponent: &mut ParsedExponent, digit: u8) {
        self.buf.push(digit);

        exponent.exponent_range.end += 1;
    }

    fn exponent_is_negative(&mut self, exponent: &mut ParsedExponent) {
        self.buf.push(b'-');

        exponent.exponent_is_negative = true;
        exponent.exponent_range.start += 1;
        exponent.exponent_range.end += 1;
    }

    fn exponent_is_positive(&mut self, exponent: &mut ParsedExponent) {
        exponent.exponent_is_negative = false;
    }
}
