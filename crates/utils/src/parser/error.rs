use crate::parser::then::Then2;
use crate::parser::{ParseResult, Parser};
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Error type returned by [`Parser::parse`].
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Expected $type.
    Expected(&'static str),
    /// Expected $literal.
    ExpectedLiteral(&'static str),
    /// Expected $byte.
    ExpectedByte(u8),
    /// Expected at least $n matches.
    ExpectedMatches(usize),
    /// Value of range for $type.
    OutOfRange(&'static str),
    /// Custom error returned by [`Parser::map_res`] & [`Parser::error_msg`].
    Custom(&'static str),
}

impl Display for ParseError {
    #[cold]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            ParseError::Expected(x) => write!(f, "expected {x}"),
            ParseError::ExpectedLiteral(x) => write!(f, "expected {x:?}"),
            ParseError::ExpectedByte(x) => write!(f, "expected {:?}", x.escape_ascii().to_string()),
            ParseError::ExpectedMatches(x) => write!(f, "expected at least {x} match"),
            ParseError::OutOfRange(x) => write!(f, "value out of range for {x}"),
            ParseError::Custom(x) => f.write_str(x),
        }
    }
}

impl Error for ParseError {}

#[derive(Copy, Clone)]
pub struct WithErrorMsg<P> {
    pub(super) parser: P,
    pub(super) message: &'static str,
}
impl<P: Parser> Parser for WithErrorMsg<P> {
    type Output<'i> = P::Output<'i>;
    type Then<T: Parser> = Then2<Self, T>;

    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
        match self.parser.parse(input) {
            Ok(v) => Ok(v),
            Err((_, pos)) => Err((ParseError::Custom(self.message), pos)),
        }
    }

    fn then<T: Parser>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}
