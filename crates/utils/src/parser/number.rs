use crate::number::{Integer, SignedInteger, UnsignedInteger};
use crate::parser::{Leaf, LeafResult, ParseError, Parseable};
use std::marker::PhantomData;
use std::ops::RangeInclusive;

#[derive(Copy, Clone)]
pub struct UnsignedParser<U: UnsignedInteger>(PhantomData<U>);
impl<'i, U: UnsignedInteger> Leaf<'i> for UnsignedParser<U> {
    type Output = U;

    #[inline(always)]
    fn parse(&self, mut input: &'i [u8]) -> LeafResult<'i, Self::Output> {
        let mut n = match input {
            [d @ b'0'..=b'9', ..] => {
                input = &input[1..];
                U::from(d - b'0')
            }
            _ => return Err((ParseError::Expected("unsigned integer"), input)),
        };

        while let Some(&b) = input.first() {
            let d = U::from(b).wrapping_sub(U::from(b'0'));
            if d > U::from(9) {
                break;
            }
            // Avoid checked operations until near the maximum value
            if n >= U::MAX / U::from(10) {
                return unsigned_near_max(n, input);
            }
            n = n * U::from(10) + d;
            input = &input[1..];
        }

        Ok((n, input))
    }
}

#[cold]
#[inline(never)]
fn unsigned_near_max<U: UnsignedInteger>(mut n: U, mut input: &[u8]) -> LeafResult<'_, U> {
    while let Some(d @ b'0'..=b'9') = input.first() {
        n = n
            .checked_mul(U::from(10))
            .and_then(|n| n.checked_add(U::from(d - b'0')))
            .ok_or((ParseError::too_large(U::MAX), input))?;
        input = &input[1..];
    }
    Ok((n, input))
}

#[derive(Copy, Clone)]
pub struct SignedParser<S: SignedInteger>(PhantomData<S>);
impl<'i, S: SignedInteger> Leaf<'i> for SignedParser<S> {
    type Output = S;

    #[expect(clippy::cast_possible_wrap)]
    #[inline(always)]
    fn parse(&self, mut input: &'i [u8]) -> LeafResult<'i, Self::Output> {
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
            while let Some(&b) = input.first() {
                let d = S::Unsigned::from(b).wrapping_sub(S::Unsigned::from(b'0'));
                if d > S::Unsigned::from(9) {
                    break;
                }
                n = n
                    .checked_mul(S::from(10))
                    .and_then(|n| n.checked_add(d.cast_signed()))
                    .ok_or((ParseError::too_large(S::MAX), input))?;
                input = &input[1..];
            }
        } else {
            while let Some(&b) = input.first() {
                let d = S::Unsigned::from(b).wrapping_sub(S::Unsigned::from(b'0'));
                if d > S::Unsigned::from(9) {
                    break;
                }
                n = n
                    .checked_mul(S::from(10))
                    .and_then(|n| n.checked_sub(d.cast_signed()))
                    .ok_or((ParseError::too_small(S::MIN), input))?;
                input = &input[1..];
            }
        }

        Ok((n, input))
    }
}

#[derive(Copy, Clone)]
pub struct NonZeroParser<I: Integer + Parseable>(PhantomData<I>);
impl<'i, I: Integer + Parseable> Leaf<'i> for NonZeroParser<I> {
    type Output = I::NonZero;

    #[inline(always)]
    fn parse(&self, input: &'i [u8]) -> LeafResult<'i, Self::Output> {
        let (value, remaining) = I::PARSER.parse(input)?;
        match I::NonZero::try_from(value) {
            Ok(value) => Ok((value, remaining)),
            Err(_) => Err((ParseError::Expected("non-zero integer"), input)),
        }
    }
}

macro_rules! parser_for {
    ($p:ident => $($n:ident: $nonzero:ident),+ $(,)?) => {$(
        impl Parseable for std::primitive::$n {
            type Parser = $p<std::primitive::$n>;
            const PARSER: Self::Parser = $p(PhantomData);
        }

        #[doc = concat!("[`Leaf`] parser for [`prim@", stringify!($n), "`] values.")]
        #[inline]
        #[must_use]
        pub fn $n() -> $p<std::primitive::$n> {
            $p(PhantomData)
        }

        impl Parseable for std::num::NonZero<$n> {
            type Parser = NonZeroParser<$n>;
            const PARSER: Self::Parser = NonZeroParser(PhantomData);
        }

        #[doc = concat!(
            "[`Leaf`] parser for [`NonZero<", stringify!($n), ">`](std::num::NonZero) values.",
            "\n\n# Examples\n```\n",
            "# use utils::parser::{self, Parser};\n",
            "assert!(parser::", stringify!($nonzero), "().parse_complete(\"0\").is_err());\n",
            "assert!(parser::", stringify!($nonzero), "().parse_complete(\"1\").is_ok_and(|n| n.get() == 1));\n",
            "```"
        )]
        #[inline]
        #[must_use]
        pub fn $nonzero() -> NonZeroParser<std::primitive::$n> {
            NonZeroParser(PhantomData)
        }
    )+};
}
parser_for! {
    UnsignedParser =>
    u8: nonzero_u8,
    u16: nonzero_u16,
    u32: nonzero_u32,
    u64: nonzero_u64,
    u128: nonzero_u128,
}
parser_for! {
    SignedParser =>
    i8: nonzero_i8,
    i16: nonzero_i16,
    i32: nonzero_i32,
    i64: nonzero_i64,
    i128: nonzero_i128,
}

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

