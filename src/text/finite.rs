use crate::{
    text::{
        ParsedExponent,
        ParsedFinite,
        ParsedSignificand,
        StrTextBuf,
        TextWriter,
    },
    ParseError,
};
use core::fmt::{
    self,
    Write,
};

/**
A parser for a formatted decimal number.
*/
#[derive(Debug)]
pub struct FiniteParser<B> {
    buf: B,
    error: Option<ParseError>,
    significand: ParsedSignificand,
    exponent: Option<ParsedExponent>,
    has_sign: bool,
    has_decimal: bool,
    has_digits: bool,
}

impl<'a> FiniteParser<StrTextBuf<'a>> {
    pub fn parse_str(input: &'a str) -> Result<ParsedFinite<StrTextBuf<'a>>, ParseError> {
        let mut parser = FiniteParser::begin(StrTextBuf::new(input));

        parser.parse_ascii(input.as_bytes())?;

        parser.end()
    }
}

impl<B: TextWriter> FiniteParser<B> {
    pub fn begin(mut buf: B) -> Self {
        FiniteParser {
            significand: buf.begin_significand(),
            exponent: None,
            buf,
            error: None,
            has_sign: false,
            has_decimal: false,
            has_digits: false,
        }
    }

    pub fn checked_push_significand_digit(&mut self, digit: u8) -> Result<(), ParseError> {
        if self.buf.remaining_capacity() == Some(0) {
            return Err(ParseError::buffer_too_small());
        }

        Ok(self.push_significand_digit(digit))
    }

    pub(in crate::text) fn push_significand_digit(&mut self, digit: u8) {
        self.has_digits = true;

        self.buf
            .push_significand_digit(&mut self.significand, digit)
    }

    pub fn checked_significand_is_negative(&mut self) -> Result<(), ParseError> {
        if self.buf.remaining_capacity() == Some(0) {
            return Err(ParseError::buffer_too_small());
        }

        Ok(self.significand_is_negative())
    }

    pub(in crate::text) fn significand_is_negative(&mut self) {
        self.has_sign = true;

        self.buf.significand_is_negative(&mut self.significand)
    }

    pub(in crate::text) fn significand_is_positive(&mut self) {
        self.has_sign = true;

        self.buf.significand_is_positive(&mut self.significand)
    }

    pub(in crate::text) fn push_significand_decimal_point(&mut self) {
        self.has_decimal = true;
        self.has_digits = false;

        self.buf
            .push_significand_decimal_point(&mut self.significand)
    }

    pub fn checked_begin_exponent(&mut self) -> Result<(), ParseError> {
        if self.buf.remaining_capacity() == Some(0) {
            return Err(ParseError::buffer_too_small());
        }

        Ok(self.begin_exponent())
    }

    pub(in crate::text) fn begin_exponent(&mut self) {
        self.has_sign = false;
        self.has_digits = false;

        self.exponent = Some(self.buf.begin_exponent());
    }

    pub fn parse(buf: B, input: impl fmt::Display) -> Result<ParsedFinite<B>, ParseError> {
        let mut parser = FiniteParser::begin(buf);

        match write!(&mut parser, "{}", input) {
            Ok(()) => parser.end(),
            Err(err) => Err(parser.unwrap_context(err)),
        }
    }

    pub fn parse_fmt(&mut self, f: impl fmt::Display) -> Result<(), ParseError> {
        write!(self, "{}", f).map_err(|err| self.unwrap_context(err))
    }

    pub fn parse_ascii(&mut self, mut ascii: &[u8]) -> Result<(), ParseError> {
        if let Some(remaining_capacity) = self.buf.remaining_capacity() {
            if remaining_capacity < ascii.len() {
                return Err(ParseError::buffer_too_small());
            }
        }

        // If there's no exponent then parse the significand
        // The number may be split across multiple calls to `write_str`
        if self.exponent.is_none() {
            while ascii.len() > 0 {
                match ascii[0] {
                    // Push a digit to the significand
                    b'0'..=b'9' => {
                        self.push_significand_digit(ascii[0]);
                    }
                    // Mark the significand as negative
                    b'-' if !self.has_sign && !self.has_digits => {
                        self.significand_is_negative();
                    }
                    // Mark the decimal point in the significand
                    b'.' if !self.has_decimal => {
                        self.push_significand_decimal_point();
                    }
                    // Begin the exponent
                    // This will break out of this loop and start parsing the digits
                    // of the exponent instead
                    b'e' | b'E' if self.has_digits => {
                        self.begin_exponent();

                        ascii = &ascii[1..];
                        break;
                    }
                    // Uncommon: mark the significand as positive
                    b'+' if !self.has_sign && !self.has_digits => {
                        self.significand_is_positive();
                    }
                    // Any other character is an error
                    c => return Err(ParseError::unexpected_char(c, "any digit")),
                }

                ascii = &ascii[1..];
            }
        }

        // If there's an exponent then parse its digits
        // The format for the exponent is simpler than the significand
        // It's really just a simple integer
        if let Some(ref mut exponent) = self.exponent {
            while ascii.len() > 0 {
                match ascii[0] {
                    // Push a digit to the exponent
                    b'0'..=b'9' => {
                        self.has_digits = true;
                        self.buf.push_exponent_digit(exponent, ascii[0]);
                    }
                    // Mark the exponent as negative
                    b'-' if !self.has_sign && !self.has_digits => {
                        self.has_sign = true;
                        self.buf.exponent_is_negative(exponent);
                    }
                    // Uncommon: mark the exponent as positive
                    b'+' if !self.has_sign && !self.has_digits => {
                        self.has_sign = true;
                        self.buf.exponent_is_positive(exponent);
                    }
                    // Any other character is an error
                    c => return Err(ParseError::unexpected_char(c, "any digit")),
                }

                ascii = &ascii[1..];
            }
        }

        // We can't guarantee at this point that the number is complete
        // It may be streamed through multiple calls to `write_str`.
        Ok(())
    }

    pub fn end(self) -> Result<ParsedFinite<B>, ParseError> {
        debug_assert!(
            self.error.is_none(),
            "attempt to complete a parser with an error context"
        );

        if !self.has_digits {
            return Err(ParseError::unexpected_end(if !self.has_sign {
                "a sign or digit"
            } else {
                "any digit"
            }));
        }

        Ok(ParsedFinite {
            finite_buf: self.buf,
            finite_significand: self.significand,
            finite_exponent: self.exponent,
        })
    }

    pub fn context(&mut self, err: ParseError) -> fmt::Error {
        self.error = Some(err);
        fmt::Error
    }

    pub fn unwrap_context(&mut self, _: fmt::Error) -> ParseError {
        self.error.take().unwrap_or_else(ParseError::source)
    }
}

impl<B: TextWriter> Write for FiniteParser<B> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.parse_ascii(s.as_bytes())
            .map_err(|err| self.context(err))
    }
}
