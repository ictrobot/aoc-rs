//! Items relating to puzzle input.

use crate::error_type;
use std::fmt::Formatter;
use std::num::ParseIntError;

/// Enum for distinguishing between example and real inputs.
///
/// Some puzzles require this as different constants may be used for example inputs to simplify the
/// problem. For example [2022 day 15](https://adventofcode.com/2022/day/15) part 1, which uses
/// `y=10` in the example, but `y=2000000` for real inputs.
///
/// Most puzzle solutions should ignore this value.
#[allow(clippy::module_name_repetitions)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum InputType {
    Example,
    Real,
}

error_type! {
    /// Error type which should be returned by puzzle `new` functions
    pub enum InvalidInputError {
        UnexpectedChar(char),
        UnexpectedString(String),
    } wraps [
        ParseIntError
    ]
    impl Display match {
        Self::UnexpectedChar(c) => |f: &mut Formatter| write!(f, "unexpected character '{c}'"),
        Self::UnexpectedString(s) => |f: &mut Formatter| write!(f, "unexpected string '{s}'"),
    }
}
