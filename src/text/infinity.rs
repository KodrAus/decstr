use crate::{
    text::{
        ParsedInfinity,
        TextWriter,
    },
    ParseError,
};
use core::fmt::{
    self,
    Write,
};

#[derive(Debug)]
pub struct InfinityParser<B> {
    expecting: &'static [u8],
    infinity: ParsedInfinity,
    buf: B,
    error: Option<ParseError>,
}

const INFINITY_BUF_EXPECTING: &'static [u8] = b"infinity";

impl<B: TextWriter> InfinityParser<B> {
    pub fn begin(buf: B) -> Self {
        InfinityParser {
            buf,
            error: None,
            expecting: INFINITY_BUF_EXPECTING,
            infinity: ParsedInfinity {
                is_infinity_negative: false,
            },
        }
    }

    pub fn advance(&mut self, b: u8) {
        self.expecting = &self.expecting[1..];

        self.buf.advance_significand(b);
    }

    pub fn infinity_is_negative(&mut self) {
        self.infinity.is_infinity_negative = true;
    }

    pub fn parse_fmt(&mut self, f: impl fmt::Display) -> Result<(), ParseError> {
        write!(self, "{}", f).map_err(|err| self.unwrap_context(err))
    }

    pub fn parse_ascii(&mut self, ascii: &[u8]) -> Result<(), ParseError> {
        for b in ascii {
            match b {
                b'-' if self.is_at_start() => {
                    self.infinity.is_infinity_negative = true;
                    self.buf.advance_significand(*b);
                }
                c if !self.expecting.is_empty() && self.expecting[0].eq_ignore_ascii_case(c) => {
                    self.expecting = &self.expecting[1..];
                    self.buf.advance_significand(*b);
                }
                c => return Err(ParseError::unexpected_char(*c, &[], "")),
            }
        }

        Ok(())
    }

    pub fn end(self) -> Result<ParsedInfinity, ParseError> {
        debug_assert!(
            self.error.is_none(),
            "attempt to complete a parser with an error context"
        );

        match self.expecting {
            // If we just encounter `inf` then we still have a valid infinity
            b"" | b"inity" => Ok(self.infinity),
            _ => Err(ParseError::unexpected_end(&self.expecting[0..1], "")),
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
        self.expecting.len() == INFINITY_BUF_EXPECTING.len()
    }
}

impl<B: TextWriter> Write for InfinityParser<B> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.parse_ascii(s.as_bytes())
            .map_err(|err| self.context(err))
    }
}
