use crate::input::InputError;
use crate::parser::combinator::{
    Commit, Map, MapResult, Optional, Or, RepeatArrayVec, RepeatFold, RepeatN, RepeatVec,
    WithConsumed, WithPrefix, WithSuffix,
};
use crate::parser::error::{ParseError, WithErrorMsg};
use crate::parser::iterator::{ParserIterator, ParserMatchesIterator};
use crate::parser::simple::{Constant, Eol};
use crate::parser::then::{Then, Then2, Unimplemented};

/// [`Result`] type returned by [`Parser::parse_ctx`].
pub type ParserResult<'i, T> = Result<(T, &'i [u8]), ErrToken>;

/// Trait implemented by all parsers.
///
/// Unlike [`Leaf`](super::Leaf) parsers, parsers implementing this trait may handle branching and
/// error recovery, including backtracking and commit.
///
/// Every [`Leaf`](super::Leaf) parser is also a [`Parser`] via a blanket implementation.
#[must_use]
pub trait Parser<'i>: Sized {
    /// Type of the value produced by [`parse`](Self::parse_ctx) when successful.
    type Output;

    /// Type of the chained parser returned by [`then`](Self::then).
    ///
    /// This is used to allow multiple [`then`](Self::then) calls to extend one tuple, instead of
    /// nesting tuples inside each other.
    type Then<T: Parser<'i>>: Then<'i, Self, T>;

    /// Parse the provided bytes.
    ///
    /// Returns a tuple of the successfully parsed [`Output`](Self::Output) value and the
    /// remaining bytes, or an [`ErrToken`] representing that an error was pushed into the provided
    /// [`ParseState`].
    fn parse_ctx(
        &self,
        input: &'i [u8],
        state: &mut ParseState<'i>,
        commit: &mut bool,
        tail: bool,
    ) -> ParserResult<'i, Self::Output>;

    // Provided methods

    /// Sequence another parser after this one.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, ParseState, Parser};
    /// assert!(matches!(
    ///     parser::i32()
    ///         .then(parser::i32())
    ///         .parse_complete("123-123"),
    ///     Ok((123, -123)),
    /// ));
    /// ```
    #[inline]
    fn then<T: Parser<'i>>(self, next: T) -> Self::Then<T> {
        Then::then(self, next)
    }

    /// Attempt to parse using this parser, followed by the provided parser.
    ///
    /// The second parser will not be tried if the first parser commits.
    ///
    /// See also [`parser::one_of`](super::one_of()).
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, ParseError, Parser};
    /// let parser = parser::u8()
    ///     .map(|x| u32::from(x) * 1001001)
    ///     .or(parser::u32());
    /// assert!(matches!(
    ///     parser.parse_complete("123"),
    ///     Ok(123123123)
    /// ));
    /// assert!(matches!(
    ///     parser.parse_complete("1000"),
    ///     Ok(1000)
    /// ));
    /// ```
    #[inline]
    fn or<T: Parser<'i, Output = Self::Output>>(self, alternative: T) -> Or<Self, T> {
        Or {
            first: self,
            second: alternative,
        }
    }

    /// Prevent backtracking past this parser if it succeeds and consumes input.
    ///
    /// After committing, any later errors within the current alternative are treated as fatal,
    /// preventing the current innermost alternative parser from trying other branches.
    ///
    /// This can be used to help ensure error messages are clear in cases where returning the
    /// furthest error from another branch would be misleading.
    /// It can also improve performance in error cases.
    ///
    /// If this parser is not inside an alternative, this has no effect.
    ///
    /// The following alternative parsers honor commit to prevent backtracking:
    /// - [`Parser::optional`]
    /// - [`Parser::or`]
    /// - [`Parser::repeat_arrayvec`]
    /// - [`Parser::repeat`]
    /// - [`parser::one_of`](super::one_of())
    /// - [`parser::parse_tree`](super::parse_tree)
    #[inline]
    fn commit(self) -> Commit<Self> {
        Commit { parser: self }
    }

    /// Map the output of this parser using the supplied function.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// assert!(matches!(
    ///     parser::u32()
    ///         .map(|x| x * 2)
    ///         .parse_complete("123"),
    ///     Ok(246)
    /// ));
    /// ```
    ///
    /// Closure that returns a value borrowing from both its input and an outer variable:
    /// ```
    /// # use utils::parser::{self, Parser};
    /// let my_string = String::from("123");
    /// let my_vec = vec![4, 5, 6];
    /// assert!(matches!(
    ///     parser::take_while(u8::is_ascii_digit)
    ///         .map(|x| (x, my_vec.as_slice()))
    ///         .parse_complete(&my_string),
    ///     Ok((&[b'1', b'2', b'3'], &[4, 5, 6]))
    /// ));
    /// ```
    #[inline]
    fn map<O, F: Fn(Self::Output) -> O>(self, f: F) -> Map<Self, F> {
        Map {
            parser: self,
            map_fn: f,
        }
    }

    /// Map the output of this parser using the supplied fallible function.
    ///
    /// Errors must be `&'static str`, which will be mapped to [`ParseError::Custom`].
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, ParseError, Parser};
    /// let parser = parser::u8()
    ///     .map_res(|x| x.checked_mul(2).ok_or("input too large"));
    /// assert!(matches!(
    ///     parser.parse_complete("123"),
    ///     Ok(246)
    /// ));
    /// assert_eq!(
    ///     parser.parse_complete("200").unwrap_err().into_source(),
    ///     ParseError::Custom("input too large"),
    /// );
    /// ```
    ///
    /// Closure that returns a value borrowing from both its input and an outer variable:
    /// ```
    /// # use utils::parser::{self, Parser};
    /// let my_string = String::from("123");
    /// let my_vec = vec![4, 5, 6];
    /// assert!(matches!(
    ///     parser::take_while(u8::is_ascii_digit)
    ///         .map_res(|x| {
    ///             if x.len() < 10 {
    ///                 Ok((x, my_vec.as_slice()))
    ///             } else {
    ///                 Err("expected fewer than 10 digits")
    ///             }
    ///         })
    ///         .parse_complete(&my_string),
    ///     Ok((&[b'1', b'2', b'3'], &[4, 5, 6]))
    /// ));
    /// ```
    #[inline]
    fn map_res<O, F: Fn(Self::Output) -> Result<O, &'static str>>(
        self,
        f: F,
    ) -> MapResult<Self, F> {
        MapResult {
            parser: self,
            map_fn: f,
        }
    }

    /// Wrap [`Output`](Self::Output) in [`Option`], returning [`None`] on error unless the parser
    /// commits.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, ParseError, Parser};
    /// let parser = parser::u32()
    ///     .optional();
    /// assert!(matches!(
    ///     parser.parse_first("123"),
    ///     Ok((Some(123), &[]))
    /// ));
    /// assert!(matches!(
    ///     parser.parse_first("abc"),
    ///     Ok((None, &[b'a', b'b', b'c']))
    /// ));
    /// ```
    #[inline]
    fn optional(self) -> Optional<Self> {
        Optional { parser: self }
    }

    /// Repeat this parser `N` times, returning an [`array`].
    ///
    /// If the number of items is variable use [`repeat_arrayvec`](Self::repeat_arrayvec) or
    /// [`repeat`](Self::repeat).
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// assert!(matches!(
    ///     parser::u32()
    ///         .repeat_n(",") // N = 3 is inferred
    ///         .parse_complete("12,34,56"),
    ///     Ok([12, 34, 56])
    /// ));
    /// ```
    #[inline]
    fn repeat_n<const N: usize, S: Parser<'i>>(self, separator: S) -> RepeatN<N, Self, S>
    where
        Self::Output: Default,
    {
        RepeatN {
            parser: self,
            separator,
        }
    }

    /// Repeat this parser while it matches, returning a [`ArrayVec`](crate::array::ArrayVec).
    ///
    /// This parser can parse up to `N` items. If more items match, it will return an error.
    ///
    /// Returns error if the item parser commits and fails, or if the separator commits and then
    /// either it or the next item fails.
    ///
    /// See [`repeat`](Self::repeat) if the upper bound is large or not known, and
    /// [`repeat_n`](Self::repeat_n) if the number of items is consistent.
    ///
    /// # Examples
    /// ```
    /// # use utils::array::ArrayVec;
    /// # use utils::parser::{self, Parser};
    /// let parser = parser::u32()
    ///     .repeat_arrayvec::<5, _>(",", 3);
    /// assert_eq!(
    ///     parser.parse_first("12,34,56,78").unwrap(),
    ///     (ArrayVec::from_slice(&[12, 34, 56, 78]).unwrap(), &b""[..])
    /// );
    /// assert_eq!(
    ///     parser.parse_first("12,34,56,abc").unwrap(),
    ///     (ArrayVec::from_slice(&[12, 34, 56]).unwrap(), &b",abc"[..])
    /// );
    /// assert!(parser.parse_first("12,34").is_err());
    /// ```
    #[inline]
    fn repeat_arrayvec<const N: usize, S: Parser<'i>>(
        self,
        separator: S,
        min_elements: usize,
    ) -> RepeatArrayVec<N, Self, S>
    where
        Self::Output: Copy + Default,
    {
        RepeatArrayVec {
            parser: self,
            separator,
            min_elements,
        }
    }

    /// Repeat this parser while it matches, folding every element into an accumulator.
    ///
    /// This is modeled after [`Iterator::fold`]. See also [`repeat`](Self::repeat).
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// let parser = parser::u32().repeat_fold(",", 3, 0, |acc, x| acc + x);
    /// assert_eq!(
    ///     parser.parse_first("12,34,56,78").unwrap(),
    ///     (12 + 34 + 56 + 78, &b""[..])
    /// );
    /// assert_eq!(
    ///     parser.parse_first("12,34,56,abc").unwrap(),
    ///     (12 + 34 + 56, &b",abc"[..])
    /// );
    /// assert!(parser.parse_first("12,34").is_err());
    /// ```
    #[inline]
    fn repeat_fold<S: Parser<'i>, A: Clone, F: Fn(A, Self::Output) -> A>(
        self,
        separator: S,
        min_elements: usize,
        init: A,
        f: F,
    ) -> RepeatFold<Self, S, A, F> {
        RepeatFold {
            parser: self,
            separator,
            min_elements,
            init,
            f,
        }
    }

    /// Repeat this parser while it matches, returning a [`Vec`].
    ///
    /// Returns error if the item parser commits and fails, or if the separator commits and then
    /// either it or the next item fails.
    ///
    /// To avoid allocating, prefer [`repeat_n`](Self::repeat_n) if the number of items is
    /// consistent and known in advance, or [`repeat_arrayvec`](Self::repeat_arrayvec) if the number
    /// of items is variable but has a known upper bound.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// let parser = parser::u32()
    ///     .repeat(",", 3);
    /// assert_eq!(parser.parse_first("12,34,56,78").unwrap(), (vec![12, 34, 56, 78], &b""[..]));
    /// assert_eq!(parser.parse_first("12,34,56,abc").unwrap(), (vec![12, 34, 56], &b",abc"[..]));
    /// assert!(parser.parse_first("12,34").is_err());
    /// ```
    #[inline]
    fn repeat<S: Parser<'i>>(self, separator: S, min_elements: usize) -> RepeatVec<Self, S> {
        RepeatVec {
            parser: self,
            separator,
            min_elements,
        }
    }

    /// Return the output of this parser as well as the bytes consumed.
    ///
    /// This can be used to map any errors that occur while processing the parsed input back to the
    /// problematic item's position in the input.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// assert_eq!(
    ///     parser::u32().with_consumed().parse_first("012,345,678").unwrap(),
    ///     ((12, &b"012"[..]), &b",345,678"[..])
    /// );
    /// ```
    #[inline]
    fn with_consumed(self) -> WithConsumed<Self> {
        WithConsumed { parser: self }
    }

    /// Parse a prefix (normally a string literal) before this parser.
    ///
    /// The result of the prefix parser is discarded.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// assert_eq!(
    ///     parser::u32()
    ///         .with_prefix("abc")
    ///         .parse_complete("abc123")
    ///         .unwrap(),
    ///     123,
    /// );
    /// ```
    #[inline]
    fn with_prefix<T: Parser<'i>>(self, prefix: T) -> WithPrefix<Self, T> {
        WithPrefix {
            parser: self,
            prefix,
        }
    }

    /// Parse a suffix (normally a string literal) after this parser.
    ///
    /// The result of the suffix parser is discarded.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// assert_eq!(
    ///     parser::u32()
    ///         .with_suffix("abc")
    ///         .parse_complete("123abc")
    ///         .unwrap(),
    ///     123,
    /// );
    /// ```
    #[inline]
    fn with_suffix<T: Parser<'i>>(self, suffix: T) -> WithSuffix<Self, T> {
        WithSuffix {
            parser: self,
            suffix,
        }
    }

    /// Parse a end of line (or end of string) after this parser.
    ///
    /// Equivalent to [`parser.with_suffix`](Parser::with_suffix)`(`[`parser::eol()`](super::eol)`)`.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// assert_eq!(
    ///     parser::u32()
    ///         .with_eol()
    ///         .parse_first("123\nabc")
    ///         .unwrap(),
    ///     (123, &b"abc"[..]),
    /// );
    /// ```
    #[inline]
    fn with_eol(self) -> WithSuffix<Self, Eol> {
        WithSuffix {
            parser: self,
            suffix: Eol(),
        }
    }

    /// Replace this parser's error message with the provided string.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, ParseError, Parser};
    /// let parser = parser::u8()
    ///     .error_msg("expected power level");
    /// assert_eq!(
    ///     parser.parse_complete("123").unwrap(),
    ///     123,
    /// );
    /// assert_eq!(
    ///     parser.parse_complete("abc").unwrap_err().into_source(),
    ///     ParseError::Custom("expected power level"),
    /// );
    /// ```
    #[inline]
    fn error_msg(self, message: &'static str) -> WithErrorMsg<Self> {
        WithErrorMsg {
            parser: self,
            message,
        }
    }

    /// Apply this parser once, returning the parsed value and the remaining input or error.
    ///
    /// This method should only be used to parse the first match and should not be called repeatedly
    /// to parse the remainder of the input, as the reported error positions will be incorrect.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// assert_eq!(parser::u32().parse_first("1234").unwrap(), (1234, &b""[..]));
    /// assert_eq!(parser::u32().parse_first("123abc").unwrap(), (123, &b"abc"[..]));
    /// assert!(parser::u32().parse_first("abc123").is_err());
    /// ```
    #[inline]
    fn parse_first(&self, input: &'i str) -> Result<(Self::Output, &'i [u8]), InputError> {
        let mut state = ParseState::default();
        self.parse_ctx(input.as_bytes(), &mut state, &mut false, false)
            .map_err(|_| state.into_input_error(input))
    }

    /// Apply this parser once, checking the provided input is fully consumed.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// assert_eq!(parser::u32().parse_complete("1234").unwrap(), 1234);
    /// assert!(parser::u32().parse_complete("1234abc").is_err());
    /// ```
    #[inline]
    fn parse_complete(&self, input: &'i str) -> Result<Self::Output, InputError> {
        let mut state = ParseState::default();
        match self.parse_ctx(input.as_bytes(), &mut state, &mut false, true) {
            Ok((v, [])) => return Ok(v),
            Ok((_, remaining)) => {
                // Ensure there is an error reported. This may not be the error returned below, as
                // one may have already been reported further into the input.
                let _ = state.error(ParseError::ExpectedEof(), remaining);
            }
            Err(_) => {}
        }
        Err(state.into_input_error(input))
    }

    /// Apply this parser repeatedly until the provided input is fully consumed.
    ///
    /// Equivalent to `parser.repeat(parser::noop(), 0).parse_complete(input)`.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// assert_eq!(
    ///     parser::u32()
    ///         .then(parser::u32().with_prefix("x"))
    ///         .with_suffix(",".or(parser::eof()))
    ///         .parse_all("1x2,3x4,1234x5678")
    ///         .unwrap(),
    ///     vec![
    ///         (1, 2),
    ///         (3, 4),
    ///         (1234, 5678),
    ///     ]
    /// );
    /// ```
    #[inline]
    fn parse_all(&self, input: &'i str) -> Result<Vec<Self::Output>, InputError> {
        ParserRef(self)
            .repeat(Constant(()), 1)
            .parse_complete(input)
    }

    /// Similar to [`parse_all`](Self::parse_all) but expects a newline between each item.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// assert_eq!(
    ///     parser::u32()
    ///         .then(parser::u32().with_prefix("x"))
    ///         .parse_lines("1x2\n3x4\n1234x5678")
    ///         .unwrap(),
    ///     vec![
    ///         (1, 2),
    ///         (3, 4),
    ///         (1234, 5678),
    ///     ]
    /// );
    /// ```
    #[inline]
    fn parse_lines(&self, input: &'i str) -> Result<Vec<Self::Output>, InputError> {
        ParserRef(self).repeat(Eol(), 1).parse_complete(input)
    }

    /// Create an iterator which applies this parser repeatedly until the provided input is fully
    /// consumed.
    ///
    /// The returned iterator will lazily parse the provided input string, producing a sequence of
    /// [`Result`] values. Once the end of input is reached, or an error is returned, the parser
    /// will always return [`None`].
    ///
    /// # Examples
    /// ```
    /// # use utils::input::InputError;
    /// # use utils::parser::{self, Parser};
    /// let iterator = parser::u32()
    ///     .with_eol()
    ///     .parse_iterator("12\n34\n56\n78");
    /// for item in iterator {
    ///     println!("{}", item?);
    /// }
    /// # Ok::<(), InputError>(())
    /// ```
    ///
    /// ```
    /// # use utils::parser::{self, Parser};
    /// let mut iterator = parser::u32()
    ///     .with_eol()
    ///     .parse_iterator("12\n34\nnot a integer");
    /// assert_eq!(iterator.next().unwrap().unwrap(), 12);
    /// assert_eq!(iterator.next().unwrap().unwrap(), 34);
    /// assert!(iterator.next().unwrap().is_err());
    /// assert!(iterator.next().is_none());
    /// ```
    ///
    /// ```
    /// # use utils::input::InputError;
    /// # use utils::parser::{self, Parser};
    /// let filtered = parser::u32()
    ///     .with_eol()
    ///     .parse_iterator("11\n22\n33\n44\n55")
    ///     .filter(|r| r.is_err() || r.as_ref().is_ok_and(|v| v % 2 == 0))
    ///     .collect::<Result<Vec<u32>, InputError>>()?;
    /// assert_eq!(filtered, vec![22, 44]);
    /// # Ok::<(), InputError>(())
    /// ```
    #[inline]
    fn parse_iterator(self, input: &str) -> ParserIterator<'_, Self> {
        ParserIterator {
            input,
            remaining: input.as_bytes(),
            parser: self,
        }
    }

    /// Create an iterator which returns matches only and skips over errors.
    ///
    /// This is intended for cases that require extracting matches out of the input.
    /// Otherwise, [`parse_iterator`](Self::parse_iterator) should be used with a parser that can
    /// match the entire input structure.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// assert_eq!(
    ///     parser::u32()
    ///         .matches_iterator("abc123d456efg7hi8jk9lmnop")
    ///         .collect::<Vec<_>>(),
    ///     vec![123, 456, 7, 8, 9]
    /// );
    /// ```
    #[inline]
    fn matches_iterator(self, input: &str) -> ParserMatchesIterator<'_, Self> {
        ParserMatchesIterator {
            remaining: input.as_bytes(),
            parser: self,
        }
    }
}

