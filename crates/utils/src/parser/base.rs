use crate::input::{InputError, MapWithInputExt};
use crate::parser::combinator::{
    Map, MapResult, Optional, Or, RepeatArrayVec, RepeatN, RepeatVec, WithConsumed, WithPrefix,
    WithSuffix,
};
use crate::parser::error::{ParseError, WithErrorMsg};
use crate::parser::iterator::{ParserIterator, ParserMatchesIterator};
use crate::parser::simple::{Constant, Eol};
use crate::parser::then::{Then, Then2, Unimplemented};

/// [`Result`] type returned by [`Parser::parse`].
pub type ParseResult<'i, T> = Result<(T, &'i [u8]), (ParseError, &'i [u8])>;

/// Parser trait.
///
/// Implementations should avoid allocating where possible.
#[must_use]
pub trait Parser<'i>: Sized {
    /// Type of the value produced by [`parse`](Self::parse) when successful.
    type Output;

    /// Type of the chained parser returned by [`then`](Self::then).
    ///
    /// This is used to allow multiple [`then`](Self::then) calls to extend one tuple, instead of
    /// nesting tuples inside each other.
    type Then<T: Parser<'i>>: Then<'i, Self, T>;

    /// Parse the given sequence of bytes.
    ///
    /// Returns a tuple of the successfully parsed [`Output`](Self::Output) value and the
    /// remaining bytes, or a tuple containing a [`ParseError`] and the location of the error.
    ///
    /// The returned slices *must* be subslices of the input slice, otherwise [`InputError::new`]
    /// (in [`parse_all`](Self::parse_all)) will panic.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// assert_eq!(parser::u32().parse(b"1234abc"), Ok((1234, &b"abc"[..])));
    /// assert!(parser::u32().parse(b"abc1234").is_err());
    /// ```
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output>;

    // Provided methods

    /// Sequence another parser after this one.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// assert_eq!(
    ///     parser::i32()
    ///         .then(parser::i32())
    ///         .parse(b"123-123"),
    ///     Ok(((123, -123), &b""[..]))
    /// );
    /// ```
    #[inline]
    fn then<T: Parser<'i>>(self, next: T) -> Self::Then<T> {
        Then::then(self, next)
    }

    /// Attempt to parse using this parser, followed by provided parser.
    ///
    /// If this parser succeeds, the alternative provider won't be tried. If both error, the error
    /// from the parser which parsed further into the input is returned (preferring the first error
    /// if both errored at the same position).
    ///
    /// See also [`parser::one_of`](super::one_of()).
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, ParseError, Parser};
    /// let parser = parser::u8()
    ///     .map(|x| u32::from(x) * 1001001)
    ///     .or(parser::u32());
    /// assert_eq!(
    ///     parser.parse(b"123"),
    ///     Ok((123123123, &b""[..]))
    /// );
    /// assert_eq!(
    ///     parser.parse(b"1000"),
    ///     Ok((1000, &b""[..]))
    /// );
    /// ```
    #[inline]
    fn or<T: Parser<'i, Output = Self::Output>>(self, alternative: T) -> Or<Self, T> {
        Or {
            first: self,
            second: alternative,
        }
    }

    /// Map the output of this parser using the supplied function.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// assert_eq!(
    ///     parser::u32()
    ///         .map(|x| x * 2)
    ///         .parse(b"123"),
    ///     Ok((246, &b""[..]))
    /// );
    /// ```
    ///
    /// Closure that returns a value borrowing from both its input and an outer variable:
    /// ```
    /// # use utils::parser::{self, Parser};
    /// let my_vec = vec![1, 2, 3];
    /// assert_eq!(
    ///     parser::take_while(u8::is_ascii_digit)
    ///         .map(|x| (x, my_vec.as_slice()))
    ///         .parse(b"123"),
    ///     Ok(((&b"123"[..], &[1, 2, 3][..]), &b""[..]))
    /// );
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
    /// assert_eq!(
    ///     parser.parse(b"123"),
    ///     Ok((246, &b""[..]))
    /// );
    /// assert_eq!(
    ///     parser.parse(b"200"),
    ///     Err((ParseError::Custom("input too large"), &b"200"[..]))
    /// );
    /// ```
    ///
    /// Closure that returns a value borrowing from both its input and an outer variable:
    /// ```
    /// # use utils::parser::{self, Parser};
    /// let my_vec = vec![1, 2, 3];
    /// assert_eq!(
    ///     parser::take_while(u8::is_ascii_digit)
    ///         .map_res(|x| {
    ///             if x.len() < 100 {
    ///                 Ok((x, my_vec.as_slice()))
    ///             } else {
    ///                 Err("expected more digits")
    ///             }
    ///         })
    ///         .parse(b"123"),
    ///     Ok(((&b"123"[..], &[1, 2, 3][..]), &b""[..]))
    /// );
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

    /// Wrap [`Output`](Self::Output) in [`Option`], returning [`None`] on error.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, ParseError, Parser};
    /// let parser = parser::u32()
    ///     .optional();
    /// assert_eq!(
    ///     parser.parse(b"123"),
    ///     Ok((Some(123), &b""[..]))
    /// );
    /// assert_eq!(
    ///     parser.parse(b"abc"),
    ///     Ok((None, &b"abc"[..]))
    /// );
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
    /// assert_eq!(
    ///     parser::u32()
    ///         .repeat_n(",") // N = 3 is inferred
    ///         .parse(b"12,34,56"),
    ///     Ok(([12, 34, 56], &b""[..]))
    /// );
    /// ```
    #[inline]
    fn repeat_n<const N: usize, S: Parser<'i>>(self, separator: S) -> RepeatN<N, Self, S>
    where
        Self::Output: Copy + Default,
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
    /// See [`repeat`](Self::repeat) if the upper bound is large or not known, and
    /// [`repeat_n`](Self::repeat_n) if the number of items is consistent.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// let parser = parser::u32()
    ///     .repeat_arrayvec(",", 3);
    /// assert_eq!(parser.parse(b"12,34,56,78"), Ok(([12, 34, 56, 78].into(), &b""[..])));
    /// assert!(parser.parse(b"12,34").is_err());
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

    /// Repeat this parser while it matches, returning a [`Vec`].
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
    /// assert_eq!(parser.parse(b"12,34,56,78"), Ok((vec![12, 34, 56, 78], &b""[..])));
    /// assert!(parser.parse(b"12,34").is_err());
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
    ///     parser::u32().with_consumed().parse(b"012,345,678"),
    ///     Ok(((12, &b"012"[..]), &b",345,678"[..]))
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
    ///         .parse(b"abc123"),
    ///     Ok((123, &b""[..]))
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
    ///         .parse(b"123abc"),
    ///     Ok((123, &b""[..]))
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
    ///     parser::u32().with_eol()
    ///         .parse(b"123\nabc"),
    ///     Ok((123, &b"abc"[..]))
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
    ///     parser.parse(b"123"),
    ///     Ok((123, &b""[..]))
    /// );
    /// assert_eq!(
    ///     parser.parse(b"abc"),
    ///     Err((ParseError::Custom("expected power level"), &b"abc"[..]))
    /// );
    /// ```
    #[inline]
    fn error_msg(self, message: &'static str) -> WithErrorMsg<Self> {
        WithErrorMsg {
            parser: self,
            message,
        }
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
        match self.parse(input.as_bytes()).map_with_input(input)? {
            (v, []) => Ok(v),
            (_, remaining) => Err(InputError::new(input, remaining, ParseError::ExpectedEof())),
        }
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
            .repeat(Constant(()), 0)
            .parse_complete(input)
    }

    /// Similar to [`parse_all`](Self::parse_all) but expects a newline after each item.
    ///
    /// Equivalent to [`parser.with_eol()`](Parser::with_eol)`.parse_all(input)`.
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
        ParserRef(self)
            .with_suffix(Eol())
            .repeat(Constant(()), 0)
            .parse_complete(input)
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
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        self.0.parse(input)
    }
}

/// Matches the string literal exactly.
///
/// Normally used with [`with_prefix`](Parser::with_prefix)/[`with_suffix`](Parser::with_suffix).
impl<'i> Parser<'i> for &'static str {
    type Output = ();
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
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
/// Normally used with [`with_prefix`](Parser::with_prefix)/[`with_suffix`](Parser::with_suffix).
impl<'i> Parser<'i> for u8 {
    type Output = ();
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        if input.first() == Some(self) {
            Ok(((), &input[1..]))
        } else {
            Err((ParseError::ExpectedByte(*self), input))
        }
    }
}

/// Allow custom functions and closures to be used as parsers.
impl<'i, O, F: Fn(&'i [u8]) -> ParseResult<'i, O>> Parser<'i> for F {
    type Output = O;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        self(input)
    }
}

/// Trait for types that have a canonical parser.
pub trait Parseable {
    type Parser: for<'i> Parser<'i, Output = Self>;
    const PARSER: Self::Parser;
}
