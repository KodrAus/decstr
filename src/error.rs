use core::fmt;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Error {
            kind: ErrorKind::Parse(err),
        }
    }
}

impl From<OverflowError> for Error {
    fn from(err: OverflowError) -> Self {
        Error {
            kind: ErrorKind::Overflow(err),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::Parse(ref err) => fmt::Display::fmt(err, f),
            ErrorKind::Overflow(ref err) => fmt::Display::fmt(err, f),
        }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub enum ErrorKind {
    Parse(ParseError),
    Overflow(OverflowError),
}

#[derive(Debug)]
pub struct ParseError {
    kind: ParseErrorKind,
    expected: &'static str,
}

#[derive(Debug)]
enum ParseErrorKind {
    Char { got: u8 },
    End,
    BufferTooSmall,
}

impl ParseError {
    pub(crate) fn buffer_too_small() -> Self {
        ParseError {
            expected: "",
            kind: ParseErrorKind::BufferTooSmall,
        }
    }

    pub(crate) fn unexpected_char(got: u8, expected: &'static str) -> Self {
        ParseError {
            expected,
            kind: ParseErrorKind::Char { got },
        }
    }

    pub(crate) fn unexpected_end(expected: &'static str) -> Self {
        ParseError {
            expected,
            kind: ParseErrorKind::End,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ParseErrorKind::Char { got } => {
                write!(f, "unexpected character `{}`", got as char)?;
            }
            ParseErrorKind::End => {
                write!(f, "unexpected end of input")?;
            }
            ParseErrorKind::BufferTooSmall => {
                write!(f, "the buffer is too small")?;
            }
        };

        if self.expected.len() == 1 {
            write!(f, ", expected `{}`", self.expected)?;
        } else if self.expected.len() > 0 {
            write!(f, ", expected {}", self.expected)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct OverflowError {
    max_width_bytes: usize,
    required_width_bytes: Option<usize>,
    note: &'static str,
}

impl fmt::Display for OverflowError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "the value cannot fit into a decimal of `{}` bytes",
            self.max_width_bytes
        )?;

        if let Some(required_width_bytes) = self.required_width_bytes {
            write!(f, "; the width needed is `{}` bytes", required_width_bytes)?;
        }

        if self.note.len() > 0 {
            write!(f, "; {}", self.note)?;
        }

        Ok(())
    }
}

impl OverflowError {
    pub(crate) fn would_overflow(
        max_width_bytes: usize,
        required_width_bytes: usize,
    ) -> OverflowError {
        OverflowError {
            max_width_bytes,
            required_width_bytes: Some(required_width_bytes),
            note: "",
        }
    }

    pub(crate) fn exact_size_mismatch(
        got_width_bytes: usize,
        required_width_bytes: usize,
        note: &'static str,
    ) -> OverflowError {
        OverflowError {
            max_width_bytes: got_width_bytes,
            required_width_bytes: Some(required_width_bytes),
            note,
        }
    }

    pub(crate) fn exponent_out_of_range(
        max_width_bytes: usize,
        note: &'static str,
    ) -> OverflowError {
        OverflowError {
            max_width_bytes,
            required_width_bytes: None,
            note,
        }
    }

    pub fn max_width_bytes(&self) -> usize {
        self.max_width_bytes
    }

    pub fn required_width_bytes(&self) -> Option<usize> {
        self.required_width_bytes
    }
}

#[cfg(feature = "std")]
mod std_support {
    use super::*;

    use std::error;

    impl error::Error for Error {}

    impl error::Error for ParseError {}

    impl error::Error for OverflowError {}
}
