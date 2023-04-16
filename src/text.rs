/*!
A text-based format for decimal numbers.

This module implements text parsers for decimal numbers. The parsers take a number such as
`-123.456e-789` and parse its features into a temporary buffer along with offsets for the
decimal point and exponent. When the input number is a string then no temporary buffering
needs to take place.

The output from the text parser is a `ParsedDecimal` that classifies the number and gives offsets
to any digits or other features within it. These offsets can be used to convert the number into
different representations.

This module is organized around _kinds_ of numbers.
*/

mod buf;
mod finite;
mod infinity;
mod nan;

pub use self::{
    buf::*,
    finite::*,
    infinity::*,
    nan::*,
};

use core::{
    fmt::{
        self,
        Write,
    },
    ops::Range,
    str,
};

use crate::ParseError;

/**
A decimal number parsed from its textual representation.
*/
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ParsedDecimal<B> {
    /**
    The number is finite, like `-123.456e7`.
    */
    Finite(ParsedFinite<B>),
    /**
    The number is infinity, like `-inf`.
    */
    Infinity(ParsedInfinity),
    /**
    The number is a NaN, like `nan`.
    */
    Nan(ParsedNan<B>),
}

/**
A decimal number parsed from a formatted string.

The number contains a buffer of digits and some metadata about the significand and exponent.
The buffer may be the original string parsed, or it could be digits buffered from it.
 */
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParsedFinite<B> {
    pub finite_buf: B,
    pub finite_significand: ParsedSignificand,
    pub finite_exponent: Option<ParsedExponent>,
}

/**
A value that is not a number parsed form a formatted string.

The NaN may contain a payload, which is an integer value.
*/
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParsedNan<B> {
    pub nan_buf: B,
    pub nan_header: ParsedNanHeader,
    pub nan_payload: Option<ParsedSignificand>,
}

/**
An infinite value parsed form a formatted string.
*/
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParsedInfinity {
    pub is_infinity_negative: bool,
}

impl Default for ParsedInfinity {
    fn default() -> Self {
        ParsedInfinity {
            is_infinity_negative: false,
        }
    }
}

/**
Offsets and metadata about the significand parsed from a formatted decimal
 */
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParsedSignificand {
    pub significand_is_negative: bool,
    pub significand_range: Range<usize>,
    pub decimal_point: Option<ParsedDecimalPoint>,
}

impl Default for ParsedSignificand {
    fn default() -> Self {
        ParsedSignificand {
            significand_is_negative: false,
            significand_range: 0..0,
            decimal_point: None,
        }
    }
}

/**
Offsets and metadata about the significand decimal point parsed from a formatted decimal.
 */
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParsedDecimalPoint {
    pub decimal_point_range: Range<usize>,
}

impl Default for ParsedDecimalPoint {
    fn default() -> Self {
        ParsedDecimalPoint {
            decimal_point_range: 0..0,
        }
    }
}

/**
Offsets and metadata about the exponent parsed from a formatted decimal.
 */
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParsedExponent {
    pub exponent_is_negative: bool,
    pub exponent_range: Range<usize>,
}

impl Default for ParsedExponent {
    fn default() -> Self {
        ParsedExponent {
            exponent_is_negative: false,
            exponent_range: 0..0,
        }
    }
}

/**
The header for a NaN value.

The header contains information about the NaN outside of its payload.
*/
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParsedNanHeader {
    pub is_nan_signaling: bool,
    pub is_nan_negative: bool,
}

impl Default for ParsedNanHeader {
    fn default() -> Self {
        ParsedNanHeader {
            is_nan_signaling: false,
            is_nan_negative: false,
        }
    }
}

/**
A parser for a decimal number that may be finite, infinite, or NaN (not-a-number).
*/
#[derive(Debug)]
pub struct DecimalParser<B>(DecimalParserInner<B>);

#[derive(Debug)]
enum DecimalParserInner<B> {
    AtStart {
        buf: Option<B>,
        is_negative: Option<bool>,
        error: Option<ParseError>,
    },
    Finite(FiniteParser<B>),
    Infinity(InfinityParser<B>),
    Nan(NanParser<B>),
}

