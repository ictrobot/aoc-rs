#![allow(non_snake_case)]

use crate::parser::{ParseResult, Parser};

#[derive(Copy, Clone)]
pub enum Unimplemented {}
impl Parser for Unimplemented {
    type Output<'i> = Unimplemented;
    type Then<T: Parser> = Unimplemented;

    fn parse<'i>(&self, _: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
        unimplemented!();
    }

    fn then<T: Parser>(self, _: T) -> Self::Then<T> {
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
        impl<$($t: Parser),+> Parser for $name<$($t),+> {
            type Output<'i> = ($($t::Output<'i>),+);
            type Then<T: Parser> = $next_name<$($t),+, T>;

            #[inline(always)]
            fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
                $(let ($t, input) = self.$t.parse(input)?;)+
                Ok((($($t),+), input))
            }

            fn then<T: Parser>(self, next: T) -> Self::Then<T> {
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
        impl<$($t: Parser),+> Parser for $name<$($t),+> {
            type Output<'i> = ($($t::Output<'i>),+);
            type Then<T: Parser> = Unimplemented;

            #[inline(always)]
            fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Output<'i>> {
                $(let ($t, input) = self.$t.parse(input)?;)+
                Ok((($($t),+), input))
            }

            fn then<T: Parser>(self, _: T) -> Self::Then<T> {
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

impl<A: Parser, B: Parser> Then2<A, B> {
    pub(super) fn new(first: A, second: B) -> Self {
        Self {
            A: first,
            B: second,
        }
    }
}
