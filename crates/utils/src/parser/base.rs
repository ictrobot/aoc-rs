use crate::input::{InputError, MapWithInputExt};
use crate::parser::combinator::{Map, MapResult, Optional, Or, Repeat, WithPrefix, WithSuffix};
use crate::parser::error::WithErrorMsg;
use crate::parser::simple::Eol;
use crate::parser::then::Then2;
use crate::parser::ParseError;

/// [`Result`] type returned by [`Parser::parse`].
pub type ParseResult<'i, T> = Result<(T, &'i [u8]), (ParseError, &'i [u8])>;

/// Parser trait, generic over the input `'i` lifetime.
///
/// Implementations should avoid allocating where possible.
pub trait Parser<'i>: Sized {
    /// Type of the value produced by [`parse`](Self::parse) when successful.
    type Output;

    /// Type of the chained parser returned by [`then`](Self::then).
    ///
    /// This is used to allow multiple [`then`](Self::then) calls to extend one tuple, instead of
    /// nesting tuples inside each other.
    type Then<T: Parser<'i>>: Parser<'i>;

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
    fn then<T: Parser<'i>>(self, next: T) -> Self::Then<T>;

    // Provided methods

    /// Attempt to parse using this parser, followed by provided parser.
    ///
    /// If this parser succeeds, the alternative provider won't be tried. If both error, the error
    /// from the parser which parsed further into the input is returned (preferring the first error
    /// if both errored at the same position).
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
    fn optional(self) -> Optional<Self> {
        Optional { parser: self }
    }

    /// Repeat this parser `N` times, returning an array.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// assert_eq!(
    ///     parser::u32()
    ///         .with_suffix(",".optional())
    ///         .repeat() // N = 3 is inferred
    ///         .parse(b"12,34,56"),
    ///     Ok(([12, 34, 56], &b""[..]))
    /// );
    /// ```
    fn repeat<const N: usize>(self) -> Repeat<N, Self>
    where
        Self::Output: Copy + Default,
    {
        Repeat { parser: self }
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
    fn with_suffix<T: Parser<'i>>(self, suffix: T) -> WithSuffix<Self, T> {
        WithSuffix {
            parser: self,
            suffix,
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
    fn error_msg(self, message: &'static str) -> WithErrorMsg<Self> {
        WithErrorMsg {
            parser: self,
            message,
        }
    }

    /// Apply this parser repeatedly until the provided input is fully consumed.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// assert_eq!(
    ///     parser::u32()
    ///         .then(parser::u32().with_prefix("x"))
    ///         .with_suffix(",".optional())
    ///         .parse_all("1x2,3x4,1234x5678")
    ///         .unwrap(),
    ///     vec![
    ///         (1, 2),
    ///         (3, 4),
    ///         (1234, 5678),
    ///     ]
    /// );
    /// ```
    fn parse_all(&self, input: &'i str) -> Result<Vec<Self::Output>, InputError> {
        let mut results = Vec::new();
        let mut remaining = input.as_bytes();
        while !remaining.is_empty() {
            let (v, new_remaining) = self.parse(remaining).map_with_input(input)?;

            if results.is_empty() {
                let length = remaining.len() - new_remaining.len();
                results.reserve(2 + ((remaining.len() / length) * 6 / 5));
            }

            remaining = new_remaining;
            results.push(v);
        }
        Ok(results)
    }

    /// Similar to [`parse_all`](Self::parse_all) but expects a newline after each item.
    ///
    /// Equivalent to `parser.with_suffix(`[`parser::eol()`](super::eol)`).parse_all(input)`.
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
    fn parse_lines(&self, input: &'i str) -> Result<Vec<Self::Output>, InputError> {
        // Can't use WithSuffix as it consumes the input parser
        struct LineParser<'a, P>(&'a P);
        impl<'a, 'i, P: Parser<'i>> Parser<'i> for LineParser<'a, P> {
            type Output = P::Output;
            type Then<T: Parser<'i>> = Then2<Self, T>;

            #[inline]
            fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
                match self.0.parse(input) {
                    Ok((v, remaining)) => match Eol().parse(remaining) {
                        Ok(((), remaining)) => Ok((v, remaining)),
                        Err(e) => Err(e),
                    },
                    Err(e) => Err(e),
                }
            }

            fn then<T: Parser<'i>>(self, _: T) -> Self::Then<T> {
                unreachable!();
            }
        }

        LineParser(self).parse_all(input)
    }

    /// Apply this parser once, checking the provided input is fully consumed.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// assert_eq!(parser::u32().parse_complete("1234").unwrap(), 1234);
    /// assert!(parser::u32().parse_complete("1234abc").is_err());
    /// ```
    fn parse_complete(&self, input: &'i str) -> Result<Self::Output, InputError> {
        match self.parse(input.as_bytes()).map_with_input(input)? {
            (v, []) => Ok(v),
            (_, remaining) => Err(InputError::new(input, remaining, "expected end of input")),
        }
    }
}

/// Matches the string literal exactly.
///
/// Normally used with [`with_prefix`](Parser::with_prefix)/[`with_suffix`](Parser::with_suffix).
impl<'i> Parser<'i> for &'static str {
    type Output = Self;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        if let Some(remainder) = input.strip_prefix(self.as_bytes()) {
            Ok((self, remainder))
        } else {
            Err((ParseError::ExpectedLiteral(self), input))
        }
    }

    fn then<T: Parser<'i>>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

/// Matches the byte exactly.
///
/// Normally used with [`with_prefix`](Parser::with_prefix)/[`with_suffix`](Parser::with_suffix).
impl<'i> Parser<'i> for u8 {
    type Output = Self;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        if input.first() == Some(self) {
            Ok((*self, &input[1..]))
        } else {
            Err((ParseError::ExpectedByte(*self), input))
        }
    }

    fn then<T: Parser<'i>>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}
