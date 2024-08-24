use crate::parser::then::Then2;
use crate::parser::{ParseError, ParseResult, Parser};

#[derive(Copy, Clone)]
pub struct Map<P, F> {
    pub(super) parser: P,
    pub(super) map_fn: F,
}

impl<'i, P: Parser<'i>, F: Fn(P::Output) -> O, O> Parser<'i> for Map<P, F> {
    type Output = O;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        match self.parser.parse(input) {
            Ok((v, remaining)) => Ok(((self.map_fn)(v), remaining)),
            Err(e) => Err(e),
        }
    }

    fn then<T: Parser<'i>>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

#[derive(Copy, Clone)]
pub struct MapResult<P, F> {
    pub(super) parser: P,
    pub(super) map_fn: F,
}

impl<'i, P: Parser<'i>, F: Fn(P::Output) -> Result<O, &'static str>, O> Parser<'i>
    for MapResult<P, F>
{
    type Output = O;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        match self.parser.parse(input) {
            Ok((v, remaining)) => match (self.map_fn)(v) {
                Ok(mapped) => Ok((mapped, remaining)),
                Err(e) => Err((ParseError::Custom(e), input)),
            },
            Err(e) => Err(e),
        }
    }

    fn then<T: Parser<'i>>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

#[derive(Copy, Clone)]
pub struct Optional<P> {
    pub(super) parser: P,
}
impl<'i, P: Parser<'i>> Parser<'i> for Optional<P> {
    type Output = Option<P::Output>;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        match self.parser.parse(input) {
            Ok((v, remaining)) => Ok((Some(v), remaining)),
            Err(_) => Ok((None, input)),
        }
    }

    fn then<T: Parser<'i>>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

#[derive(Copy, Clone)]
pub struct Repeat<const N: usize, P> {
    pub(super) parser: P,
}
impl<'i, const N: usize, P: Parser<'i, Output: Copy + Default>> Parser<'i> for Repeat<N, P> {
    type Output = [P::Output; N];
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, mut input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        let mut output = [P::Output::default(); N];
        for item in &mut output {
            match self.parser.parse(input) {
                Ok((v, remaining)) => {
                    *item = v;
                    input = remaining;
                }
                Err(e) => return Err(e),
            }
        }
        Ok((output, input))
    }

    fn then<T: Parser<'i>>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

#[derive(Copy, Clone)]
pub struct Or<A, B> {
    pub(super) first: A,
    pub(super) second: B,
}
impl<'i, A: Parser<'i>, B: Parser<'i, Output = A::Output>> Parser<'i> for Or<A, B> {
    type Output = A::Output;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        match self.first.parse(input) {
            Ok(v) => Ok(v),
            Err((err1, remaining1)) => match self.second.parse(input) {
                Ok(v) => Ok(v),
                Err((err2, remaining2)) => {
                    // Return error from the parser which processed further, or the first if equal
                    Err(if remaining1.len() <= remaining2.len() {
                        (err1, remaining1)
                    } else {
                        (err2, remaining2)
                    })
                }
            },
        }
    }

    fn then<T: Parser<'i>>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

#[derive(Copy, Clone)]
pub struct WithPrefix<A, B> {
    pub(super) parser: A,
    pub(super) prefix: B,
}
impl<'i, A: Parser<'i>, B: Parser<'i>> Parser<'i> for WithPrefix<A, B> {
    type Output = A::Output;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        match self.prefix.parse(input) {
            Ok((_, remaining)) => self.parser.parse(remaining),
            Err(e) => Err(e),
        }
    }

    fn then<T: Parser<'i>>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

#[derive(Copy, Clone)]
pub struct WithSuffix<A, B> {
    pub(super) parser: A,
    pub(super) suffix: B,
}
impl<'i, A: Parser<'i>, B: Parser<'i>> Parser<'i> for WithSuffix<A, B> {
    type Output = A::Output;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        match self.parser.parse(input) {
            Ok((v, remaining1)) => match self.suffix.parse(remaining1) {
                Ok((_, remaining2)) => Ok((v, remaining2)),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }

    fn then<T: Parser<'i>>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}
