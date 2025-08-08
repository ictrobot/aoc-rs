use crate::all_puzzles;
use utils::date::{Day, Year};
use utils::input::strip_final_newline;

// These imports are unused if none of the year features are enabled
#[allow(clippy::allow_attributes, unused_imports)]
use utils::{
    Puzzle,
    input::{InputError, InputType},
};

/// Represents a wrapper function around a puzzle solution.
///
/// See [`PUZZLES`].
pub type PuzzleFn = fn(&str) -> Result<(String, String), InputError>;

macro_rules! matcher {
    ($(
        $y:literal => $year:ident{$(
            $d:literal => $day:ident,
        )*}
    )*) => {
        /// Constant containing each puzzle solution.
        ///
        /// Each puzzle is represented by a tuple of [`Year`], [`Day`] and [`PuzzleFn`], which takes
        /// a input string and returns the part 1 and 2 solutions as strings, or an [`InputError`].
        ///
        /// Generated from [`all_puzzles!`].
        pub const PUZZLES: &[(Year, Day, PuzzleFn)] = &[$($(
            (crate::$year::$day::YEAR, crate::$year::$day::DAY, |input: &str| {
                let input = strip_final_newline(input);
                let solution = crate::$year::$day::new(input, InputType::Real)?;
                let part1 = solution.part1();
                let part2 = solution.part2();
                Ok((part1.to_string(), part2.to_string()))
            }),
        )*)*];
    };
}
all_puzzles!(matcher);
