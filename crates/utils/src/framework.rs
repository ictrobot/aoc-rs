use crate::date::{Day, Year};
use std::fmt::{Debug, Display};
use std::fs::read_to_string;
use std::io;
use std::path::PathBuf;

/// Common trait implemented by puzzles to provide [`Year`], [`Day`] and read input.
///
/// [`year!`](crate::year!) implements this automatically.
pub trait Puzzle {
    const YEAR: Year;
    const DAY: Day;

    fn read_input() -> io::Result<String> {
        fn read(year: Year, day: Day) -> io::Result<String> {
            let mut path = PathBuf::new();
            path.push("inputs");
            path.push(format!("year{year:#}"));
            path.push(format!("day{day:#}.txt"));
            read_to_string(path).map(|s| s.trim_ascii_end().replace("\r\n", "\n"))
        }
        read(Self::YEAR, Self::DAY)
    }
}

/// Trait implemented by puzzles to provide example inputs and answers.
///
/// [`examples!`](crate::examples!) implements this automatically.
pub trait PuzzleExamples<P1: Debug + Display + 'static, P2: Debug + Display + 'static> {
    const EXAMPLES: &'static [(&'static str, Option<P1>, Option<P2>)];
}

/// Macro to generate the crate root for each year crate, implementing common items.
///
/// For each day, the module is declared, the struct re-exported and the [`Puzzle`] trait
/// implemented.
///
/// A `puzzle!` macro is defined and exported, which takes one or more callback macro paths and a
/// list of arguments captured as `tt` fragments. The macro expands to calling the first callback
/// with the remaining callback paths, the provided arguments and paths to all the day
/// structs defined in this crate. These macros are then chained across all year crates to implement
/// [`aoc::all_puzzles!`](../aoc/macro.all_puzzles.html).
///
/// Running `cargo xtask update` will automatically update the list of days inside macro invocations
/// in files matching `crates/year????/src/lib.rs`.
///
/// # Examples
///
/// ```ignore
/// utils::year!(2015 => year2015, ${
///     1 => day01::Day01,
///     2 => day02::Day02,
/// });
/// ```
#[macro_export]
macro_rules! year {
    ($year:literal => $crate_name:ident, $dollar:tt{$(
        $day:literal => $day_mod:ident::$day_struct:ident$(<$lifetime:lifetime>)?,
    )+}) => {
        $(
            mod $day_mod;
            #[doc = concat!("[", $year, " Day ", $day, "](https://adventofcode.com/", $year, "/day/", $day, "):")]
            pub use $day_mod::$day_struct;
            impl $crate::Puzzle for $day_struct$(<$lifetime>)? {
                #[doc = concat!("Year ", $year)]
                const YEAR: $crate::date::Year = $crate::date::Year::new_const::<$year>();
                #[doc = concat!("Day ", $day)]
                const DAY: $crate::date::Day = $crate::date::Day::new_const::<$day>();
            }
        )+

        /// Macro which supplies a list of implemented puzzle solutions in this crate.
        ///
        /// Automatically generated by [utils::year!]. Refer to its documentation for more details.
        #[macro_export]
        macro_rules! puzzles {
            (
                [$dollar callback:path $dollar(,$dollar($dollar callbacks:path),+)?]
                $dollar ($dollar args:tt)*
            ) => {
                $dollar callback!{
                    $dollar([$dollar($dollar callbacks),+])?
                    $dollar($dollar args)*
                    $([::$crate_name::$day_struct])+
                }
            }
        }
    };
}

/// Version of the `puzzles!` macro generated by [`year!`] which appends no extra arguments.
#[macro_export]
macro_rules! puzzles_noop {
    ([$callback:path $(,$($callbacks:path),+)?] $($args:tt)*) => {
        $callback!{
            $([$($callbacks),+])?
            $($args)*
        }
    };
}

