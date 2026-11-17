use crate::parser::{Leaf, LeafResult, ParseError};
use crate::str::{TinyStr, TinyStrInt, TinyStrLen};
use std::num::NonZero;

#[derive(Copy, Clone)]
pub struct TinyStringParser<T: TinyStrInt> {
    _phantom: std::marker::PhantomData<T>,
    f: fn(&u8) -> bool,
}
impl<'i, T: TinyStrInt> Leaf<'i> for TinyStringParser<T> {
    type Output = TinyStr<T>;

    #[inline(always)]
    fn parse(&self, input: &'i [u8]) -> LeafResult<'i, Self::Output> {
        // Use a fixed-size chunk to avoid bounds checks
        let Some(chunk) = input.get(..=T::LEN) else {
            return tinystr_near_end(input, self.f)
                .ok_or((ParseError::ExpectedAtLeastMatches(1, self.f), input));
        };

        let mut len = 0;
        let mut mask = T::len_mask(0);
        for b in chunk {
            if !(self.f)(b) {
                break;
            }
            len += 1;
            // Compiled into a constant in the unrolled loop
            mask = T::len_mask(len);
        }

        if len > T::LEN {
            return Err((
                ParseError::ExpectedAtMostMatches(T::LEN, self.f),
                &input[T::LEN..],
            ));
        }

        match T::from_bytes(chunk, mask) {
            None => Err((ParseError::ExpectedAtLeastMatches(1, self.f), input)),
            Some(v) => Ok((TinyStr::from_raw(v), &input[len..])),
        }
    }
}

#[cold]
#[inline(never)]
fn tinystr_near_end<T: TinyStrInt>(
    input: &[u8],
    f: fn(&u8) -> bool,
) -> Option<(TinyStr<T>, &[u8])> {
    // Always at most T::LEN
    let len = input.iter().take_while(|&b| f(b)).count();
    Some((TinyStr::new(&input[..len])?, &input[len..]))
}

/// [`Leaf`] parser for [`TinyStr`] strings between 1 and 2 bytes matching the provided function.
///
/// Similar to [`Parser::repeat_arrayvec`](crate::parser::Parser::repeat_arrayvec) if more than 2
/// bytes match, it will return an error.
///
/// # Examples
/// ```
/// # use utils::parser::{self, Leaf};
/// # use utils::str::TinyStr2;
/// let parser = parser::tinystr2(u8::is_ascii_lowercase);
/// assert_eq!(
///     parser.parse(b"ab: 123"),
///     Ok((TinyStr2::from_const(b"ab"), &b": 123"[..]))
/// );
/// assert!(parser.parse(b"abc: 123").is_err());
/// assert!(parser.parse(b"ABC").is_err());
/// ```
#[inline]
#[must_use]
pub fn tinystr2(f: fn(&u8) -> bool) -> TinyStringParser<NonZero<u16>> {
    TinyStringParser {
        _phantom: std::marker::PhantomData,
        f,
    }
}

/// [`Leaf`] parser for [`TinyStr`] strings between 1 and 4 bytes matching the provided function.
///
/// Similar to [`Parser::repeat_arrayvec`](crate::parser::Parser::repeat_arrayvec) if more than 4
/// bytes match, it will return an error.
///
/// # Examples
/// ```
/// # use utils::parser::{self, Leaf};
/// # use utils::str::TinyStr4;
/// let parser = parser::tinystr4(u8::is_ascii_lowercase);
/// assert_eq!(
///     parser.parse(b"abc: 123"),
///     Ok((TinyStr4::from_const(b"abc"), &b": 123"[..]))
/// );
/// assert_eq!(
///     parser.parse(b"abcd: 123"),
///     Ok((TinyStr4::from_const(b"abcd"), &b": 123"[..]))
/// );
/// assert!(parser.parse(b"abcde: 123").is_err());
/// assert!(parser.parse(b"ABC").is_err());
/// ```
#[inline]
#[must_use]
pub fn tinystr4(f: fn(&u8) -> bool) -> TinyStringParser<NonZero<u32>> {
    TinyStringParser {
        _phantom: std::marker::PhantomData,
        f,
    }
}

/// [`Leaf`] parser for [`TinyStr`] strings between 1 and 8 bytes matching the provided function.
///
/// Similar to [`Parser::repeat_arrayvec`](crate::parser::Parser::repeat_arrayvec) if more than 8
/// bytes match, it will return an error.
///
/// # Examples
/// ```
/// # use utils::parser::{self, Leaf};
/// # use utils::str::TinyStr8;
/// let parser = parser::tinystr8(u8::is_ascii_lowercase);
/// assert_eq!(
///     parser.parse(b"abcde: 123"),
///     Ok((TinyStr8::from_const(b"abcde"), &b": 123"[..]))
/// );
/// assert_eq!(
///     parser.parse(b"abcdefgh: 123"),
///     Ok((TinyStr8::from_const(b"abcdefgh"), &b": 123"[..]))
/// );
/// assert!(parser.parse(b"abcdefghi: 123").is_err());
/// assert!(parser.parse(b"ABC").is_err());
/// ```
#[inline]
#[must_use]
pub fn tinystr8(f: fn(&u8) -> bool) -> TinyStringParser<NonZero<u64>> {
    TinyStringParser {
        _phantom: std::marker::PhantomData,
        f,
    }
}

#[derive(Copy, Clone)]
pub struct TinyStringExact<const N: usize> {
    f: fn(&u8) -> bool,
}
impl<'i, const N: usize> Leaf<'i> for TinyStringExact<N>
where
    (): TinyStrLen<N>,
{
    type Output = TinyStr<<() as TinyStrLen<N>>::Int>;

    #[inline(always)]
    fn parse(&self, input: &'i [u8]) -> LeafResult<'i, Self::Output> {
        let mut accumulator = <<() as TinyStrLen<N>>::Int as TinyStrInt>::Raw::default();

        for i in 0..N {
            if let Some(&b) = input.get(i)
                && (self.f)(&b)
            {
                accumulator =
                    <<() as TinyStrLen<N>>::Int as TinyStrInt>::set_raw(accumulator, b, i);
            } else {
                return Err((ParseError::ExpectedExactlyMatches(N, self.f), &input[i..]));
            }
        }

        match <<() as TinyStrLen<N>>::Int as TinyStrInt>::from_raw(accumulator) {
            None => Err((ParseError::ExpectedExactlyMatches(N, self.f), input)),
            Some(v) => Ok((TinyStr::from_raw(v), &input[N..])),
        }
    }
}

/// [`Leaf`] parser for [`TinyStr`] strings exactly `N` long matching the provided function.
///
/// # Examples
/// ```
/// # use utils::parser::{self, Leaf};
/// # use utils::str::TinyStr4;
/// let parser = parser::tinystr::<3>(u8::is_ascii_lowercase);
/// assert_eq!(
///     parser.parse(b"abc: 123"),
///     Ok((TinyStr4::from_const(b"abc"), &b": 123"[..]))
/// );
/// assert_eq!(
///     parser.parse(b"abcd: 123"),
///     Ok((TinyStr4::from_const(b"abc"), &b"d: 123"[..]))
/// );
/// assert!(parser.parse(b"ab: 123").is_err());
/// assert!(parser.parse(b"ABC").is_err());
/// ```
#[inline]
#[must_use]
pub fn tinystr<const N: usize>(f: fn(&u8) -> bool) -> TinyStringExact<N>
where
    (): TinyStrLen<N>,
{
    TinyStringExact { f }
}