impl<'i, I: Integer + Parseable> Leaf<'i> for NumberRange<I> {
    type Output = I;

    #[inline(always)]
    fn parse(&self, input: &'i [u8]) -> LeafResult<'i, Self::Output> {
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

/// [`Leaf`] parser for numbers in the supplied range.
///
/// The type of the number to parse is inferred from the range's type.
///
/// See also [`byte_range`](super::byte_range).
///
/// # Examples
/// ```
/// # use utils::parser::{self, Leaf};
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

impl<'i> Leaf<'i> for Digit {
    type Output = u8;

    #[inline(always)]
    fn parse(&self, input: &'i [u8]) -> LeafResult<'i, Self::Output> {
        if let Some(d @ b'0'..=b'9') = input.first() {
            Ok((d - b'0', &input[1..]))
        } else {
            Err((ParseError::Expected("digit"), input))
        }
    }
}

/// [`Leaf`] parser for single digits.
///
/// # Examples
/// ```
/// # use utils::parser::{self, Leaf};
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

#[derive(Copy, Clone)]
pub struct DigitsRange<U, const MIN: u128, const MAX: u128>(PhantomData<U>);
impl<U: UnsignedInteger, const MIN: u128, const MAX: u128> DigitsRange<U, MIN, MAX> {
    const MIN_DIGITS: usize = (MIN | 1).ilog10() as usize + 1;
    const MAX_DIGITS: usize = (MAX | 1).ilog10() as usize + 1;

    // Ensure too large check is omitted when MAX is the largest MAX_DIGITS number.
    // n < MIN is already skipped when MIN = 0.
    const CHECK_MAX: bool = MAX.saturating_add(1).ilog10() == (MAX | 1).ilog10();
}
impl<'i, U: UnsignedInteger, const MIN: u128, const MAX: u128> Leaf<'i>
    for DigitsRange<U, MIN, MAX>
{
    type Output = U;

    #[inline(always)]
    fn parse(&self, input: &'i [u8]) -> LeafResult<'i, Self::Output> {
        const {
            assert!(MIN <= MAX);
            assert!(Self::MAX_DIGITS <= U::SAFE_DIGITS);
        }

        // Using a fixed-size chunk to avoid bounds checks
        let (n, rem) = if let Some((chunk, mut rem)) = input.split_at_checked(Self::MAX_DIGITS) {
            let mut n = U::ZERO;
            for (i, &b) in chunk.iter().enumerate().take(Self::MIN_DIGITS) {
                let d = U::from(b).wrapping_sub(U::from(b'0'));
                if d > U::from(9) {
                    // cold_path makes these predictable branches instead of conditional moves
                    std::hint::cold_path();
                    return Err((ParseError::Expected("digit"), &input[i..]));
                }
                n = n * U::from(10) + d;
            }
            for (i, &b) in chunk.iter().enumerate().skip(Self::MIN_DIGITS) {
                let d = U::from(b).wrapping_sub(U::from(b'0'));
                if d > U::from(9) {
                    std::hint::cold_path();
                    rem = &input[i..];
                    break;
                }
                n = n * U::from(10) + d;
            }
            (n, rem)
        } else {
            digits_near_end(input, Self::MIN_DIGITS)?
        };

        if n < MIN.try_into().unwrap() {
            Err((ParseError::too_small(MIN), input))
        } else if Self::CHECK_MAX && n > MAX.try_into().unwrap() {
            Err((ParseError::too_large(MAX), input))
        } else {
            Ok((n, rem))
        }
    }
}

#[cold]
#[inline(never)]
fn digits_near_end<U: UnsignedInteger>(input: &[u8], min_digits: usize) -> LeafResult<'_, U> {
    // Always less than max_digits
    let len = input.iter().take_while(|b| b.is_ascii_digit()).count();
    if len < min_digits {
        return Err((ParseError::Expected("digit"), &input[len..]));
    }

    let n = input[..len]
        .iter()
        .fold(U::ZERO, |n, &d| n * U::from(10) + U::from(d - b'0'));
    Ok((n, &input[len..]))
}

/// [`Leaf`] parser for unsigned numbers, reading only digits matching the supplied range.
///
/// This can be faster than [`number_range`] for ranges with a small number of digits, but unlike
/// the normal number parsers it does not check for trailing digits. It should be followed by a
/// parser that wouldn't match digits to ensure the number is in the intended range.
///
/// `MIN` and `MAX` must match the supplied range, and all numbers with as many digits as `MAX`
/// must fit in the unsigned type.
///
/// See also [`parser::digits_range!`](crate::parser::digits_range!).
///
/// # Examples
/// ```
/// # use utils::parser::{self, Leaf};
/// let parser = parser::digits_range::<_, 1, 12>(1u32..=12);
/// assert_eq!(parser.parse(b"07-21"), Ok((7, &b"-21"[..])));
/// assert_eq!(parser.parse(b"123"), Ok((12, &b"3"[..])));
/// assert!(parser.parse(b"13-21").is_err());
/// ```
#[must_use]
#[inline(always)]
#[expect(clippy::inline_always, reason = "ensure assert is optimized out")]
pub fn digits_range<U: UnsignedInteger, const MIN: u128, const MAX: u128>(
    range: RangeInclusive<U>,
) -> DigitsRange<U, MIN, MAX> {
    assert!(*range.start() == MIN.try_into().unwrap() && *range.end() == MAX.try_into().unwrap());
    DigitsRange(PhantomData)
}
