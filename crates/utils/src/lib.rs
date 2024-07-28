//! Common utilities used by the [`aoc`](../aoc/) and year crates.
#![cfg_attr(not(feature = "unsafe"), forbid(unsafe_code))]

pub mod date;
mod framework;
pub mod input;
pub mod md5;
pub mod multithreading;
pub mod multiversion;
pub mod number;
pub mod parser;
pub mod point;
pub mod simd;

pub use framework::{Puzzle, PuzzleExamples};

/// Standard imports for puzzle solutions.
pub mod prelude {
    pub use crate::examples;
    pub use crate::input::{InputError, InputType};
    pub use crate::parser::{self, Parser as _};
}
