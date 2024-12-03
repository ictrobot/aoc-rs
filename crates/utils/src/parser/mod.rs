//! Parser combinator library.

mod base;
mod combinator;
mod error;
mod iterator;
mod macros;
mod number;
mod one_of;
mod simple;
mod then;

pub use base::*;
pub use error::ParseError;
pub use iterator::{ParserIterator, ParserMatchesIterator};
pub use number::{i128, i16, i32, i64, i8, number_range, u128, u16, u32, u64, u8};
pub use one_of::one_of;
pub use simple::{byte, byte_range, constant, eof, eol, noop, take_while, take_while1};

pub use crate::parser_literal_map as literal_map;
pub use crate::parser_parse_tree as parse_tree;
