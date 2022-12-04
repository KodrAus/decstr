/*!
Parse numbers in text to decimal.
*/

use core::fmt;

use crate::{
    binary::BinaryBuf,
    convert::decimal_from_parsed,
    text::{
        DecimalParser,
        TextBuf,
        TextWriter,
    },
    Error,
};

/**
Parse and encode a decimal from its text representation.
*/
pub(crate) fn decimal_from_str<D: BinaryBuf>(f: &str) -> Result<D, Error> {
    Ok(decimal_from_parsed(DecimalParser::parse_str(f)?)?)
}

/**
Parse and encode a decimal from a formattable value.
*/
pub(crate) fn decimal_from_fmt<B: TextWriter + TextBuf, D: BinaryBuf>(
    f: impl fmt::Display,
    buf: B,
) -> Result<D, Error> {
    let mut parser = DecimalParser::begin(buf);

    parser.parse_fmt(f)?;

    Ok(decimal_from_parsed(parser.end()?)?)
}
