use crate::array::ArrayVec;
use crate::parser::then::Then2;
use crate::parser::{ParseError, ParseState, Parser, ParserResult};

pub struct Commit<P> {
    pub(super) parser: P,
}
impl<'i, P: Parser<'i>> Parser<'i> for Commit<P> {
    type Output = P::Output;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse_ctx(
        &self,
        input: &'i [u8],
        state: &mut ParseState<'i>,
        commit: &mut bool,
        tail: bool,
    ) -> ParserResult<'i, Self::Output> {
        let (v, remaining) = self.parser.parse_ctx(input, state, commit, tail)?;
        if remaining.len() < input.len() {
            *commit = true;
        }
        Ok((v, remaining))
    }
}

#[derive(Copy, Clone)]
pub struct Map<P, F> {
    pub(super) parser: P,
    pub(super) map_fn: F,
}

impl<'i, P: Parser<'i>, F: Fn(P::Output) -> O, O> Parser<'i> for Map<P, F> {
    type Output = O;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse_ctx(
        &self,
        input: &'i [u8],
        state: &mut ParseState<'i>,
        commit: &mut bool,
        tail: bool,
    ) -> ParserResult<'i, Self::Output> {
        let (v, remaining) = self.parser.parse_ctx(input, state, commit, tail)?;
        Ok(((self.map_fn)(v), remaining))
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
    fn parse_ctx(
        &self,
        input: &'i [u8],
        state: &mut ParseState<'i>,
        commit: &mut bool,
        tail: bool,
    ) -> ParserResult<'i, Self::Output> {
        let (v, remaining) = self.parser.parse_ctx(input, state, commit, tail)?;
        match (self.map_fn)(v) {
            Ok(mapped) => Ok((mapped, remaining)),
            Err(e) => Err(state.error(ParseError::Custom(e), remaining)),
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
    fn parse_ctx(
        &self,
        input: &'i [u8],
        state: &mut ParseState<'i>,
        _: &mut bool,
        tail: bool,
    ) -> ParserResult<'i, Self::Output> {
        let mut commit = false;
        match self.parser.parse_ctx(input, state, &mut commit, tail) {
            Ok((v, remaining)) => Ok((Some(v), remaining)),
            Err(t) if commit => Err(t),
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
    fn parse_ctx(
        &self,
        mut input: &'i [u8],
        state: &mut ParseState<'i>,
        commit: &mut bool,
        _: bool,
    ) -> ParserResult<'i, Self::Output> {
        let mut output = [P::Output::default(); N];
        for (i, item) in output.iter_mut().enumerate() {
            (*item, input) = self.parser.parse_ctx(input, state, commit, false)?;

            if i < N - 1 {
                (_, input) = self.separator.parse_ctx(input, state, commit, false)?;
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
    fn parse_ctx(
        &self,
        mut input: &'i [u8],
        state: &mut ParseState<'i>,
        _: &mut bool,
        _: bool,
    ) -> ParserResult<'i, Self::Output> {
        let mut output = ArrayVec::new();

        let mut commit = false;
        let mut input_before_sep = input;
        let token = loop {
            let (v, remaining) = match self.parser.parse_ctx(input, state, &mut commit, false) {
                Ok(v) => v,
                Err(t) => break t,
            };

            commit = false;

            let consumed = input.len() - remaining.len();
            assert!(consumed > 0, "parsing item consumed no input");

            if output.push(v).is_err() {
                return Err(state.error(ParseError::ExpectedLessItems(N), input));
            }
            input_before_sep = remaining;

            match self
                .separator
                .parse_ctx(remaining, state, &mut commit, false)
            {
                Ok((_, remaining)) => input = remaining,
                Err(t) => break t,
            }
        };

        if output.len() < self.min_elements || commit {
            // Return error if not enough elements were parsed, or if the most recent separator or
            // item parser committed but failed to parse another item.
            Err(token)
        } else {
            Ok((output, input_before_sep))
        }
    }
}

#[derive(Copy, Clone)]
pub struct RepeatFold<P, S, A, F> {
    pub(super) parser: P,
    pub(super) separator: S,
    pub(super) min_elements: usize,
    pub(super) init: A,
    pub(super) f: F,
}
impl<'i, P: Parser<'i>, S: Parser<'i>, A: Clone, F: Fn(A, P::Output) -> A> Parser<'i>
    for RepeatFold<P, S, A, F>
{
    type Output = A;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse_ctx(
        &self,
        mut input: &'i [u8],
        state: &mut ParseState<'i>,
        _: &mut bool,
        _: bool,
    ) -> ParserResult<'i, A> {
        let mut acc = self.init.clone();
        let mut elements = 0usize;

        let mut commit = false;
        let mut input_before_sep = input;
        let token = loop {
            let (v, remaining) = match self.parser.parse_ctx(input, state, &mut commit, false) {
                Ok(v) => v,
                Err(t) => break t,
            };

            acc = (self.f)(acc, v);
            elements += 1;

            commit = false;
            input_before_sep = remaining;

            match self
                .separator
                .parse_ctx(remaining, state, &mut commit, false)
            {
                Ok((_, remaining)) => input = remaining,
                Err(t) => break t,
            }
        };

        if elements < self.min_elements || commit {
            // Return error if not enough elements were parsed, or if the most recent separator or
            // item parser committed but failed to parse another item.
            Err(token)
        } else {
            Ok((acc, input_before_sep))
        }
    }
}

#[derive(Copy, Clone)]
pub struct RepeatVec<P, S> {
    pub(super) parser: P,
    pub(super) separator: S,
    pub(super) min_elements: usize,
}
impl<'i, P: Parser<'i>, S: Parser<'i>> Parser<'i> for RepeatVec<P, S> {
    type Output = Vec<P::Output>;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse_ctx(
        &self,
        mut input: &'i [u8],
        state: &mut ParseState<'i>,
        _: &mut bool,
        tail: bool,
    ) -> ParserResult<'i, Vec<P::Output>> {
        let mut output = Vec::new();

        let mut commit = false;
        let mut input_before_sep = input;
        let token = loop {
            let (v, remaining) = match self.parser.parse_ctx(input, state, &mut commit, false) {
                Ok(v) => v,
                Err(t) => break t,
            };

            commit = false;

            let consumed = input.len() - remaining.len();
            assert!(consumed > 0, "parsing item consumed no input");

            // When parsing the entire remaining input, after parsing the first item use the
            // proportion of consumed bytes for one item to reserve capacity for the output vec
            if tail && output.is_empty() {
                output.reserve(((remaining.len() / consumed) * 7 / 5) + 2);
            }

            output.push(v);
            input_before_sep = remaining;

            match self
                .separator
                .parse_ctx(remaining, state, &mut commit, false)
            {
                Ok((_, remaining)) => input = remaining,
                Err(t) => break t,
            }
        };

        if output.len() < self.min_elements || commit || (tail && !input_before_sep.is_empty()) {
            // Return error if:
            // - not enough elements were parsed
            // - the most recent separator or item parser committed but failed to parse another item
            // - the entire input was not consumed when it should have been
            Err(token)
        } else {
            Ok((output, input_before_sep))
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
    fn parse_ctx(
        &self,
        input: &'i [u8],
        state: &mut ParseState<'i>,
        _: &mut bool,
        tail: bool,
    ) -> ParserResult<'i, Self::Output> {
        let mut commit = false;
        match self.first.parse_ctx(input, state, &mut commit, tail) {
            Ok(v) => Ok(v),
            Err(t) if commit => Err(t),
            // The second parser's commit value is ignored as its result is always returned.
            // The parent commit value isn't passed as .or() conceptually has its own scope.
            Err(_) => self.second.parse_ctx(input, state, &mut false, tail),
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
    fn parse_ctx(
        &self,
        input: &'i [u8],
        state: &mut ParseState<'i>,
        commit: &mut bool,
        tail: bool,
    ) -> ParserResult<'i, Self::Output> {
        let (v, remaining) = self.parser.parse_ctx(input, state, commit, tail)?;
        Ok(((v, &input[..input.len() - remaining.len()]), remaining))
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
    fn parse_ctx(
        &self,
        input: &'i [u8],
        state: &mut ParseState<'i>,
        commit: &mut bool,
        tail: bool,
    ) -> ParserResult<'i, Self::Output> {
        let (_, input) = self.prefix.parse_ctx(input, state, commit, false)?;
        self.parser.parse_ctx(input, state, commit, tail)
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
    fn parse_ctx(
        &self,
        input: &'i [u8],
        state: &mut ParseState<'i>,
        commit: &mut bool,
        tail: bool,
    ) -> ParserResult<'i, Self::Output> {
        let (v, input) = self.parser.parse_ctx(input, state, commit, false)?;
        let (_, input) = self.suffix.parse_ctx(input, state, commit, tail)?;
        Ok((v, input))
    }
}
