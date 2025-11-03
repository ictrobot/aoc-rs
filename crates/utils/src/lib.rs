//! Common utilities used by the [`aoc`](../aoc/) and year crates.
#![cfg_attr(not(feature = "unsafe"), forbid(unsafe_code))]

pub mod array;
pub mod ascii;
pub mod bit;
pub mod date;
mod r#enum;
mod framework;
pub mod geometry;
pub mod graph;
pub mod grid;
pub mod input;
pub mod md5;
#[cfg(not(target_family = "wasm"))]
pub mod multithreading;
pub mod multiversion;
pub mod number;
pub mod parser;
pub mod queue;
pub mod simd;
pub mod slice;
#[cfg(target_family = "wasm")]
pub mod wasm;

pub use framework::{PuzzleDate, PuzzleExamples};
#[cfg(target_family = "wasm")]
pub use wasm::multithreading;

/// Standard imports for puzzle solutions.
pub mod prelude {
    pub use crate::examples;
    pub use crate::input::{InputError, InputType};
    pub use crate::parser::{self, Parseable as _, Parser as _};
}
