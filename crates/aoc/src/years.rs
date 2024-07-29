//! Implementation for [`all_puzzles!`](crate::all_puzzles!).
//!
//! Each of the re-exported macros below is either the `puzzles!` macro from the corresponding
//! crate if the corresponding feature is enabled, or the [`utils::puzzles_noop!`] no-op macro if
//! it is not. These macros are then chained together by [`all_puzzles!`](crate::all_puzzles!).
//!
//! This main advantage of this approach over passing `#[cfg(feature = ?)]` attributes to the
//! callback is that it prevents disabled years from being expanded at all, which should speed up
//! compilation.
//!
//! Running `cargo xtask update` will automatically update the list of re-exports.

// xtask update re-exports
#[cfg(not(feature = "year2015"))]
pub use ::utils::puzzles_noop as year2015;
#[cfg(not(feature = "year2016"))]
pub use ::utils::puzzles_noop as year2016;
#[cfg(feature = "year2015")]
pub use ::year2015::puzzles as year2015;
#[cfg(feature = "year2016")]
pub use ::year2016::puzzles as year2016;

/// Macro which invokes a callback macro with a list of all implemented puzzle solutions.
///
/// This macro chains re-exported `puzzles!` macros in [`aoc::years`](self), provided by each year
/// crate.
///
/// The callback macro will be called once, with all solutions, which allows generation of complete
/// match statements. The paths can be matched with `$([$(::$p:ident)+])*`. `$([$p:path])*` can also
/// be used, but it has [limitations](https://github.com/rust-lang/rust/issues/48067) making it far
/// less useful.
///
/// Running `cargo xtask update` will automatically update the chain of year macros.
///
/// # Examples
///
/// Simple `main` function to run all examples:
///
/// ```
/// # use aoc::all_puzzles;
/// # use utils::{PuzzleExamples, Puzzle};
/// # use utils::input::InputType;
/// #
/// macro_rules! callback {
///     ($([$(::$p:ident)+])*) => {$(
///         println!("{} {}", $(::$p)+::YEAR, $(::$p)+::DAY);///
///         for (input_str, p1, p2) in $(::$p)+::EXAMPLES.iter() {
///             let solution = $(::$p)+::new(input_str, InputType::Example).unwrap();
///             println!("  parse({input_str}) = {solution:?}");
///             if (p1.is_some()) { println!("  part1(...) = {}", solution.part1()); }
///             if (p2.is_some()) { println!("  part2(...) = {}", solution.part2()); }
///         }
///     )*};
/// }
/// all_puzzles!(callback);
/// ```
///
/// Generating a match statement and passing extra arguments:
///
/// ```
/// # use aoc::all_puzzles;
/// # use utils::date::{Day, Year};
/// # use utils::{PuzzleExamples, Puzzle};
/// #
/// fn example_count(year: Year, day: Day) -> Option<usize> {
///     macro_rules! callback {
///         ($year:ident $day:ident $([$(::$p:ident)+])*) => {
///             match ($year, $day) {
///                 $(
///                     ($(::$p)+::YEAR, $(::$p)+::DAY) => Some($(::$p)+::EXAMPLES.len()),
///                 )*
///                 _ => None,
///             }
///         };
///     }
///     all_puzzles!{callback, year, day}
/// }
/// ```
#[allow(clippy::module_name_repetitions)] // Once exported name is aoc::all_puzzles
#[macro_export]
macro_rules! all_puzzles {
    ($callback:path $(,$arg:tt)*$(,)?) => {
        ::utils::puzzles_noop!{
            [
                // xtask update all_puzzles
                $crate::years::year2015,
                $crate::years::year2016,

                $callback
            ]
            $($arg)*
        }
    };
}
