//! Implementation for [`all_puzzles!`](crate::all_puzzles!).
//!
//! Each of the year crates is re-exported below if the corresponding feature is enabled, or a
//! placeholder crate with the [`utils::puzzles_noop!`] no-op `puzzles!` macro if it is not.
//! The `puzzles!` macros are then chained together by [`all_puzzles!`](crate::all_puzzles!).
//!
//! This main advantage of this approach over passing `#[cfg(feature = ?)]` attributes to the
//! callback is that it prevents disabled years from being expanded at all, which should speed up
//! compilation.
//!
//! Running `cargo xtask update` will automatically update the list of re-exports.

// xtask update re-exports
#[cfg(not(feature = "year2015"))]
pub mod year2015 {
    pub use ::utils::puzzles_noop as puzzles;
}
#[cfg(not(feature = "year2016"))]
pub mod year2016 {
    pub use ::utils::puzzles_noop as puzzles;
}
#[cfg(not(feature = "year2017"))]
pub mod year2017 {
    pub use ::utils::puzzles_noop as puzzles;
}
#[cfg(feature = "year2015")]
pub use ::year2015;
#[cfg(feature = "year2016")]
pub use ::year2016;
#[cfg(feature = "year2017")]
pub use ::year2017;

/// Macro which invokes a callback macro with a list of all implemented puzzle solutions.
///
/// This macro chains `puzzles!` macros in the re-exported year modules. The callback macro will be
/// called once with all the solutions, which makes it easy to generate match statements or arrays.
///
/// Running `cargo xtask update` will automatically update the chain of year macros.
///
/// # Examples
///
/// Simple `main` function to run all examples:
///
/// ```
/// # use aoc::all_puzzles;
/// # use utils::PuzzleExamples;
/// # use utils::input::InputType;
/// #
/// macro_rules! callback {
///     ($(
///         $y:literal => $year:ident{$(
///             $d:literal => $day:ident,
///         )*}
///     )*) => {$($(
///         println!("{} {}", $y, $d);
///         for (input_str, p1, p2) in aoc::$year::$day::EXAMPLES {
///             let solution = aoc::$year::$day::new(input_str, InputType::Example).unwrap();
///             println!("  parse({input_str}) = {solution:?}");
///             if (p1.is_some()) { println!("  part1(...) = {}", solution.part1()); }
///             if (p2.is_some()) { println!("  part2(...) = {}", solution.part2()); }
///         }
///     )*)*};
/// }
/// all_puzzles!(callback);
/// ```
///
/// Generating a match statement:
///
/// ```
/// # use aoc::all_puzzles;
/// # use utils::date::{Day, Year};
/// # use utils::{PuzzleExamples, Puzzle};
/// #
/// fn example_count(year: Year, day: Day) -> Option<usize> {
///     macro_rules! callback {
///         ($(
///             $y:literal => $year:ident{$(
///                 $d:literal => $day:ident,
///             )*}
///         )*) => {
///             match (year, day) {
///                 $($(
///                     (aoc::$year::$day::YEAR, aoc::$year::$day::DAY) => Some(aoc::$year::$day::EXAMPLES.len()),
///                 )*)*
///                 _ => None,
///             }
///         };
///     }
///     all_puzzles!{callback}
/// }
/// ```
#[macro_export]
macro_rules! all_puzzles {
    ($callback:path $(,$arg:tt)*$(,)?) => {
        $crate::utils::puzzles_noop!{
            [
                // xtask update all_puzzles
                $crate::year2015::puzzles,
                $crate::year2016::puzzles,
                $crate::year2017::puzzles,

                $callback
            ]
            $($arg)*
        }
    };
}
