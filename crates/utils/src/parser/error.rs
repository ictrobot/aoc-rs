use crate::ascii::AsciiSet;
use crate::parser::then::Then2;
use crate::parser::{ParseState, Parser, ParserResult};
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Error type returned by parsers.
///
/// Returned by both [`Parser::parse_ctx`] and [`Leaf::parse`](super::Leaf::parse).
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
    /// Expected one of $set.
    ExpectedOneOf(AsciiSet),
    /// Expected at least $n matches.
    ExpectedMatches(usize),
    /// Expected $n items or less.
    ExpectedLessItems(usize),
    /// Expected end of input.
    ExpectedEof(),
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
            ParseError::ExpectedByte(x) => write!(f, "expected {:?}", x as char),
            ParseError::ExpectedByteRange(min, max) => {
                write!(f, "expected {:?}-{:?}", min as char, max as char)
            }
            ParseError::ExpectedOneOf(set) => write!(f, "expected one of {set}"),
            ParseError::ExpectedEof() => write!(f, "expected end of input"),
            ParseError::ExpectedMatches(x) => write!(f, "expected at least {x} match"),
            ParseError::ExpectedLessItems(x) => write!(f, "expected {x} items or less"),
            ParseError::NumberTooLarge(x) => write!(f, "expected number <= {x}"),
            ParseError::NumberTooSmall(x) => write!(f, "expected number >= {x}"),
            ParseError::NumberOutOfRange() => write!(f, "number out of range"),
            ParseError::Custom(x) => f.write_str(x),
        }
    }
}

impl Error for ParseError {}

impl PartialEq<ParseError> for Box<dyn Error> {
    fn eq(&self, other: &ParseError) -> bool {
        if let Some(pe) = self.downcast_ref::<ParseError>() {
            pe == other
        } else {
            false
        }
    }
}
impl PartialEq<Box<dyn Error>> for ParseError {
    fn eq(&self, other: &Box<dyn Error>) -> bool {
        other == self
    }
}

#[derive(Copy, Clone)]
pub struct WithErrorMsg<P> {
    pub(super) parser: P,
    pub(super) message: &'static str,
}
impl<'i, P: Parser<'i>> Parser<'i> for WithErrorMsg<P> {
    type Output = P::Output;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse_ctx(
        &self,
        input: &'i [u8],
        state: &mut ParseState<'i>,
        commit: &mut bool,
        tail: bool,
    ) -> ParserResult<'i, Self::Output> {
        let prev_remaining = state.error.map(|e| e.1);
        self.parser
            .parse_ctx(input, state, commit, tail)
            .inspect_err(|_| {
                let remaining = state.error.unwrap().1;
                if prev_remaining != Some(remaining) {
                    // If the error location has changed, update the stored message
                    state.error = Some((ParseError::Custom(self.message), remaining));
                }
            })
    }
}
