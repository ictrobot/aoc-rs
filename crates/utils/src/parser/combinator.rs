use crate::array::ArrayVec;
use crate::input::{InputError, MapWithInputExt};
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
}

#[derive(Copy, Clone)]
pub struct RepeatN<const N: usize, P, S> {
    pub(super) parser: P,
    pub(super) separator: S,
}
impl<'i, const N: usize, P: Parser<'i, Output: Copy + Default>, S: Parser<'i>> Parser<'i>
    for RepeatN<N, P, S>
{
    type Output = [P::Output; N];
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, mut input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        let mut output = [P::Output::default(); N];
        for (i, item) in output.iter_mut().enumerate() {
            match self.parser.parse(input) {
                Ok((v, remaining)) => {
                    *item = v;
                    input = remaining;
                }
                Err(e) => return Err(e),
            }

            if i < N - 1 {
                match self.separator.parse(input) {
                    Ok((_, remaining)) => input = remaining,
                    Err(e) => return Err(e),
                }
            }
        }
        Ok((output, input))
    }
}

#[derive(Copy, Clone)]
pub struct RepeatArrayVec<const N: usize, P, S> {
    pub(super) parser: P,
    pub(super) separator: S,
    pub(super) min_elements: usize,
}
impl<'i, const N: usize, P: Parser<'i, Output: Copy + Default>, S: Parser<'i>> Parser<'i>
    for RepeatArrayVec<N, P, S>
{
    type Output = ArrayVec<P::Output, N>;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, mut input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        let mut output = ArrayVec::new();

        let err = loop {
            let (v, remaining) = match self.parser.parse(input) {
                Ok(v) => v,
                Err(err) => break err,
            };

            let consumed = input.len() - remaining.len();
            assert!(consumed > 0, "parsing item consumed no input");

            if output.push(v).is_err() {
                return Err((ParseError::ExpectedLessItems(N), input));
            }
            input = remaining;

            match self.separator.parse(input) {
                Ok((_, remaining)) => input = remaining,
                Err(err) => break err,
            }
        };

        if output.len() >= self.min_elements {
            Ok((output, input))
        } else {
            Err(err)
        }
    }
}

#[derive(Copy, Clone)]
pub struct RepeatVec<P, S> {
    pub(super) parser: P,
    pub(super) separator: S,
    pub(super) min_elements: usize,
}
impl<'i, P: Parser<'i>, S: Parser<'i>> RepeatVec<P, S> {
    #[inline]
    fn helper(&self, mut input: &'i [u8], consume_all: bool) -> ParseResult<'i, Vec<P::Output>> {
        let mut output = Vec::new();

        let err = loop {
            let (v, remaining) = match self.parser.parse(input) {
                Ok(v) => v,
                Err(err) => break err,
            };

            let consumed = input.len() - remaining.len();
            assert!(consumed > 0, "parsing item consumed no input");

            // When parsing the complete input, after parsing the first item use the proportion of
            // consumed bytes for one item to reserve capacity for the output vec
            if consume_all && output.is_empty() {
                output.reserve(((remaining.len() / consumed) * 7 / 5) + 2);
            }

            output.push(v);
            input = remaining;

            match self.separator.parse(input) {
                Ok((_, remaining)) => input = remaining,
                Err(err) => break err,
            }
        };

        if (consume_all && !input.is_empty()) || output.len() < self.min_elements {
            // Return the last parsing error if this parser should consume the entire input and it
            // hasn't, or if the minimum number of elements isn't met.
            Err(err)
        } else {
            Ok((output, input))
        }
    }
}
impl<'i, P: Parser<'i>, S: Parser<'i>> Parser<'i> for RepeatVec<P, S> {
    type Output = Vec<P::Output>;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        self.helper(input, false)
    }

    // Override the default implementation to set consume_all to true
    fn parse_complete(&self, input: &'i str) -> Result<Self::Output, InputError> {
        match self.helper(input.as_bytes(), true).map_with_input(input)? {
            (v, []) => Ok(v),
            (_, remaining) => Err(InputError::new(input, remaining, "expected end of input")),
        }
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

    #[inline(always)]
    #[expect(
        clippy::inline_always,
        reason = "required for parsing of long or chains to be inlined"
    )]
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
}

#[derive(Copy, Clone)]
pub struct WithConsumed<P> {
    pub(super) parser: P,
}
impl<'i, P: Parser<'i>> Parser<'i> for WithConsumed<P> {
    type Output = (P::Output, &'i [u8]);
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        match self.parser.parse(input) {
            Ok((v, remaining)) => Ok(((v, &input[..input.len() - remaining.len()]), remaining)),
            Err(e) => Err(e),
        }
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
}
