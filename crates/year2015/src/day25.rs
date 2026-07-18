use std::num::NonZeroU64;
use utils::number::mod_pow;
use utils::prelude::*;

/// Modular exponentiation.
#[derive(Clone, Debug)]
pub struct Day25 {
    row: NonZeroU64,
    column: NonZeroU64,
}

impl Day25 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (row, column) = parser::nonzero_u64()
            .with_prefix(
                "To continue, please consult the code grid in the manual.  Enter the code at row ",
            )
            .then(
                parser::nonzero_u64()
                    .with_prefix(", column ")
                    .with_suffix("."),
            )
            .parse_complete(input)?;
        Ok(Self { row, column })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        let (row, column) = (self.row.get(), self.column.get());
        let triangle = (row + column - 2) * (row + column - 1) / 2;
        let index = triangle + column - 1;

        (20151125 * mod_pow(252533, index, 33554393)) % 33554393
    }

    #[must_use]
    pub fn part2(&self) -> &'static str {
        "🎄"
    }
}

examples!(Day25 -> (u64, &'static str) [
    {
        input: "To continue, please consult the code grid in the manual.  Enter the code at row 1, column 1.",
        part1: 20151125,
    },
    {
        input: "To continue, please consult the code grid in the manual.  Enter the code at row 3, column 4.",
        part1: 7981243,
    },
    {
        input: "To continue, please consult the code grid in the manual.  Enter the code at row 6, column 3.",
        part1: 25397450,
    },
    {
        input: "To continue, please consult the code grid in the manual.  Enter the code at row 6, column 6.",
        part1: 27995004,
    },
]);
