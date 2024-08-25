//! Common utilities used by the [`aoc`](../aoc/) and year crates.
#![cfg_attr(not(feature = "unsafe"), forbid(unsafe_code))]

pub mod bit;
pub mod date;
mod framework;
pub mod grid;
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
    pub use crate::input::{InputError, InputType, MapWithInputExt as _};
    pub use crate::parser::{self, Parser as _};
}