// Workaround to allow using methods which consume a parser in methods which take references.
struct ParserRef<'a, P>(&'a P);
impl<'i, P: Parser<'i>> Parser<'i> for ParserRef<'_, P> {
    type Output = P::Output;
    type Then<T: Parser<'i>> = Unimplemented;

    #[inline]
    fn parse_ctx(
        &self,
        input: &'i [u8],
        state: &mut ParseState<'i>,
        commit: &mut bool,
        tail: bool,
    ) -> ParserResult<'i, Self::Output> {
        self.0.parse_ctx(input, state, commit, tail)
    }
}

struct FromFn<F>(F);
impl<'i, F: Fn(&'i [u8], &mut ParseState<'i>, &mut bool, bool) -> ParserResult<'i, O>, O> Parser<'i>
    for FromFn<F>
{
    type Output = O;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse_ctx(
        &self,
        input: &'i [u8],
        state: &mut ParseState<'i>,
        commit: &mut bool,
        tail: bool,
    ) -> ParserResult<'i, Self::Output> {
        self.0(input, state, commit, tail)
    }
}

/// [`Parser`] which delegates to the provided function/closure.
///
/// This wrapper exists to avoid conflicting implementations of the [`Parser`] trait, which would
/// occur if both [`Leaf`](super::Leaf) and [`Parser`] were implemented for the [`Fn`] trait family.
#[inline]
pub fn from_parser_fn<'i, O>(
    f: impl Fn(&'i [u8], &mut ParseState<'i>, &mut bool, bool) -> ParserResult<'i, O>,
) -> impl Parser<'i, Output = O> {
    FromFn(f)
}

