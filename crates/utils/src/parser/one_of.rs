use crate::parser::then::Then2;
use crate::parser::{ParseState, Parser, ParserResult};

/// Use a second trait to force usage of the [`one_of`] method, preventing tuples from being used as
/// parsers directly, which could be confusing.
#[doc(hidden)]
pub trait ParserOneOfTuple<'i> {
    type Output;
    fn one_of(
        &self,
        input: &'i [u8],
        state: &mut ParseState<'i>,
        tail: bool,
    ) -> ParserResult<'i, Self::Output>;
}

macro_rules! one_of_impl {
    ($($l:ident: $n:tt),+) => {
        impl<'i, A: Parser<'i>, $($l: Parser<'i, Output = A::Output>),+> ParserOneOfTuple<'i> for (A, $($l,)*) {
            type Output = A::Output;

            #[inline(always)]
            fn one_of(
                &self,
                input: &'i [u8],
                state: &mut ParseState<'i>,
                tail: bool,
            ) -> ParserResult<'i, Self::Output> {
                let mut commit = false;
                let token = match self.0.parse_ctx(input, state, &mut commit, tail) {
                    Ok(v) => return Ok(v),
                    Err(t) if commit => return Err(t),
                    Err(t) => t,
                };

                $(
                let mut commit = false;
                match self.$n.parse_ctx(input, state, &mut commit, tail) {
                    Ok(v) => return Ok(v),
                    Err(t) if commit => return Err(t),
                    Err(_) => {},
                }
                )+

                Err(token)
            }
        }
    };
}

one_of_impl! {B: 1}
one_of_impl! {B: 1, C: 2}
one_of_impl! {B: 1, C: 2, D: 3}
one_of_impl! {B: 1, C: 2, D: 3, E: 4}
one_of_impl! {B: 1, C: 2, D: 3, E: 4, F: 5}
one_of_impl! {B: 1, C: 2, D: 3, E: 4, F: 5, G: 6}
one_of_impl! {B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7}
one_of_impl! {B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8}
one_of_impl! {B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9}
one_of_impl! {B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10}
one_of_impl! {B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11}

#[derive(Copy, Clone)]
pub struct OneOf<O> {
    options: O,
}
impl<'i, O: ParserOneOfTuple<'i>> Parser<'i> for OneOf<O> {
    type Output = O::Output;
    type Then<T: Parser<'i>> = Then2<Self, T>;

    #[inline]
    fn parse_ctx(
        &self,
        input: &'i [u8],
        state: &mut ParseState<'i>,
        _: &mut bool,
        tail: bool,
    ) -> ParserResult<'i, Self::Output> {
        self.options.one_of(input, state, tail)
    }
}

/// [`Parser`] which tries a list of parsers in order until one succeeds.
///
/// If a parser commits, no further parsers are tried.
///
/// This is similar to [`Parser::or`] but supports a variable number of parsers.
///
/// Prefer [`parser::literal_map`](super::literal_map) if all the parsers are string literals.
///
/// # Examples
/// ```
/// # use utils::input::InputError;
/// # use utils::parser::{self, ParseError, Parser};
/// #[derive(Debug, PartialEq)]
/// enum Value {
///     Unsigned8(u8),
///     Unsigned32(u32),
///     Signed8(i8),
///     Signed32(i32),
/// }
///
/// let parser = parser::one_of((
///     parser::u8().map(Value::Unsigned8),
///     parser::u32().map(Value::Unsigned32),
///     parser::i8().map(Value::Signed8),
///     parser::i32().map(Value::Signed32),
/// ));
///
/// assert_eq!(
///     parser.parse_complete("31").unwrap(),
///     Value::Unsigned8(31),
/// );
/// assert_eq!(
///     parser.parse_complete("4294967295").unwrap(),
///     Value::Unsigned32(4294967295),
/// );
/// assert_eq!(
///     parser.parse_complete("-1").unwrap(),
///     Value::Signed8(-1)
/// );
/// assert_eq!(
///     parser.parse_complete("-2147483648").unwrap(),
///     Value::Signed32(-2147483648)
/// );
///
/// assert_eq!(
///     parser.parse_complete("not a number").unwrap_err().into_source(),
///     ParseError::Expected("unsigned integer")
/// );
/// assert_eq!(
///     parser.parse_complete("-4294967295").unwrap_err().into_source(),
///     ParseError::NumberTooSmall(-2147483648)
/// );
/// ```
#[inline]
#[must_use]
pub fn one_of<'i, L: ParserOneOfTuple<'i>>(options: L) -> OneOf<L> {
    OneOf { options }
}
