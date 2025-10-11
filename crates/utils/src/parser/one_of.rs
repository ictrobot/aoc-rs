use crate::parser::then::Then2;
use crate::parser::{ParseResult, Parser};

/// Use a second trait to force usage of the [`one_of`] method, preventing tuples from being used as
/// parsers directly, which could be confusing.
pub trait ParserOneOfTuple<'i> {
    type Output;
    fn one_of(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output>;
}

macro_rules! one_of_impl {
    ($($l:ident: $n:tt),+) => {
        impl<'i, A: Parser<'i>, $($l: Parser<'i, Output = A::Output>),+> ParserOneOfTuple<'i> for (A, $($l,)*) {
            type Output = A::Output;

            #[inline(always)]
            fn one_of(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
                let mut err = match self.0.parse(input) {
                    Ok(v) => return Ok(v),
                    Err(err) => err,
                };

                $(match self.$n.parse(input) {
                    Ok(v) => return Ok(v),
                    Err(this_err) => {
                        if this_err.1.len() < err.1.len() {
                            err = this_err;
                        }
                    }
                })+

                Err(err)
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
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
        self.options.one_of(input)
    }
}

/// Attempt to parse using a list of parsers.
///
/// Similar to [`Parser::or`], each parser will be tried in order until one succeeds. If no parsers
/// succeed, the error from the parser furthest into the input is returned.
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
///     parser.parse(b"not a number").unwrap_err().0,
///     ParseError::Expected("unsigned integer")
/// );
/// assert_eq!(
///     parser.parse(b"-4294967295").unwrap_err().0,
///     ParseError::NumberTooSmall(-2147483648)
/// );
/// ```
#[inline]
#[must_use]
pub fn one_of<'i, L: ParserOneOfTuple<'i>>(options: L) -> OneOf<L> {
    OneOf { options }
}
