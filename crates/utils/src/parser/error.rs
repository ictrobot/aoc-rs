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
    /// Expected $min - $max.
    ExpectedByteRange(u8, u8),
    /// Expected at least $n matches.
    ExpectedMatches(usize),
    /// Expected number <= $num.
    NumberTooLarge(i128),
    /// Expected number >= $num.
    NumberTooSmall(i128),
    /// Number out of range.
    ///
    /// Used as a fallback if min/max bound doesn't fit in an [`i128`] (for example, [`u128::MAX`]).
    NumberOutOfRange(),
    /// Custom error returned by [`Parser::map_res`] & [`Parser::error_msg`].
    Custom(&'static str),
}

impl ParseError {
    #[inline]
    pub(super) fn too_large(max: impl TryInto<i128>) -> Self {
        if let Ok(max) = max.try_into() {
            Self::NumberTooLarge(max)
        } else {
            Self::NumberOutOfRange()
        }
    }

    #[inline]
    pub(super) fn too_small(min: impl TryInto<i128>) -> Self {
        if let Ok(min) = min.try_into() {
            Self::NumberTooSmall(min)
        } else {
            Self::NumberOutOfRange()
        }
    }
}

impl Display for ParseError {
    #[cold]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            ParseError::Expected(x) => write!(f, "expected {x}"),
            ParseError::ExpectedLiteral(x) => write!(f, "expected {x:?}"),
            ParseError::ExpectedByte(x) => write!(f, "expected {:?}", x.escape_ascii().to_string()),
            ParseError::ExpectedByteRange(min, max) => {
                write!(
                    f,
                    "expected {:?}-{:?}",
                    min.escape_ascii().to_string(),
                    max.escape_ascii().to_string(),
                )
            }
            ParseError::ExpectedMatches(x) => write!(f, "expected at least {x} match"),
            ParseError::NumberTooLarge(x) => write!(f, "expected number <= {x}"),
            ParseError::NumberTooSmall(x) => write!(f, "expected number >= {x}"),
            ParseError::NumberOutOfRange() => write!(f, "number out of range"),
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