/// Macro to generate a list of examples, implement [`PuzzleExamples`] and add example tests.
///
/// The provided types for `part1` and `part2` don't have to match the types returned by the day's
/// functions, but they must be comparable with [`PartialEq`]. For functions returning [`String`]
/// `&'static str` should be used.
///
/// If no examples are provided, tests aren't generated.
///
/// # Examples
///
/// Adding examples to a `Day01` puzzle where `part1` returns [`u32`] and `part2` returns [`u64`].
/// The first example has correct answers defined for both parts. The second and third examples
/// are only applicable to `part1` and `part2` of the puzzle respectively.
///
/// ```ignore
/// examples!(Day01 -> (u32, u64) [
///     {input: "ABCDEF", part1: 30, part2: 342},
///     {input: "AAAAAA", part1: 21},
///     {input: "ABC123", part2: 853},
/// ]);
/// ```
///
/// Example inputs can also be included from the crate's examples directory by using `file` instead
/// of `input`:
///
/// ```ignore
/// examples!(Day01 -> (u32, u64) [
///     {input: "Short example", part1: 27},
///     {file: "day01_example.txt", part2: 483},
/// ]);
/// ```
#[macro_export]
macro_rules! examples {
    ($day:ident$(<$lifetime:lifetime>)? -> ($p1:ty, $p2:ty) [$($($tail:tt,)+)?]) => {
        impl $crate::PuzzleExamples<$p1, $p2> for $day$(<$lifetime>)? {
            const EXAMPLES: &'static [(&'static str, Option<$p1>, Option<$p2>)] = &[$($(
                $crate::examples!(@item $tail)
            ),+)?];
        }

        $(
        #[cfg(test)]
        mod example_tests {
            use $crate::{PuzzleExamples, input::InputType};
            use super::$day;
            $crate::examples!(@ignore $($tail)+);

            #[test]
            fn new() {
                for (i, example) in $day::EXAMPLES.iter().enumerate() {
                    let solution = $day::new(example.0, InputType::Example);
                    assert!(
                        solution.is_ok(),
                        "new failed for example {i}: {:?}",
                        example.0,
                    );
                }
            }

            #[test]
            fn part1() {
                for (i, example) in $day::EXAMPLES.iter().enumerate() {
                    if let Some(expected) = example.1 {
                        let solution = $day::new(example.0, InputType::Example).unwrap();
                        assert_eq!(
                            solution.part1(),
                            expected,
                            "part 1 incorrect for example {i}: {:?}",
                            example.0,
                        );
                    }
                }
            }

            #[test]
            fn part2() {
                for (i, example) in $day::EXAMPLES.iter().enumerate() {
                    if let Some(expected) = example.2 {
                        let solution = $day::new(example.0, InputType::Example).unwrap();
                        assert_eq!(
                            solution.part2(),
                            expected,
                            "part 2 incorrect for example {i}: {:?}",
                            example.0,
                        );
                    }
                }
            }
        }
        )?
    };

    (@item {input: $str:literal, part1: $p1:literal, part2: $p2:expr $(,)?}) => {
        ($str, Some($p1), Some($p2))
    };
    (@item {input: $str:literal, part1: $p1:literal $(,)?}) => {
        ($str, Some($p1), None)
    };
    (@item {input: $str:literal, part2: $p2:expr $(,)?}) => {
        ($str, None, Some($p2))
    };
    (@item {file: $file:literal, part1: $p1:literal, part2: $p2:expr $(,)?}) => {
        (
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/", $file)).trim_ascii_end(),
            Some($p1),
            Some($p2),
        )
    };
    (@item {file: $file:literal, part1: $p1:literal $(,)?}) => {
        (
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/", $file)).trim_ascii_end(),
            Some($p1),
            None,
        )
    };
    (@item {file: $file:literal, part2: $p2:expr $(,)?}) => {
        (
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/", $file)).trim_ascii_end(),
            None,
            Some($p2),
        )
    };
    (@ignore $($tail:tt)*) => {};
}
