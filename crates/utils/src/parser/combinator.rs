use crate::array::ArrayVec;
use crate::input::{InputError, MapWithInputExt};
use crate::parser::then::Then2;
use crate::parser::{ParseError, ParseResult, Parser};

#[derive(Copy, Clone)]
pub struct Map<P, F> {
    pub(super) parser: P,
    pub(super) map_fn: F,
}

impl<P: Parser, F: for<'i> Fn(P::Output<'i>) -> O, O> Parser for Map<P, F> {
    type Output<'i> = O;
    type Then<T: Parser> = Then2<Self, T>;

    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
        match self.parser.parse(input) {
            Ok((v, remaining)) => Ok(((self.map_fn)(v), remaining)),
            Err(e) => Err(e),
        }
    }

    fn then<T: Parser>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

#[derive(Copy, Clone)]
pub struct MapResult<P, F> {
    pub(super) parser: P,
    pub(super) map_fn: F,
}

impl<P: Parser, F: for<'i> Fn(P::Output<'i>) -> Result<O, &'static str>, O> Parser
    for MapResult<P, F>
{
    type Output<'i> = O;
    type Then<T: Parser> = Then2<Self, T>;

    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
        match self.parser.parse(input) {
            Ok((v, remaining)) => match (self.map_fn)(v) {
                Ok(mapped) => Ok((mapped, remaining)),
                Err(e) => Err((ParseError::Custom(e), input)),
            },
            Err(e) => Err(e),
        }
    }

    fn then<T: Parser>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

#[derive(Copy, Clone)]
pub struct Optional<P> {
    pub(super) parser: P,
}
impl<P: Parser> Parser for Optional<P> {
    type Output<'i> = Option<P::Output<'i>>;
    type Then<T: Parser> = Then2<Self, T>;

    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
        match self.parser.parse(input) {
            Ok((v, remaining)) => Ok((Some(v), remaining)),
            Err(_) => Ok((None, input)),
        }
    }

    fn then<T: Parser>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

#[derive(Copy, Clone)]
pub struct RepeatN<const N: usize, P> {
    pub(super) parser: P,
}
impl<const N: usize, P: for<'i> Parser<Output<'i>: Copy + Default>> Parser for RepeatN<N, P> {
    type Output<'i> = [P::Output<'i>; N];
    type Then<T: Parser> = Then2<Self, T>;

    #[inline]
    fn parse<'i>(&self, mut input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
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

    fn then<T: Parser>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

#[derive(Copy, Clone)]
pub struct RepeatArrayVec<const N: usize, P, S> {
    pub(super) parser: P,
    pub(super) separator: S,
    pub(super) min_elements: usize,
}
impl<const N: usize, P: for<'i> Parser<Output<'i>: Copy + Default>, S: Parser> Parser
    for RepeatArrayVec<N, P, S>
{
    type Output<'i> = ArrayVec<P::Output<'i>, N>;
    type Then<T: Parser> = Then2<Self, T>;

    #[inline]
    fn parse<'i>(&self, mut input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
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

    fn then<T: Parser>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

#[derive(Copy, Clone)]
pub struct RepeatVec<P, S> {
    pub(super) parser: P,
    pub(super) separator: S,
    pub(super) min_elements: usize,
}
impl<P: Parser, S: Parser> RepeatVec<P, S> {
    #[inline]
    fn helper<'i>(
        &self,
        mut input: &'i [u8],
        consume_all: bool,
    ) -> ParseResult<'i, Vec<P::Output<'i>>> {
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
impl<P: Parser, S: Parser> Parser for RepeatVec<P, S> {
    type Output<'i> = Vec<P::Output<'i>>;
    type Then<T: Parser> = Then2<Self, T>;

    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
        self.helper(input, false)
    }

    fn then<T: Parser>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }

    // Override the default implementation to set consume_all to true
    fn parse_complete<'i>(&self, input: &'i str) -> Result<Self::Output<'i>, InputError> {
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
impl<A: Parser, B: for<'i> Parser<Output<'i> = A::Output<'i>>> Parser for Or<A, B> {
    type Output<'i> = A::Output<'i>;
    type Then<T: Parser> = Then2<Self, T>;

    #[inline(always)]
    #[expect(
        clippy::inline_always,
        reason = "required for parsing of long or chains to be inlined"
    )]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
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

    fn then<T: Parser>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

#[derive(Copy, Clone)]
pub struct WithPrefix<A, B> {
    pub(super) parser: A,
    pub(super) prefix: B,
}
impl<A: Parser, B: Parser> Parser for WithPrefix<A, B> {
    type Output<'i> = A::Output<'i>;
    type Then<T: Parser> = Then2<Self, T>;

    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
        match self.prefix.parse(input) {
            Ok((_, remaining)) => self.parser.parse(remaining),
            Err(e) => Err(e),
        }
    }

    fn then<T: Parser>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}

#[derive(Copy, Clone)]
pub struct WithSuffix<A, B> {
    pub(super) parser: A,
    pub(super) suffix: B,
}
impl<A: Parser, B: Parser> Parser for WithSuffix<A, B> {
    type Output<'i> = A::Output<'i>;
    type Then<T: Parser> = Then2<Self, T>;

    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
        match self.parser.parse(input) {
            Ok((v, remaining1)) => match self.suffix.parse(remaining1) {
                Ok((_, remaining2)) => Ok((v, remaining2)),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }

    fn then<T: Parser>(self, next: T) -> Self::Then<T> {
        Then2::new(self, next)
    }
}
