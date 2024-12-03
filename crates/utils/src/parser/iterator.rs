use crate::input::InputError;
use crate::parser::Parser;
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

impl<'a, P: Parser> Iterator for ParserIterator<'a, P> {
    type Item = Result<P::Output<'a>, InputError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }

        match self.parser.parse(self.remaining) {
            Ok((v, remaining)) => {
                self.remaining = remaining;
                Some(Ok(v))
            }
            Err((err, remaining)) => {
                self.remaining = &[]; // Ensure future calls return None
                Some(Err(InputError::new(self.input, remaining, err)))
            }
        }
    }
}

impl<P: Parser> FusedIterator for ParserIterator<'_, P> {}

/// An iterator which returns successful parse outputs only, skipping over errors.
///
/// See [`Parser::matches_iterator`].
#[derive(Copy, Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct ParserMatchesIterator<'a, P> {
    pub(super) remaining: &'a [u8],
    pub(super) parser: P,
}

impl<'a, P: Parser> Iterator for ParserMatchesIterator<'a, P> {
    type Item = P::Output<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while !self.remaining.is_empty() {
            if let Ok((v, remaining)) = self.parser.parse(self.remaining) {
                self.remaining = remaining;
                return Some(v);
            }
            self.remaining = &self.remaining[1..];
        }
        None
    }
}

impl<P: Parser> FusedIterator for ParserMatchesIterator<'_, P> {}
