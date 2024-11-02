use crate::parser::then::{Then2, Unimplemented};
use crate::parser::{ParseError, ParseResult, Parser};
use std::ops::RangeInclusive;

#[derive(Copy, Clone)]
pub struct Byte();
impl Parser for Byte {
    type Output<'i> = u8;
    type Then<T: Parser> = Then2<Self, T>;

    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
        if let [byte, remaining @ ..] = input {
            Ok((*byte, remaining))
        } else {
            Err((ParseError::Expected("byte"), input))
        }
    }

    fn then<T: Parser>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

/// Parser that consumes a single byte.
///
/// Not to be confused with [`u8`](super::u8), which parses a number in the range 0-255.
///
/// # Examples
/// ```
/// # use utils::parser::{self, Parser};
/// assert_eq!(
///     parser::byte().parse(b"abcdef"),
///     Ok((b'a', &b"bcdef"[..]))
/// );
/// assert_eq!(
///     parser::byte().parse(b"123"),
///     Ok((b'1', &b"23"[..]))
/// );
/// ```
#[must_use]
pub fn byte() -> Byte {
    Byte()
}

#[derive(Copy, Clone)]
pub struct ByteRange {
    min: u8,
    max: u8,
}
impl Parser for ByteRange {
    type Output<'i> = u8;
    type Then<T: Parser> = Then2<Self, T>;

    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
        if let [byte, remaining @ ..] = input {
            if *byte >= self.min && *byte <= self.max {
                Ok((*byte, remaining))
            } else {
                Err((ParseError::ExpectedByteRange(self.min, self.max), input))
            }
        } else {
            Err((ParseError::Expected("byte"), input))
        }
    }

    fn then<T: Parser>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

/// Parser that consumes a single byte in the supplied range.
///
/// See also [`number_range`](super::number_range) and [`byte`].
///
/// # Examples
/// ```
/// # use utils::parser::{self, Parser};
/// assert_eq!(
///     parser::byte_range(b'a'..=b'z').parse(b"hello world"),
///     Ok((b'h', &b"ello world"[..]))
/// );
/// ```
#[must_use]
pub fn byte_range(range: RangeInclusive<u8>) -> ByteRange {
    let min = *range.start();
    let max = *range.end();
    assert!(min <= max);
    ByteRange { min, max }
}

#[derive(Copy, Clone)]
pub struct Constant<V: Copy>(pub(super) V);
impl<V: Copy> Parser for Constant<V> {
    type Output<'i> = V;
    type Then<T: Parser> = Then2<Self, T>;

    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
        Ok((self.0, input))
    }

    fn then<T: Parser>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

/// Parser that consumes no input and always succeeds, returning the provided value.
///
/// # Examples
/// ```
/// # use utils::parser::{self, Parser};
/// assert_eq!(
///     parser::constant(1).parse(b"abc"),
///     Ok((1, &b"abc"[..]))
/// );
/// ```
#[must_use]
pub fn constant<T: Copy>(v: T) -> Constant<T> {
    Constant(v)
}

/// Parser that consumes no input and always succeeds, returning [`()`](unit).
///
/// # Examples
/// ```
/// # use utils::parser::{self, Parser};
/// assert_eq!(
///     parser::noop().parse(b"abc"),
///     Ok(((), &b"abc"[..]))
/// );
/// ```
#[must_use]
pub fn noop() -> Constant<()> {
    const {
        assert!(size_of::<Constant<()>>() == 0);
    }
    Constant(())
}

#[derive(Copy, Clone)]
pub struct Eof();
impl Parser for Eof {
    type Output<'i> = ();
    type Then<T: Parser> = Unimplemented;

    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
        match input {
            [] => Ok(((), input)),
            _ => Err((ParseError::Expected("end of input"), input)),
        }
    }

    fn then<T: Parser>(self, _next: T) -> Self::Then<T> {
        panic!("chaining after eof will never match");
    }
}

/// Parser which matches the end of the input.
///
/// Useful when parsing a list and each item is separated by a separator, unless it is at the end of
/// the input.
///
/// # Examples
/// ```
/// # use utils::parser::{self, Parser};
/// assert_eq!(
///     parser::eof().parse(b""),
///     Ok(((), &b""[..]))
/// );
/// assert_eq!(
///     parser::u32()
///         .with_suffix(b','.or(parser::eof()))
///         .repeat_n()
///         .parse(b"12,34,56"),
///     Ok(([12, 34, 56], &b""[..]))
/// );
/// ```
#[must_use]
pub fn eof() -> Eof {
    Eof()
}

#[derive(Copy, Clone)]
pub struct Eol();
impl Parser for Eol {
    type Output<'i> = ();
    type Then<T: Parser> = Then2<Self, T>;

    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
        match input {
            [b'\n', remaining @ ..] | [b'\r', b'\n', remaining @ ..] => Ok(((), remaining)),
            [] => Ok(((), input)),
            _ => Err((ParseError::Expected("newline or end of input"), input)),
        }
    }

    fn then<T: Parser>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

/// Parser which matches newlines or the end of the input.
///
/// Matches both LF and CRLF line endings.
///
/// # Examples
/// ```
/// # use utils::parser::{self, Parser};
/// assert_eq!(
///     parser::eol().parse(b"\nabc"),
///     Ok(((), &b"abc"[..]))
/// );
/// assert_eq!(
///     parser::eol().parse(b"\r\nabc"),
///     Ok(((), &b"abc"[..]))
/// );
/// assert_eq!(
///     parser::eol().parse(b""),
///     Ok(((), &b""[..]))
/// );
/// ```
#[must_use]
pub fn eol() -> Eol {
    Eol()
}

#[derive(Copy, Clone)]
pub struct TakeWhile<const N: usize>(fn(&u8) -> bool);
impl<const N: usize> Parser for TakeWhile<N> {
    type Output<'i> = &'i [u8];
    type Then<T: Parser> = Then2<Self, T>;

    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
        let mut end = 0;
        while end < input.len() && self.0(&input[end]) {
            end += 1;
        }
        if end >= N {
            Ok(input.split_at(end))
        } else {
            Err((ParseError::ExpectedMatches(N), &input[end..]))
        }
    }

    fn then<T: Parser>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

/// Parser for substrings consisting of bytes matching the provided function.
///
/// # Examples
/// ```
/// # use utils::parser::{self, Parser};
/// let parser = parser::take_while(u8::is_ascii_lowercase);
/// assert_eq!(
///     parser.parse(b"abc def"),
///     Ok((&b"abc"[..], &b" def"[..]))
/// );
/// assert_eq!(
///     parser.parse(b"ABC"),
///     Ok((&b""[..], &b"ABC"[..]))
/// );
/// ```
#[must_use]
pub fn take_while(f: fn(&u8) -> bool) -> TakeWhile<0> {
    TakeWhile(f)
}

/// Parser for non-empty substrings consisting of bytes matching the provided function.
///
/// # Examples
/// ```
/// # use utils::parser::{self, Parser};
/// let parser = parser::take_while1(u8::is_ascii_lowercase);
/// assert_eq!(
///     parser.parse(b"abc def"),
///     Ok((&b"abc"[..], &b" def"[..]))
/// );
/// assert!(parser.parse(b"ABC").is_err());
/// ```
#[must_use]
pub fn take_while1(f: fn(&u8) -> bool) -> TakeWhile<1> {
    TakeWhile(f)
}
