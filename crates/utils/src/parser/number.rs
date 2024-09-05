use crate::number::{SignedInteger, UnsignedInteger};
use crate::parser::then::Then2;
use crate::parser::{ParseError, ParseResult, Parser};
use std::any::type_name;
use std::marker::PhantomData;

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
            _ => return Err((ParseError::Expected(type_name::<U>()), input)),
        };

        while let Some(d @ b'0'..=b'9') = input.first() {
            n = n
                .checked_mul(U::from(10))
                .and_then(|n| n.checked_add(U::from(d - b'0')))
                .ok_or((ParseError::OutOfRange(type_name::<U>()), input))?;
            input = &input[1..];
        }

        Ok((n, input))
    }

    fn then<T: Parser<'i>>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
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
            _ => return Err((ParseError::Expected(type_name::<S>()), input)),
        };

        if positive {
            while let Some(d @ b'0'..=b'9') = input.first() {
                n = n
                    .checked_mul(S::from(10))
                    .and_then(|n| n.checked_add(S::from((d - b'0') as i8)))
                    .ok_or((ParseError::OutOfRange(type_name::<S>()), input))?;
                input = &input[1..];
            }
        } else {
            while let Some(d @ b'0'..=b'9') = input.first() {
                n = n
                    .checked_mul(S::from(10))
                    .and_then(|n| n.checked_sub(S::from((d - b'0') as i8)))
                    .ok_or((ParseError::OutOfRange(type_name::<S>()), input))?;
                input = &input[1..];
            }
        }

        Ok((n, input))
    }

    fn then<T: Parser<'i>>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

macro_rules! parser_fn {
    ($p:ident => $($n:ident),+) => {$(
        #[doc = concat!("Parser for [`prim@", stringify!($n), "`] values.")]
        #[must_use]
        pub fn $n() -> $p<std::primitive::$n> {
            $p(PhantomData::default())
        }
    )+};
}
parser_fn! { UnsignedParser => u8, u16, u32, u64, u128 }
parser_fn! { SignedParser => i8, i16, i32, i64, i128 }
