#![allow(non_snake_case)]

use crate::parser::{ParseState, Parser, ParserResult};

pub trait Then<'i, P: Parser<'i>, T: Parser<'i>>: Parser<'i> {
    fn then(parser: P, then: T) -> Self;
}

#[derive(Copy, Clone)]
pub enum Unimplemented {}
impl<'i> Parser<'i> for Unimplemented {
    type Output = Unimplemented;
    type Then<T: Parser<'i>> = Unimplemented;

    fn parse_ctx(
        &self,
        _: &'i [u8],
        _: &mut ParseState<'i>,
        _: &mut bool,
        _: bool,
    ) -> ParserResult<'i, Self::Output> {
        unimplemented!();
    }
}
impl<'i, P: Parser<'i>, T: Parser<'i>> Then<'i, P, T> for Unimplemented {
    fn then(_: P, _: T) -> Self {
        unimplemented!();
    }
}

macro_rules! then_impl {
    (
        $name:ident<$new:ident> => [$($t:ident),+],
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
            #[expect(unused_variables)]
            fn parse_ctx(
                &self,
                input: &'i [u8],
                state: &mut ParseState<'i>,
                commit: &mut bool,
                tail: bool,
            ) -> ParserResult<'i, Self::Output> {
                // Create variables set to false for each $t, then shadow $new to tail to pass it
                // through to the final parser. Then, as each parser is called, shadow the variables
                // again with the output.
                $(let $t = false;)+
                let $new = tail;
                $(let ($t, input) = self.$t.parse_ctx(input, state, commit, $t)?;)+
                Ok((($($t),+), input))
            }
        }
        impl<'i, $($t: Parser<'i>),+, T: Parser<'i>> Then<'i, $name<$($t),+>, T> for $next_name<$($t),+, T> {
            fn then(parser: $name<$($t),+>, next: T) -> Self {
                Self{$($t: parser.$t),+, $next_t: next}
            }
        }
        then_impl!{$next_name<$next_t> => $($tail)*}
    };
    (
        $name:ident<$new:ident> => [$($t:ident),+],
    ) => {
        #[derive(Copy, Clone)]
        pub struct $name<$($t),+>{
            $($t: $t,)+
        }
        impl<'i, $($t: Parser<'i>),+> Parser<'i> for $name<$($t),+> {
            type Output = ($($t::Output),+);
            type Then<T: Parser<'i>> = Unimplemented;

            #[inline(always)]
            #[expect(unused_variables)]
            fn parse_ctx(
                &self,
                input: &'i [u8],
                state: &mut ParseState<'i>,
                commit: &mut bool,
                tail: bool,
            ) -> ParserResult<'i, Self::Output> {
                $(let $t = false;)+
                let $new = tail;
                $(let ($t, input) = self.$t.parse_ctx(input, state, commit, $t)?;)+
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

impl<'i, A: Parser<'i>, B: Parser<'i>> Then<'i, A, B> for Then2<A, B> {
    fn then(parser: A, then: B) -> Self {
        Then2 { A: parser, B: then }
    }
}
