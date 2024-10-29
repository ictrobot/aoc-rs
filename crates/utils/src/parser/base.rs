use crate::input::{InputError, MapWithInputExt};
use crate::parser::combinator::{
    Map, MapResult, Optional, Or, RepeatN, RepeatVec, WithPrefix, WithSuffix,
};
use crate::parser::error::WithErrorMsg;
use crate::parser::simple::Eol;
use crate::parser::then::Then2;
use crate::parser::ParseError;

/// [`Result`] type returned by [`Parser::parse`].
pub type ParseResult<'i, T> = Result<(T, &'i [u8]), (ParseError, &'i [u8])>;

/// Parser trait.
///
/// Implementations should avoid allocating where possible.
pub trait Parser: Sized {
    /// Type of the value produced by [`parse`](Self::parse) when successful.
    ///
    /// Generic over the input `'i` lifetime.
    type Output<'i>;

    /// Type of the chained parser returned by [`then`](Self::then).
    ///
    /// This is used to allow multiple [`then`](Self::then) calls to extend one tuple, instead of
    /// nesting tuples inside each other.
    type Then<T: Parser>: Parser;

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
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>>;

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
    fn then<T: Parser>(self, next: T) -> Self::Then<T>;

    // Provided methods

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
    fn or<T: for<'i> Parser<Output<'i> = Self::Output<'i>>>(self, alternative: T) -> Or<Self, T> {
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
    fn map<O, F: for<'i> Fn(Self::Output<'i>) -> O>(self, f: F) -> Map<Self, F> {
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
    fn map_res<O, F: for<'i> Fn(Self::Output<'i>) -> Result<O, &'static str>>(
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

    /// Repeat this parser `N` times, returning an [`array`].
    ///
    /// See also [`repeat`](Self::repeat) which returns a [`Vec`] instead, for unknown or varying
    /// number of repeats.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// assert_eq!(
    ///     parser::u32()
    ///         .with_suffix(",".optional())
    ///         .repeat_n() // N = 3 is inferred
    ///         .parse(b"12,34,56"),
    ///     Ok(([12, 34, 56], &b""[..]))
    /// );
    /// ```
    fn repeat_n<const N: usize>(self) -> RepeatN<N, Self>
    where
        for<'i> Self::Output<'i>: Copy + Default,
    {
        RepeatN { parser: self }
    }

    /// Repeat this parser while it matches, returning a [`Vec`].
    ///
    /// If the number of items is constant and known in advance, prefer [`repeat_n`](Self::repeat_n)
    /// as it avoids allocating.
    ///
    /// See also [`repeat_min`](Self::repeat_min), which ensures at least N items are parsed.
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// assert_eq!(
    ///     parser::u32()
    ///         .with_suffix(",".optional())
    ///         .repeat()
    ///         .parse(b"12,34,56,78"),
    ///     Ok((vec![12, 34, 56, 78], &b""[..]))
    /// );
    /// ```
    fn repeat(self) -> RepeatVec<Self> {
        RepeatVec {
            parser: self,
            min_elements: 0,
        }
    }

    /// Repeat this parser at least N times, returning a [`Vec`].
    ///
    /// See also [`repeat`](Self::repeat).
    ///
    /// # Examples
    /// ```
    /// # use utils::parser::{self, Parser};
    /// let parser = parser::u32()
    ///     .with_suffix(",".optional())
    ///     .repeat_min(3);
    /// assert_eq!(parser.parse(b"12,34,56,78"), Ok((vec![12, 34, 56, 78], &b""[..])));
    /// assert!(parser.parse(b"12,34").is_err());
    /// ```
    fn repeat_min(self, min_elements: usize) -> RepeatVec<Self> {
        RepeatVec {
            parser: self,
            min_elements,
        }
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
    fn with_prefix<T: Parser>(self, prefix: T) -> WithPrefix<Self, T> {
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
    fn with_suffix<T: Parser>(self, suffix: T) -> WithSuffix<Self, T> {
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
    fn parse_all<'i>(&self, input: &'i str) -> Result<Vec<Self::Output<'i>>, InputError> {
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
    fn parse_lines<'i>(&self, input: &'i str) -> Result<Vec<Self::Output<'i>>, InputError> {
        // Can't use WithSuffix as it consumes the input parser
        struct LineParser<'a, P>(&'a P);
        impl<'a, P: Parser> Parser for LineParser<'a, P> {
            type Output<'i> = P::Output<'i>;
            type Then<T: Parser> = Then2<Self, T>;

            #[inline]
            fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
                match self.0.parse(input) {
                    Ok((v, remaining)) => match Eol().parse(remaining) {
                        Ok(((), remaining)) => Ok((v, remaining)),
                        Err(e) => Err(e),
                    },
                    Err(e) => Err(e),
                }
            }

            fn then<T: Parser>(self, _: T) -> Self::Then<T> {
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
    fn parse_complete<'i>(&self, input: &'i str) -> Result<Self::Output<'i>, InputError> {
        match self.parse(input.as_bytes()).map_with_input(input)? {
            (v, []) => Ok(v),
            (_, remaining) => Err(InputError::new(input, remaining, "expected end of input")),
        }
    }
}

/// Matches the string literal exactly.
///
/// Normally used with [`with_prefix`](Parser::with_prefix)/[`with_suffix`](Parser::with_suffix).
impl Parser for &'static str {
    type Output<'i> = Self;
    type Then<T: Parser> = Then2<Self, T>;

    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
        if let Some(remainder) = input.strip_prefix(self.as_bytes()) {
            Ok((self, remainder))
        } else {
            Err((ParseError::ExpectedLiteral(self), input))
        }
    }

    fn then<T: Parser>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

/// Matches the byte exactly.
///
/// Normally used with [`with_prefix`](Parser::with_prefix)/[`with_suffix`](Parser::with_suffix).
impl Parser for u8 {
    type Output<'i> = Self;
    type Then<T: Parser> = Then2<Self, T>;

    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
        if input.first() == Some(self) {
            Ok((*self, &input[1..]))
        } else {
            Err((ParseError::ExpectedByte(*self), input))
        }
    }

    fn then<T: Parser>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

/// Allow custom functions and closures to be used as parsers.
impl<O, F: Fn(&[u8]) -> ParseResult<O>> Parser for F {
    type Output<'i> = O;
    type Then<T: Parser> = Then2<Self, T>;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
        self(input)
    }

    fn then<T: Parser>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

/// Trait for types that have a canonical parser.
pub trait Parseable {
    type Parser: for<'i> Parser<Output<'i> = Self>;
    const PARSER: Self::Parser;
}
