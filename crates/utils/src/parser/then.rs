#![allow(non_snake_case)]

use crate::parser::{ParseResult, Parser};

#[derive(Copy, Clone)]
pub enum Unimplemented {}
impl<'i> Parser<'i> for Unimplemented {
    type Output = Unimplemented;
    type Then<T: Parser<'i>> = Unimplemented;

    fn parse(&self, _: &'i [u8]) -> ParseResult<'i, Self::Output> {
        unimplemented!();
    }

    fn then<T: Parser<'i>>(self, _: T) -> Self::Then<T> {
        unimplemented!()
    }
}

macro_rules! then_impl {
    (
        $name:ident<$_:ident> => [$($t:ident),+],
        $next_name:ident<$next_t:ident> => $($tail:tt)*
    ) => {
        #[derive(Copy, Clone)]
        pub struct $name<$($t),+>{
            $($t: $t,)+
        }
        impl<'i, $($t: Parser<'i>),+> Parser<'i> for $name<$($t),+> {
            type Output = ($($t::Output),+);
            type Then<T: Parser<'i>> = $next_name<$($t),+, T>;

            #[inline(always)]
            #[allow(clippy::inline_always)] // Required for parsing of long then chains to be inlined
            fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
                $(let ($t, input) = self.$t.parse(input)?;)+
                Ok((($($t),+), input))
            }

            fn then<T: Parser<'i>>(self, next: T) -> Self::Then<T> {
                $next_name{$($t: self.$t),+, $next_t: next}
            }
        }
        then_impl!{$next_name<$next_t> => $($tail)*}
    };
    (
        $name:ident<$_:ident> => [$($t:ident),+],
    ) => {
        #[derive(Copy, Clone)]
        pub struct $name<$($t),+>{
            $($t: $t,)+
        }
        impl<'i, $($t: Parser<'i>),+> Parser<'i> for $name<$($t),+> {
            type Output = ($($t::Output),+);
            type Then<T: Parser<'i>> = Unimplemented;

            #[inline(always)]
            #[allow(clippy::inline_always)] // Required for parsing of long then chains to be inlined
            fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output> {
                $(let ($t, input) = self.$t.parse(input)?;)+
                Ok((($($t),+), input))
            }

            fn then<T: Parser<'i>>(self, _: T) -> Self::Then<T> {
                unimplemented!()
            }
        }
    };
}

then_impl! {
    Then2<B> => [A, B],
    Then3<C> => [A, B, C],
    Then4<D> => [A, B, C, D],
    Then5<E> => [A, B, C, D, E],
    Then6<F> => [A, B, C, D, E, F],
    Then7<G> => [A, B, C, D, E, F, G],
    Then8<H> => [A, B, C, D, E, F, G, H],
}

impl<'i, A: Parser<'i>, B: Parser<'i>> Then2<A, B> {
    pub(super) fn new(first: A, second: B) -> Self {
        Self {
            A: first,
            B: second,
        }
    }
}
