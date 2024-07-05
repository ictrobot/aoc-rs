//! Common utilities used by the [`aoc`](../aoc/) and year crates.

pub mod date;
mod error;
mod framework;
pub mod input;
pub mod md5;
pub mod number;
pub mod point;

pub use framework::{Puzzle, PuzzleExamples};

/// Standard imports for puzzle solutions.
pub mod prelude {
    pub use crate::examples;
    pub use crate::input::{InputType, InvalidInputError};
}
