use crate::number::{Integer, SignedInteger, UnsignedInteger};
use crate::parser::then::Then2;
use crate::parser::{ParseError, ParseResult, Parseable, Parser};
use std::marker::PhantomData;
use std::ops::RangeInclusive;

#[derive(Copy, Clone)]
pub struct UnsignedParser<U: UnsignedInteger>(PhantomData<U>);
impl<'i, U: UnsignedInteger> Parser<'i> for UnsignedParser<U> {
    type Output = U;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, mut input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        let mut n = match input {
            [d @ b'0'..=b'9', ..] => {
                input = &input[1..];
                U::from(d - b'0')
            }
            _ => return Err((ParseError::Expected("unsigned integer"), input)),
        };

        while let Some(d @ b'0'..=b'9') = input.first() {
            n = n
                .checked_mul(U::from(10))
                .and_then(|n| n.checked_add(U::from(d - b'0')))
                .ok_or((ParseError::too_large(U::MAX), input))?;
            input = &input[1..];
        }

        Ok((n, input))
    }
}

#[derive(Copy, Clone)]
pub struct SignedParser<S: SignedInteger>(PhantomData<S>);
impl<'i, S: SignedInteger> Parser<'i> for SignedParser<S> {
    type Output = S;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[expect(clippy::cast_possible_wrap)]
    #[inline]
    fn parse(&self, mut input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        let (mut n, positive) = match input {
            [d @ b'0'..=b'9', rem @ ..] | [b'+', d @ b'0'..=b'9', rem @ ..] => {
                input = rem;
                (S::from((d - b'0') as i8), true)
            }
            [b'-', d @ b'0'..=b'9', rem @ ..] => {
                input = rem;
                (S::from(-((d - b'0') as i8)), false)
            }
            _ => return Err((ParseError::Expected("signed integer"), input)),
        };

        if positive {
            while let Some(d @ b'0'..=b'9') = input.first() {
                n = n
                    .checked_mul(S::from(10))
                    .and_then(|n| n.checked_add(S::from((d - b'0') as i8)))
                    .ok_or((ParseError::too_large(S::MAX), input))?;
                input = &input[1..];
            }
        } else {
            while let Some(d @ b'0'..=b'9') = input.first() {
                n = n
                    .checked_mul(S::from(10))
                    .and_then(|n| n.checked_sub(S::from((d - b'0') as i8)))
                    .ok_or((ParseError::too_small(S::MIN), input))?;
                input = &input[1..];
            }
        }

        Ok((n, input))
    }
}

macro_rules! parser_for {
    ($p:ident => $($n:ident),+) => {$(
        impl Parseable for std::primitive::$n {
            type Parser = $p<std::primitive::$n>;
            const PARSER: Self::Parser = $p(PhantomData);
        }

        #[doc = concat!("Parser for [`prim@", stringify!($n), "`] values.")]
        #[inline]
        #[must_use]
        pub fn $n() -> $p<std::primitive::$n> {
            $p(PhantomData)
        }
    )+};
}
parser_for! { UnsignedParser => u8, u16, u32, u64, u128 }
parser_for! { SignedParser => i8, i16, i32, i64, i128 }

/// Parsing as [`usize`] should be discouraged as it leads to parsers which behave differently at
/// runtime on 32-bit and 64-bit platforms, so no `parser::usize()` function is provided.
///
/// However, [`Parseable`] is implemented for [`usize`] as it is safe to use [`number_range()`]
/// with a constant hard-coded max, which will fail at compile time if the constant is too large
/// for the platform's usize.
impl Parseable for std::primitive::usize {
    type Parser = UnsignedParser<std::primitive::usize>;
    const PARSER: Self::Parser = UnsignedParser(PhantomData);
}

#[derive(Copy, Clone)]
pub struct NumberRange<I> {
    min: I,
    max: I,
}

impl<'i, I: Integer + Parseable> Parser<'i> for NumberRange<I> {
    type Output = I;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        let (v, remaining) = I::PARSER.parse(input)?;
        if v < self.min {
            Err((ParseError::too_small(self.min), input))
        } else if v > self.max {
            Err((ParseError::too_large(self.max), input))
        } else {
            Ok((v, remaining))
        }
    }
}

/// Parser for numbers in the supplied range.
///
/// The type of the number to parse is inferred from the range's type.
///
/// See also [`byte_range`](super::byte_range).
///
/// # Examples
/// ```
/// # use utils::parser::{self, Parser};
/// assert_eq!(
///     parser::number_range(100u8..=125u8).parse(b"123, 120"),
///     Ok((123u8, &b", 120"[..]))
/// );
/// ```
#[inline]
#[must_use]
pub fn number_range<I: Integer + Parseable>(range: RangeInclusive<I>) -> NumberRange<I> {
    let min = *range.start();
    let max = *range.end();
    assert!(min <= max);
    NumberRange { min, max }
}

#[derive(Copy, Clone)]
pub struct Digit {}

impl<'i> Parser<'i> for Digit {
    type Output = u8;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        if let Some(d @ b'0'..=b'9') = input.first() {
            Ok((d - b'0', &input[1..]))
        } else {
            Err((ParseError::Expected("digit"), input))
        }
    }
}

/// Parser for single digits.
///
/// # Examples
/// ```
/// # use utils::parser::{self, Parser};
/// assert_eq!(
///     parser::digit().parse(b"12345"),
///     Ok((1u8, &b"2345"[..]))
/// );
/// ```
#[inline]
#[must_use]
pub fn digit() -> Digit {
    Digit {}
}
