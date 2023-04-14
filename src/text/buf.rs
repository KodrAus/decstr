use crate::text::{
    ParsedExponent,
    ParsedSignificand,
};

mod buffered;
mod pre_formatted;

pub use self::{
    buffered::*,
    pre_formatted::*,
};

/**
A buffer for parsed decimal numbers.

A `TextBuf` is probably also going to be a `TextWriter`, which produces the buffers this
trait provides.
*/
pub trait TextBuf {
    /**
    Get the bytes of the underlying buffer.
    */
    fn get_ascii(&self) -> &[u8];
}

/**
A writer for parsed decimal numbers that can produce text buffers containing digits and
track offsets.

A `TextWriter` is probably also going to be a `TextBuf`, which stashes the written bytes
to be accessed later.
*/
pub trait TextWriter {
    fn remaining_capacity(&self) -> Option<usize>;

    fn begin_significand(&mut self) -> ParsedSignificand;
    fn advance_significand(&mut self, b: u8);
    fn push_significand_digit(&mut self, significand: &mut ParsedSignificand, digit: u8);
    fn push_significand_decimal_point(&mut self, significand: &mut ParsedSignificand);
    fn significand_is_negative(&mut self, significand: &mut ParsedSignificand);
    fn significand_is_positive(&mut self, significand: &mut ParsedSignificand);

    fn begin_exponent(&mut self) -> ParsedExponent;
    fn push_exponent_digit(&mut self, exponent: &mut ParsedExponent, digit: u8);
    fn exponent_is_negative(&mut self, exponent: &mut ParsedExponent);
    fn exponent_is_positive(&mut self, exponent: &mut ParsedExponent);
}
