use crate::parser::then::Then2;
use crate::parser::{ParseError, ParseState, Parser, ParserResult};

/// [`Result`] type returned by [`Leaf::parse`].
///
/// Contains the error information inline, unlike [`ParserResult`].
pub type LeafResult<'i, T> = Result<(T, &'i [u8]), (ParseError, &'i [u8])>;

/// Trait implemented by atomic, fail‑fast parsers.
///
/// Leaf parsers always return the first error they encounter without backtracking.
#[must_use]
pub trait Leaf<'i>: Sized {
    /// Type of the value produced by [`parse`](Self::parse) when successful.
    type Output;

    /// Parse one item, returning immediately on error without backtracking.
    ///
    /// Returns a tuple of the successfully parsed [`Output`](Self::Output) value and the
    /// remaining bytes, or a tuple containing a [`ParseError`] and the location of the error.
    fn parse(&self, input: &'i [u8]) -> LeafResult<'i, Self::Output>;
}

impl<'i, L: Leaf<'i>> Parser<'i> for L {
    type Output = L::Output;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    fn parse_ctx(
        &self,
        input: &'i [u8],
        state: &mut ParseState<'i>,
        _: &mut bool,
        _: bool,
    ) -> ParserResult<'i, Self::Output> {
        match self.parse(input) {
            Ok(v) => Ok(v),
            Err((err, remaining)) => Err(state.error(err, remaining)),
        }
    }
}

/// Matches the string literal exactly.
///
/// Normally used with [`with_prefix`](Parser::with_prefix) or [`with_suffix`](Parser::with_suffix).
impl<'i> Leaf<'i> for &'static str {
    type Output = ();

    #[inline]
    fn parse(&self, input: &'i [u8]) -> LeafResult<'i, Self::Output> {
        // This is faster than using strip_prefix for the common case where the string is a short
        // string literal known at compile time.
        if input.len() >= self.len() && self.bytes().zip(input).all(|(a, &b)| a == b) {
            Ok(((), &input[self.len()..]))
        } else {
            Err((ParseError::ExpectedLiteral(self), input))
        }
    }
}

/// Matches the byte exactly.
///
/// Normally used with [`with_prefix`](Parser::with_prefix) or [`with_suffix`](Parser::with_suffix).
impl<'i> Leaf<'i> for u8 {
    type Output = ();

    #[inline]
    fn parse(&self, input: &'i [u8]) -> LeafResult<'i, Self::Output> {
        if input.first() == Some(self) {
            Ok(((), &input[1..]))
        } else {
            Err((ParseError::ExpectedByte(*self), input))
        }
    }
}

/// Allow custom functions/closures to be used as [`Leaf`] parsers.
impl<'i, F: Fn(&'i [u8]) -> LeafResult<'i, O>, O> Leaf<'i> for F {
    type Output = O;

    #[inline]
    fn parse(&self, input: &'i [u8]) -> LeafResult<'i, Self::Output> {
        self(input)
    }
}

/// Coerces the provided function/closure into a [`Leaf`] parser.
///
/// This function exists to help with type inference.
#[inline]
#[must_use]
pub const fn from_leaf_fn<'i, F, O>(f: F) -> F
where
    F: Fn(&'i [u8]) -> LeafResult<'i, O>,
    F: Leaf<'i, Output = O>,
{
    f
}

/// Trait for types that have a canonical [`Leaf`] parser.
pub trait Parseable {
    type Parser: for<'i> Leaf<'i, Output = Self>;
    const PARSER: Self::Parser;
}
