use crate::input::InputError;
use crate::parser::{ParseState, Parser};
use std::iter::FusedIterator;

/// An iterator that lazily parses the input using the provided parser.
///
/// See [`Parser::parse_iterator`].
#[derive(Copy, Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct ParserIterator<'a, P> {
    pub(super) input: &'a str,
    pub(super) remaining: &'a [u8],
    pub(super) parser: P,
}

impl<'a, P: Parser<'a>> Iterator for ParserIterator<'a, P> {
    type Item = Result<P::Output, InputError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }

        // Don't use parse_once so errors are reported at the correct position in the overall input.
        let mut state = ParseState::default();
        if let Ok((v, remaining)) =
            self.parser
                .parse_ctx(self.remaining, &mut state, &mut false, false)
        {
            self.remaining = remaining;
            Some(Ok(v))
        } else {
            self.remaining = &[]; // Ensure future calls return None
            Some(Err(state.into_input_error(self.input)))
        }
    }
}

impl<'a, P: Parser<'a>> FusedIterator for ParserIterator<'a, P> {}

impl<'a, P: Parser<'a>> ParserIterator<'a, P> {
    /// Returns the remaining input.
    ///
    /// # Examples
    /// ```
    /// # use utils::input::InputError;
    /// # use utils::parser::{self, Parser};
    /// let mut iterator = parser::u32()
    ///     .with_eol()
    ///     .parse_iterator("12\n34\n56\n78");
    /// assert_eq!(iterator.next().unwrap().unwrap(), 12);
    /// assert_eq!(iterator.next().unwrap().unwrap(), 34);
    /// assert_eq!(iterator.remaining(), b"56\n78");
    /// ```
    #[inline]
    pub fn remaining(&self) -> &'a [u8] {
        self.remaining
    }
}

/// An iterator which returns successful parse outputs only, skipping over errors.
///
/// See [`Parser::matches_iterator`].
#[derive(Copy, Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct ParserMatchesIterator<'a, P> {
    pub(super) remaining: &'a [u8],
    pub(super) parser: P,
}

impl<'a, P: Parser<'a>> Iterator for ParserMatchesIterator<'a, P> {
    type Item = P::Output;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while !self.remaining.is_empty() {
            // Use parse_ctx to avoid constructing full InputError instances which is expensive
            if let Ok((v, remaining)) = self.parser.parse_ctx(
                self.remaining,
                &mut ParseState::default(),
                &mut false,
                false,
            ) {
                self.remaining = remaining;
                return Some(v);
            }
            self.remaining = &self.remaining[1..];
        }
        None
    }
}

impl<'a, P: Parser<'a>> FusedIterator for ParserMatchesIterator<'a, P> {}

impl<'a, P: Parser<'a>> ParserMatchesIterator<'a, P> {
    /// Returns the remaining input.
    ///
    /// # Examples
    /// ```
    /// # use utils::input::InputError;
    /// # use utils::parser::{self, Parser};
    /// let mut iterator = parser::u32()
    ///     .matches_iterator("abc123d456e7xyz");
    /// assert_eq!(iterator.next().unwrap(), 123);
    /// assert_eq!(iterator.next().unwrap(), 456);
    /// assert_eq!(iterator.remaining(), b"e7xyz");
    /// ```
    #[inline]
    pub fn remaining(&self) -> &'a [u8] {
        self.remaining
    }
}