/// Per-parse shared state.
///
/// Tracks the furthest encountered error, allowing combinators like [`Parser::optional`] to return
/// [`Ok`] while still reporting the furthest seen error for better user-facing error messages.
#[must_use]
#[derive(Default)]
pub struct ParseState<'i> {
    pub(super) error: Option<(ParseError, &'i [u8])>,
}

impl<'i> ParseState<'i> {
    /// Record an error.
    ///
    /// The error will be discarded if a further error is already stored.
    ///
    /// Returns an [`ErrToken`] which can be used to return [`Err`] from [`Parser::parse_ctx`].
    #[inline]
    pub fn error(&mut self, error: ParseError, remaining: &'i [u8]) -> ErrToken {
        if self.error.is_none() || matches!(self.error, Some((_, r)) if r.len() > remaining.len()) {
            self.error = Some((error, remaining));
        }
        ErrToken::new()
    }

    /// Build an [`InputError`] from the furthest error seen.
    #[cold]
    pub fn into_input_error(self, input: &str) -> InputError {
        let (error, remaining) = self.error.expect("error not set");
        InputError::new(input, remaining, error)
    }
}

mod token {
    use std::fmt::{Debug, Formatter};

    // Must not be public
    #[derive(PartialEq, Eq)]
    struct Private;

    /// ZST used to ensure that [`Parser`](super::Parser) implementations push errors to
    /// [`ParseState`](super::ParseState).
    ///
    /// Parser implementation must acquire an `ErrToken` to return
    /// [`ParserResult::Err`](super::ParserResult), which can only be done by calling
    /// [`ParseState::error`](super::ParseState::error) or by propagating an `ErrToken` returned by
    /// a child parser.
    #[must_use]
    #[derive(PartialEq, Eq)] // Must not implement Clone or Default
    pub struct ErrToken(Private);

    impl ErrToken {
        pub(super) fn new() -> Self {
            Self(Private)
        }
    }

    impl Debug for ErrToken {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "ErrToken")
        }
    }
}
pub use token::ErrToken;
