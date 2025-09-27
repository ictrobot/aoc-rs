use crate::parser::then::{Then2, Unimplemented};
use crate::parser::{ParseError, ParseResult, Parser};
use std::ops::RangeInclusive;

#[derive(Copy, Clone)]
pub struct Byte();
impl<'i> Parser<'i> for Byte {
    type Output = u8;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        if let [byte, remaining @ ..] = input {
            Ok((*byte, remaining))
        } else {
            Err((ParseError::Expected("byte"), input))
        }
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
pub struct ByteLut<'a, O> {
    lut: &'a [Option<O>; 256],
    error: ParseError,
}
impl<'i, O: Copy> Parser<'i> for ByteLut<'_, O> {
    type Output = O;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        if let [byte, remaining @ ..] = input
            && let Some(output) = self.lut[*byte as usize]
        {
            Ok((output, remaining))
        } else {
            Err((self.error, input))
        }
    }
}

/// Parser that consumes a single byte and maps it using a lookup table.
///
/// Equivalent to `parser::byte().map_res(|b| LOOKUP[b as usize].ok_or("expected ..."))`, which is
/// usually faster than an equivalent match statement in the closure.
///
/// See also [`parser::byte_map!`](crate::parser::byte_map!) which wraps this function, allowing a
/// match-like syntax to be used to define the lookup table.
///
/// # Examples
/// ```
/// # use utils::parser::{self, Parser, ParseError};
/// const LOOKUP: [Option<bool>; 256] = {
///     let mut x = [None; 256];
///     x['#' as usize] = Some(true);
///     x['.' as usize] = Some(false);
///     x
/// };
///
/// let parser = parser::byte_lut(&LOOKUP, ParseError::Custom("expected '#' or '.'"));
/// assert_eq!(parser.parse(b"#..##"), Ok((true, &b"..##"[..])));
/// assert_eq!(parser.parse(b"..##"), Ok((false, &b".##"[..])));
/// assert_eq!(parser.parse(b"abc"), Err((ParseError::Custom("expected '#' or '.'"), &b"abc"[..])));
/// ```
#[inline]
#[must_use]
pub fn byte_lut<T: Copy>(lut: &'_ [Option<T>; 256], error: ParseError) -> ByteLut<'_, T> {
    ByteLut { lut, error }
}

#[derive(Copy, Clone)]
pub struct ByteRange {
    min: u8,
    max: u8,
}
impl<'i> Parser<'i> for ByteRange {
    type Output = u8;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
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
#[inline]
#[must_use]
pub fn byte_range(range: RangeInclusive<u8>) -> ByteRange {
    let min = *range.start();
    let max = *range.end();
    assert!(min <= max);
    ByteRange { min, max }
}

#[derive(Copy, Clone)]
pub struct Constant<V: Copy>(pub(super) V);
impl<'i, V: Copy> Parser<'i> for Constant<V> {
    type Output = V;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        Ok((self.0, input))
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
impl<'i> Parser<'i> for Eof {
    type Output = ();
    type Then<T: Parser<'i>> = Unimplemented;

    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        match input {
            [] => Ok(((), input)),
            _ => Err((ParseError::ExpectedEof(), input)),
        }
    }

    fn then<T: Parser<'i>>(self, _next: T) -> Self::Then<T> {
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
///         .parse_all("12,34,56")
///         .unwrap(),
///     vec![12, 34, 56],
/// );
/// ```
#[must_use]
pub fn eof() -> Eof {
    Eof()
}

#[derive(Copy, Clone)]
pub struct Eol();
impl<'i> Parser<'i> for Eol {
    type Output = ();
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        match input {
            [b'\n', remaining @ ..] | [b'\r', b'\n', remaining @ ..] => Ok(((), remaining)),
            [] => Ok(((), input)),
            _ => Err((ParseError::Expected("newline or end of input"), input)),
        }
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
impl<'i, const N: usize> Parser<'i> for TakeWhile<N> {
    type Output = &'i [u8];
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
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
