#![allow(non_snake_case)]

use crate::parser::{ParseResult, Parser};

pub trait Then<P: Parser, T: Parser>: Parser {
    fn then(parser: P, then: T) -> Self;
}

#[derive(Copy, Clone)]
pub enum Unimplemented {}
impl Parser for Unimplemented {
    type Output<'i> = Unimplemented;
    type Then<T: Parser> = Unimplemented;

    fn parse<'i>(&self, _: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
        unimplemented!();
    }
}
impl<P: Parser, T: Parser> Then<P, T> for Unimplemented {
    fn then(_: P, _: T) -> Self {
        unimplemented!();
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
        impl<$($t: Parser),+> Parser for $name<$($t),+> {
            type Output<'i> = ($($t::Output<'i>),+);
            type Then<T: Parser> = $next_name<$($t),+, T>;

            #[inline(always)]
            fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
                $(let ($t, input) = self.$t.parse(input)?;)+
                Ok((($($t),+), input))
            }
        }
        impl<$($t: Parser),+, T: Parser> Then<$name<$($t),+>, T> for $next_name<$($t),+, T> {
            fn then(parser: $name<$($t),+>, next: T) -> Self {
                Self{$($t: parser.$t),+, $next_t: next}
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
        impl<$($t: Parser),+> Parser for $name<$($t),+> {
            type Output<'i> = ($($t::Output<'i>),+);
            type Then<T: Parser> = Unimplemented;

            #[inline(always)]
            fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
                $(let ($t, input) = self.$t.parse(input)?;)+
                Ok((($($t),+), input))
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

impl<A: Parser, B: Parser> Then<A, B> for Then2<A, B> {
    fn then(parser: A, then: B) -> Self {
        Then2 { A: parser, B: then }
    }
}