impl<'a> DecimalParser<StrTextBuf<'a>> {
    pub fn parse_str(input: &'a str) -> Result<ParsedDecimal<StrTextBuf<'a>>, ParseError> {
        let mut parser = DecimalParser::begin(StrTextBuf::new(input));

        parser.parse_ascii(input.as_bytes())?;

        parser.end()
    }
}

impl<B: TextWriter> DecimalParser<B> {
    pub fn begin(buf: B) -> Self {
        DecimalParser(DecimalParserInner::AtStart {
            buf: Some(buf),
            error: None,
            is_negative: None,
        })
    }

    pub fn parse_fmt(&mut self, f: impl fmt::Display) -> Result<(), ParseError> {
        write!(self, "{}", f).map_err(|err| self.unwrap_context(err))
    }

    pub fn parse_ascii(&mut self, mut ascii: &[u8]) -> Result<(), ParseError> {
        while ascii.len() > 0 {
            match self.0 {
                // If we're parsing a finite number then forward the remaining input to it
                DecimalParserInner::Finite(ref mut finite) => return finite.parse_ascii(ascii),
                // If we're at the start of the input then look for the first character that
                // will tell us what kind of number we're expecting.
                DecimalParserInner::AtStart {
                    ref mut is_negative,
                    ref mut buf,
                    ..
                } => match ascii[0] {
                    // Finite
                    b'0'..=b'9' => {
                        let mut finite = FiniteParser::begin(buf.take().expect("missing buffer"));

                        match is_negative {
                            Some(false) => finite.significand_is_positive(),
                            Some(true) => finite.significand_is_negative(),
                            _ => (),
                        }

                        finite.push_significand_digit(ascii[0]);

                        self.0 = DecimalParserInner::Finite(finite);
                    }
                    // A `-` sign doesn't tell us whether the number is finite or not
                    // We stash it away until we know for sure
                    b'-' if is_negative.is_none() => *is_negative = Some(true),
                    // A `+` sign is treated the same as `-`
                    b'+' if is_negative.is_none() => *is_negative = Some(false),
                    // Signaling NaN
                    b's' | b'S' => {
                        let mut nan = NanParser::begin(buf.take().expect("missing buffer"));

                        match is_negative {
                            Some(false) => nan.nan_is_positive(b'+'),
                            Some(true) => nan.nan_is_negative(b'-'),
                            _ => (),
                        }

                        nan.nan_is_signaling(ascii[0]);

                        self.0 = DecimalParserInner::Nan(nan);
                    }
                    // Quiet NaN
                    b'n' | b'N' => {
                        let mut nan = NanParser::begin(buf.take().expect("missing buffer"));

                        match is_negative {
                            Some(false) => nan.nan_is_positive(b'+'),
                            Some(true) => nan.nan_is_negative(b'-'),
                            _ => (),
                        }

                        nan.nan_is_quiet(ascii[0]);

                        self.0 = DecimalParserInner::Nan(nan);
                    }
                    // Infinity
                    b'i' | b'I' => {
                        let mut inf = InfinityParser::begin(buf.take().expect("missing buffer"));

                        match is_negative {
                            Some(false) => inf.infinity_is_positive(),
                            Some(true) => inf.infinity_is_negative(),
                            _ => (),
                        }

                        inf.advance(ascii[0]);

                        self.0 = DecimalParserInner::Infinity(inf)
                    }
                    // Any other character is invalid
                    c => {
                        return Err(ParseError::unexpected_char(
                            c,
                            "a finite number, infinity, or NaN",
                        ))
                    }
                },
                // If we're parsing infinity then forward the rest of the input to it
                DecimalParserInner::Infinity(ref mut infinity) => {
                    return infinity.parse_ascii(ascii);
                }
                // If we're parsing NaN then forward the rest of the input to it
                DecimalParserInner::Nan(ref mut nan) => {
                    return nan.parse_ascii(ascii);
                }
            }

            ascii = &ascii[1..];
        }

        Ok(())
    }

