use crate::parser::then::Then2;
use crate::parser::{ParseError, ParseResult, Parser};

#[derive(Copy, Clone)]
pub struct Constant<V: Copy>(V);
impl<'i, V: Copy> Parser<'i> for Constant<V> {
    type Output = V;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        Ok((self.0, input))
    }

    fn then<T: Parser<'i>>(self, next: T) -> Self::Then<T> {
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
pub fn constant<T: Copy>(v: T) -> Constant<T> {
    Constant(v)
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

    fn then<T: Parser<'i>>(self, next: T) -> Self::Then<T> {
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

    fn then<T: Parser<'i>>(self, next: T) -> Self::Then<T> {
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
