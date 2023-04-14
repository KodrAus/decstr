use crate::{
    text::{
        ParsedNan,
        ParsedNanHeader,
        ParsedSignificand,
        TextWriter,
    },
    ParseError,
};

use core::{
    fmt::{
        self,
        Write,
    },
    str,
};

#[derive(Debug)]
pub struct NanParser<B> {
    buf: NanBuf<B>,
    error: Option<ParseError>,
    header: ParsedNanHeader,
    payload: Option<ParsedSignificand>,
}

#[derive(Debug)]
struct NanBuf<B> {
    expecting: &'static [u8],
    buf: B,
}

const NAN_BUF_EXPECTING: &[u8] = b"snan()";

impl<B: TextWriter> NanParser<B> {
    pub fn begin(buf: B) -> Self {
        NanParser {
            buf: NanBuf::begin(buf),
            error: None,
            header: ParsedNanHeader {
                is_nan_signaling: false,
                is_nan_negative: false,
            },
            payload: None,
        }
    }

    pub fn parse(buf: B, input: impl fmt::Display) -> Result<ParsedNan<B>, ParseError> {
        let mut parser = NanParser::begin(buf);

        match write!(&mut parser, "{}", input) {
            Ok(()) => parser.end(),
            Err(err) => Err(parser.unwrap_context(err)),
        }
    }

    pub fn nan_is_positive(&mut self, b: u8) {
        self.buf.nan_is_positive(&mut self.header, b)
    }

    pub fn nan_is_negative(&mut self, b: u8) {
        self.buf.nan_is_negative(&mut self.header, b)
    }

    pub fn nan_is_quiet(&mut self, b: u8) {
        self.buf.nan_is_quiet(&mut self.header, b)
    }

    pub fn nan_is_signaling(&mut self, b: u8) {
        self.buf.nan_is_signaling(&mut self.header, b)
    }

    pub fn parse_fmt(&mut self, f: impl fmt::Display) -> Result<(), ParseError> {
        write!(self, "{}", f).map_err(|err| self.unwrap_context(err))
    }

    pub fn parse_ascii(&mut self, ascii: &[u8]) -> Result<(), ParseError> {
        if let Some(remaining_capacity) = self.buf.remaining_capacity() {
            if remaining_capacity < ascii.len() {
                unimplemented!("return an appropriate `ParseError` kind");
            }
        }

        for b in ascii {
            match b {
                // Parse a digit in the payload
                b'0'..=b'9' if self.payload.is_some() => {
                    self.buf
                        .push_payload_digit(self.payload.as_mut().expect("missing buffer"), *b);
                }
                // Mark the NaN as negative
                b'-' if self.is_at_start() => {
                    self.buf.nan_is_negative(&mut self.header, *b);
                }
                // Uncommon: Mark the NaN as positive
                b'+' if self.is_at_start() => {
                    self.buf.nan_is_positive(&mut self.header, *b);
                }
                // Skip over the leading `s` in `snan`
                b'n' | b'N' if self.is_at_start() => {
                    self.buf.nan_is_quiet(&mut self.header, *b);
                }
                // Mark the NaN as signaling
                b's' | b'S' if self.is_at_start() => {
                    self.buf.nan_is_signaling(&mut self.header, *b);
                }
                // Begin the payload
                b'(' if self.buf.expecting(b'(') => {
                    self.payload = Some(self.buf.begin_payload(*b));
                }
                // Complete the payload
                b')' if self.buf.expecting(b')') => {
                    self.buf.end_payload(*b);
                }
                // Advance through the set of expected chars
                c if self.buf.expecting(*c) => {
                    self.buf.advance(*b);
                }
                // Any other character is invalid
                c => return Err(ParseError::unexpected_char(*c, "")),
            }
        }

        Ok(())
    }

    pub fn end(self) -> Result<ParsedNan<B>, ParseError> {
        debug_assert!(
            self.error.is_none(),
            "attempt to complete a parser with an error context"
        );

        match (self.buf.expecting, self.payload) {
            (
                b"",
                Some(ParsedSignificand {
                    significand_is_negative: false,
                    significand_range,
                    decimal_point: None,
                }),
            ) => Ok(ParsedNan {
                nan_buf: self.buf.buf,
                nan_header: self.header,
                nan_payload: Some(ParsedSignificand {
                    significand_is_negative: false,
                    significand_range,
                    decimal_point: None,
                }),
            }),
            (b"()", None) => Ok(ParsedNan {
                nan_buf: self.buf.buf,
                nan_header: self.header,
                nan_payload: None,
            }),
            _ => Err(ParseError::unexpected_end(
                str::from_utf8(&self.buf.expecting[0..1]).unwrap(),
            )),
        }
    }

    pub fn context(&mut self, err: ParseError) -> fmt::Error {
        self.error = Some(err);
        fmt::Error
    }

    pub fn unwrap_context(&mut self, _: fmt::Error) -> ParseError {
        self.error.take().expect("missing error context")
    }

    fn is_at_start(&self) -> bool {
        self.buf.expecting.len() == NAN_BUF_EXPECTING.len()
    }
}

impl<B: TextWriter> Write for NanParser<B> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.parse_ascii(s.as_bytes())
            .map_err(|err| self.context(err))
    }
}

impl<B: TextWriter> NanBuf<B> {
    pub(crate) fn begin(buf: B) -> Self {
        NanBuf {
            expecting: NAN_BUF_EXPECTING,
            buf,
        }
    }

    pub fn nan_is_positive(&mut self, header: &mut ParsedNanHeader, b: u8) {
        header.is_nan_negative = false;

        self.buf.advance_significand(b);
    }

    pub fn nan_is_negative(&mut self, header: &mut ParsedNanHeader, b: u8) {
        header.is_nan_negative = true;

        self.buf.advance_significand(b);
    }

    pub fn nan_is_quiet(&mut self, header: &mut ParsedNanHeader, b: u8) {
        header.is_nan_signaling = false;

        self.expecting = &self.expecting[2..];
        self.buf.advance_significand(b);
    }

    pub fn nan_is_signaling(&mut self, header: &mut ParsedNanHeader, b: u8) {
        header.is_nan_signaling = true;

        self.expecting = &self.expecting[1..];
        self.buf.advance_significand(b);
    }

    pub fn begin_payload(&mut self, b: u8) -> ParsedSignificand {
        self.expecting = &self.expecting[1..];
        self.buf.advance_significand(b);

        self.buf.begin_significand()
    }

    pub fn remaining_capacity(&self) -> Option<usize> {
        self.buf.remaining_capacity()
    }

    pub fn push_payload_digit(&mut self, significand: &mut ParsedSignificand, digit: u8) {
        self.buf.push_significand_digit(significand, digit)
    }

    pub fn end_payload(&mut self, b: u8) {
        self.expecting = &self.expecting[1..];
        self.buf.advance_significand(b);
    }

    pub fn expecting(&self, b: u8) -> bool {
        !self.expecting.is_empty() && self.expecting[0].eq_ignore_ascii_case(&b)
    }

    pub fn advance(&mut self, b: u8) {
        self.expecting = &self.expecting[1..];
        self.buf.advance_significand(b);
    }
}