    pub fn end(self) -> Result<ParsedDecimal<B>, ParseError> {
        match self.0 {
            DecimalParserInner::Finite(finite) => Ok(ParsedDecimal::Finite(finite.end()?)),
            DecimalParserInner::Infinity(infinity) => Ok(ParsedDecimal::Infinity(infinity.end()?)),
            DecimalParserInner::Nan(nan) => Ok(ParsedDecimal::Nan(nan.end()?)),
            DecimalParserInner::AtStart { .. } => Err(ParseError::unexpected_end(
                "a finite number, infinity, or NaN",
            )),
        }
    }

    pub fn context(&mut self, err: ParseError) -> fmt::Error {
        match self.0 {
            DecimalParserInner::AtStart { ref mut error, .. } => {
                *error = Some(err);

                fmt::Error
            }
            DecimalParserInner::Finite(ref mut parser) => parser.context(err),
            DecimalParserInner::Infinity(ref mut parser) => parser.context(err),
            DecimalParserInner::Nan(ref mut parser) => parser.context(err),
        }
    }

    pub fn unwrap_context(&mut self, err: fmt::Error) -> ParseError {
        match self.0 {
            DecimalParserInner::AtStart { ref mut error, .. } => {
                error.take().unwrap_or_else(ParseError::source)
            }
            DecimalParserInner::Finite(ref mut parser) => parser.unwrap_context(err),
            DecimalParserInner::Infinity(ref mut parser) => parser.unwrap_context(err),
            DecimalParserInner::Nan(ref mut parser) => parser.unwrap_context(err),
        }
    }
}

impl<B: TextWriter> Write for DecimalParser<B> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.parse_ascii(s.as_bytes())
            .map_err(|err| self.context(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_decimal_propagates_input_to_sub_parsers() {
        for (input, expected) in &[
            (
                "inf",
                ParsedDecimal::<StrTextBuf>::Infinity(ParsedInfinity {
                    is_infinity_negative: false,
                }),
            ),
            (
                "-inf",
                ParsedDecimal::<StrTextBuf>::Infinity(ParsedInfinity {
                    is_infinity_negative: true,
                }),
            ),
            (
                "NaN",
                ParsedDecimal::<StrTextBuf>::Nan(ParsedNan {
                    nan_buf: StrTextBuf::at_end("NaN"),
                    nan_header: ParsedNanHeader {
                        is_nan_signaling: false,
                        is_nan_negative: false,
                    },
                    nan_payload: None,
                }),
            ),
            (
                "-NaN",
                ParsedDecimal::<StrTextBuf>::Nan(ParsedNan {
                    nan_buf: StrTextBuf::at_end("-NaN"),
                    nan_header: ParsedNanHeader {
                        is_nan_signaling: false,
                        is_nan_negative: true,
                    },
                    nan_payload: None,
                }),
            ),
            (
                "+NaN",
                ParsedDecimal::<StrTextBuf>::Nan(ParsedNan {
                    nan_buf: StrTextBuf::at_end("+NaN"),
                    nan_header: ParsedNanHeader {
                        is_nan_signaling: false,
                        is_nan_negative: false,
                    },
                    nan_payload: None,
                }),
            ),
            (
                "sNaN",
                ParsedDecimal::<StrTextBuf>::Nan(ParsedNan {
                    nan_buf: StrTextBuf::at_end("sNaN"),
                    nan_header: ParsedNanHeader {
                        is_nan_signaling: true,
                        is_nan_negative: false,
                    },
                    nan_payload: None,
                }),
            ),
            (
                "-sNaN",
                ParsedDecimal::<StrTextBuf>::Nan(ParsedNan {
                    nan_buf: StrTextBuf::at_end("-sNaN"),
                    nan_header: ParsedNanHeader {
                        is_nan_signaling: true,
                        is_nan_negative: true,
                    },
                    nan_payload: None,
                }),
            ),
            (
                "+sNaN",
                ParsedDecimal::<StrTextBuf>::Nan(ParsedNan {
                    nan_buf: StrTextBuf::at_end("+sNaN"),
                    nan_header: ParsedNanHeader {
                        is_nan_signaling: true,
                        is_nan_negative: false,
                    },
                    nan_payload: None,
                }),
            ),
            (
                "1.23456e7",
                ParsedDecimal::<StrTextBuf>::Finite(ParsedFinite {
                    finite_buf: StrTextBuf::at_end("1.23456e7"),
                    finite_significand: ParsedSignificand {
                        significand_range: 0..7,
                        decimal_point: Some(ParsedDecimalPoint {
                            decimal_point_range: 1..2,
                        }),
                        ..Default::default()
                    },
                    finite_exponent: Some(ParsedExponent {
                        exponent_range: 8..9,
                        ..Default::default()
                    }),
                }),
            ),
            (
                "-1.23456e7",
                ParsedDecimal::<StrTextBuf>::Finite(ParsedFinite {
                    finite_buf: StrTextBuf::at_end("-1.23456e7"),
                    finite_significand: ParsedSignificand {
                        significand_is_negative: true,
                        significand_range: 1..8,
                        decimal_point: Some(ParsedDecimalPoint {
                            decimal_point_range: 2..3,
                        }),
                        ..Default::default()
                    },
                    finite_exponent: Some(ParsedExponent {
                        exponent_is_negative: false,
                        exponent_range: 9..10,
                    }),
                }),
            ),
        ] {
            let mut parser = DecimalParser::begin(StrTextBuf::new(input));
            parser.write_str(input).expect("failed to parse");
            let parsed = parser.end().expect("failed to parse");

            assert_eq!(expected, &parsed, "{}", input);
        }
    }

    #[test]
    fn parse_finite_valid() {
        for (input, expected) in &[
            (
                "0",
                ParsedFinite {
                    finite_buf: StrTextBuf::at_end("0"),
                    finite_significand: ParsedSignificand {
                        significand_is_negative: false,
                        significand_range: 0..1,
                        decimal_point: None,
                    },
                    finite_exponent: None,
                },
            ),
            (
                "-0",
                ParsedFinite {
                    finite_buf: StrTextBuf::at_end("-0"),
                    finite_significand: ParsedSignificand {
                        significand_is_negative: true,
                        significand_range: 1..2,
                        decimal_point: None,
                    },
                    finite_exponent: None,
                },
            ),
            (
                "+0",
                ParsedFinite {
                    finite_buf: StrTextBuf::at_end("+0"),
                    finite_significand: ParsedSignificand {
                        significand_is_negative: false,
                        significand_range: 1..2,
                        decimal_point: None,
                    },
                    finite_exponent: None,
                },
            ),
            (
                "00",
                ParsedFinite {
                    finite_buf: StrTextBuf::at_end("00"),
                    finite_significand: ParsedSignificand {
                        significand_is_negative: false,
                        significand_range: 0..2,
                        decimal_point: None,
                    },
                    finite_exponent: None,
                },
            ),
            (
                "0.0",
                ParsedFinite {
                    finite_buf: StrTextBuf::at_end("0.0"),
                    finite_significand: ParsedSignificand {
                        significand_is_negative: false,
                        significand_range: 0..3,
                        decimal_point: Some(ParsedDecimalPoint {
                            decimal_point_range: 1..2,
                        }),
                    },
                    finite_exponent: None,
                },
            ),
            (
                "0.00",
                ParsedFinite {
                    finite_buf: StrTextBuf::at_end("0.00"),
                    finite_significand: ParsedSignificand {
                        significand_is_negative: false,
                        significand_range: 0..4,
                        decimal_point: Some(ParsedDecimalPoint {
                            decimal_point_range: 1..2,
                        }),
                    },
                    finite_exponent: None,
                },
            ),
            (
                "123",
                ParsedFinite {
                    finite_buf: StrTextBuf::at_end("123"),
                    finite_significand: ParsedSignificand {
                        significand_is_negative: false,
                        significand_range: 0..3,
                        decimal_point: None,
                    },
                    finite_exponent: None,
                },
            ),
            (
                "123.456",
                ParsedFinite {
                    finite_buf: StrTextBuf::at_end("123.456"),
                    finite_significand: ParsedSignificand {
                        significand_is_negative: false,
                        significand_range: 0..7,
                        decimal_point: Some(ParsedDecimalPoint {
                            decimal_point_range: 3..4,
                        }),
                    },
                    finite_exponent: None,
                },
            ),
            (
                "123e456",
                ParsedFinite {
                    finite_buf: StrTextBuf::at_end("123e456"),
                    finite_significand: ParsedSignificand {
                        significand_is_negative: false,
                        significand_range: 0..3,
                        decimal_point: None,
                    },
                    finite_exponent: Some(ParsedExponent {
                        exponent_is_negative: false,
                        exponent_range: 4..7,
                    }),
                },
            ),
            (
                "123.456e789",
                ParsedFinite {
                    finite_buf: StrTextBuf::at_end("123.456e789"),
                    finite_significand: ParsedSignificand {
                        significand_is_negative: false,
                        significand_range: 0..7,
                        decimal_point: Some(ParsedDecimalPoint {
                            decimal_point_range: 3..4,
                        }),
                    },
                    finite_exponent: Some(ParsedExponent {
                        exponent_is_negative: false,
                        exponent_range: 8..11,
                    }),
                },
            ),
            (
                "123e-456",
                ParsedFinite {
                    finite_buf: StrTextBuf::at_end("123e-456"),
                    finite_significand: ParsedSignificand {
                        significand_is_negative: false,
                        significand_range: 0..3,
                        decimal_point: None,
                    },
                    finite_exponent: Some(ParsedExponent {
                        exponent_is_negative: true,
                        exponent_range: 5..8,
                    }),
                },
            ),
            (
                "123.456e-789",
                ParsedFinite {
                    finite_buf: StrTextBuf::at_end("123.456e-789"),
                    finite_significand: ParsedSignificand {
                        significand_is_negative: false,
                        significand_range: 0..7,
                        decimal_point: Some(ParsedDecimalPoint {
                            decimal_point_range: 3..4,
                        }),
                    },
                    finite_exponent: Some(ParsedExponent {
                        exponent_is_negative: true,
                        exponent_range: 9..12,
                    }),
                },
            ),
        ] {
            let mut parser = FiniteParser::begin(StrTextBuf::new(input));
            parser.write_str(input).expect("failed to parse");
            let parsed = parser.end().expect("failed to parse");

            assert_eq!(expected, &parsed, "{}", input);
        }
    }

    #[test]
    fn parse_inf_valid() {
        for (input, expected) in &[
            (
                "inf",
                ParsedInfinity {
                    is_infinity_negative: false,
                },
            ),
            (
                "-inf",
                ParsedInfinity {
                    is_infinity_negative: true,
                },
            ),
            (
                "+inf",
                ParsedInfinity {
                    is_infinity_negative: false,
                },
            ),
            (
                "infinity",
                ParsedInfinity {
                    is_infinity_negative: false,
                },
            ),
            (
                "-infinity",
                ParsedInfinity {
                    is_infinity_negative: true,
                },
            ),
            (
                "+infinity",
                ParsedInfinity {
                    is_infinity_negative: false,
                },
            ),
            (
                "INF",
                ParsedInfinity {
                    is_infinity_negative: false,
                },
            ),
            (
                "-INf",
                ParsedInfinity {
                    is_infinity_negative: true,
                },
            ),
            (
                "+INf",
                ParsedInfinity {
                    is_infinity_negative: false,
                },
            ),
            (
                "InFinITY",
                ParsedInfinity {
                    is_infinity_negative: false,
                },
            ),
        ] {
            let mut parser = InfinityParser::begin(StrTextBuf::new(input));
            parser.write_str(input).expect("failed to parse");
            let parsed = parser.end().expect("failed to parse");

            assert_eq!(expected, &parsed, "{}", input);
        }
    }

    #[test]
    fn parse_nan_valid() {
        for (input, expected) in &[
            (
                "nan",
                ParsedNan {
                    nan_buf: StrTextBuf::at_end("nan"),
                    nan_header: ParsedNanHeader {
                        is_nan_signaling: false,
                        is_nan_negative: false,
                    },
                    nan_payload: None,
                },
            ),
            (
                "-nan",
                ParsedNan {
                    nan_buf: StrTextBuf::at_end("-nan"),
                    nan_header: ParsedNanHeader {
                        is_nan_signaling: false,
                        is_nan_negative: true,
                    },
                    nan_payload: None,
                },
            ),
            (
                "+nan",
                ParsedNan {
                    nan_buf: StrTextBuf::at_end("+nan"),
                    nan_header: ParsedNanHeader {
                        is_nan_signaling: false,
                        is_nan_negative: false,
                    },
                    nan_payload: None,
                },
            ),
            (
                "snan",
                ParsedNan {
                    nan_buf: StrTextBuf::at_end("snan"),
                    nan_header: ParsedNanHeader {
                        is_nan_signaling: true,
                        is_nan_negative: false,
                    },
                    nan_payload: None,
                },
            ),
            (
                "-snan",
                ParsedNan {
                    nan_buf: StrTextBuf::at_end("-snan"),
                    nan_header: ParsedNanHeader {
                        is_nan_signaling: true,
                        is_nan_negative: true,
                    },
                    nan_payload: None,
                },
            ),
            (
                "+snan",
                ParsedNan {
                    nan_buf: StrTextBuf::at_end("+snan"),
                    nan_header: ParsedNanHeader {
                        is_nan_signaling: true,
                        is_nan_negative: false,
                    },
                    nan_payload: None,
                },
            ),
            (
                "NaN",
                ParsedNan {
                    nan_buf: StrTextBuf::at_end("NaN"),
                    nan_header: ParsedNanHeader {
                        is_nan_signaling: false,
                        is_nan_negative: false,
                    },
                    nan_payload: None,
                },
            ),
            (
                "SNAN",
                ParsedNan {
                    nan_buf: StrTextBuf::at_end("SNAN"),
                    nan_header: ParsedNanHeader {
                        is_nan_signaling: true,
                        is_nan_negative: false,
                    },
                    nan_payload: None,
                },
            ),
            (
                "nan(1234)",
                ParsedNan {
                    nan_buf: StrTextBuf::at_end("nan(1234)"),
                    nan_header: ParsedNanHeader {
                        is_nan_signaling: false,
                        is_nan_negative: false,
                    },
                    nan_payload: Some(ParsedSignificand {
                        significand_range: 4..8,
                        ..Default::default()
                    }),
                },
            ),
            (
                "nan()",
                ParsedNan {
                    nan_buf: StrTextBuf::at_end("nan()"),
                    nan_header: ParsedNanHeader {
                        is_nan_signaling: false,
                        is_nan_negative: false,
                    },
                    nan_payload: Some(ParsedSignificand {
                        significand_range: 4..4,
                        ..Default::default()
                    }),
                },
            ),
        ] {
            let mut parser = NanParser::begin(StrTextBuf::new(input));
            let _ = parser.write_str(input);

            let nan = parser.end().expect("failed to parse");

            assert_eq!(expected, &nan, "{}", input);
        }
    }

    #[test]
    fn parse_fails_on_buffer_too_small() {
        let expected_err = "the buffer is too small";

        let mut buf = DecimalParser::begin(ArrayTextBuf::<1>::default());
        let err = buf.parse_ascii(b"123").unwrap_err();
        assert_eq!(expected_err, &err.to_string());

        let mut buf = FiniteParser::begin(ArrayTextBuf::<1>::default());
        let err = buf.parse_ascii(b"123").unwrap_err();
        assert_eq!(expected_err, &err.to_string());

        let mut buf = InfinityParser::begin(ArrayTextBuf::<1>::default());
        let err = buf.parse_ascii(b"inf").unwrap_err();
        assert_eq!(expected_err, &err.to_string());

        let mut buf = NanParser::begin(ArrayTextBuf::<1>::default());
        let err = buf.parse_ascii(b"nan").unwrap_err();
        assert_eq!(expected_err, &err.to_string());
    }

    #[test]
    fn parse_finite_invalid() {
        for (input, expected_err) in &[("", "unexpected end of input, expected a sign or digit")] {
            let actual_err = FiniteParser::parse_str(input).unwrap_err();

            assert_eq!(expected_err, &actual_err.to_string(), "{}", input);
        }
    }

    #[test]
    fn parse_invalid() {
        for (input, expected_err) in &[
            (
                "",
                "unexpected end of input, expected a finite number, infinity, or NaN",
            ),
            (
                "-",
                "unexpected end of input, expected a finite number, infinity, or NaN",
            ),
            (
                "+",
                "unexpected end of input, expected a finite number, infinity, or NaN",
            ),
            ("1e", "unexpected end of input, expected a sign or digit"),
            ("1e-", "unexpected end of input, expected any digit"),
            ("1e+", "unexpected end of input, expected any digit"),
            ("in", "unexpected end of input, expected `f`"),
            ("n", "unexpected end of input, expected `a`"),
            ("s", "unexpected end of input, expected `n`"),
            ("nan(", "unexpected end of input, expected `)`"),
            ("nan(123", "unexpected end of input, expected `)`"),
            ("snan(", "unexpected end of input, expected `)`"),
            ("snan(123", "unexpected end of input, expected `)`"),
            (
                "x",
                "unexpected character `x`, expected a finite number, infinity, or NaN",
            ),
            (
                "-x",
                "unexpected character `x`, expected a finite number, infinity, or NaN",
            ),
            (
                "+x",
                "unexpected character `x`, expected a finite number, infinity, or NaN",
            ),
            ("1x", "unexpected character `x`, expected any digit"),
            ("1ex", "unexpected character `x`, expected any digit"),
            ("1e-x", "unexpected character `x`, expected any digit"),
            ("1e+x", "unexpected character `x`, expected any digit"),
            ("1.", "unexpected end of input, expected a sign or digit"),
            ("inx", "unexpected character `x`, expected `f`"),
            ("infinityx", "unexpected character `x`"),
            ("nx", "unexpected character `x`"),
            ("snx", "unexpected character `x`"),
            ("nan()x", "unexpected character `x`"),
            ("snan()x", "unexpected character `x`"),
            (
                "--",
                "unexpected character `-`, expected a finite number, infinity, or NaN",
            ),
            (
                "++",
                "unexpected character `+`, expected a finite number, infinity, or NaN",
            ),
            (
                "-+",
                "unexpected character `+`, expected a finite number, infinity, or NaN",
            ),
            (
                "+-",
                "unexpected character `-`, expected a finite number, infinity, or NaN",
            ),
            (
                ".",
                "unexpected character `.`, expected a finite number, infinity, or NaN",
            ),
            (
                "..",
                "unexpected character `.`, expected a finite number, infinity, or NaN",
            ),
            ("1.3.2", "unexpected character `.`, expected any digit"),
            ("1e1.1", "unexpected character `.`, expected any digit"),
            ("1-", "unexpected character `-`, expected any digit"),
            ("1+", "unexpected character `+`, expected any digit"),
            ("1e1-", "unexpected character `-`, expected any digit"),
            ("1e1+", "unexpected character `+`, expected any digit"),
            ("n(", "unexpected character `(`"),
            ("sn(", "unexpected character `(`"),
            ("nan(1.2)", "unexpected character `.`"),
            ("snan(1.2)", "unexpected character `.`"),
            ("nan(-1)", "unexpected character `-`"),
            ("snan(-1)", "unexpected character `-`"),
            ("nan(1e2", "unexpected character `e`"),
            ("snan(1e2)", "unexpected character `e`"),
        ] {
            let actual_err = DecimalParser::parse_str(input).unwrap_err();

            assert_eq!(expected_err, &actual_err.to_string(), "{}", input);
        }
    }

    #[test]
    fn parse_fmt_no_content() {
        struct Empty;

        impl fmt::Display for Empty {
            fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
                Ok(())
            }
        }

        let mut parser = DecimalParser::begin(ArrayTextBuf::<32>::default());
        parser.parse_fmt(Empty).unwrap();

        let expected_err = "unexpected end of input, expected a finite number, infinity, or NaN";

        assert_eq!(expected_err, &parser.end().unwrap_err().to_string());
    }

    #[test]
    fn parse_fmt_failing() {
        struct Failing;

        impl fmt::Display for Failing {
            fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
                Err(fmt::Error)
            }
        }

        let mut parser = DecimalParser::begin(ArrayTextBuf::<32>::default());

        let expected_err = "the source produced an error while parsing";

        assert_eq!(
            expected_err,
            &parser.parse_fmt(Failing).unwrap_err().to_string()
        );
    }
}
