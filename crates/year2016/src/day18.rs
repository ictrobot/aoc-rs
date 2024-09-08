use utils::prelude::*;

/// Counting safe tiles.
#[derive(Clone, Debug)]
pub struct Day18 {
    input: u128,
    columns: u32,
    input_type: InputType,
}

impl Day18 {
    pub fn new(input: &str, input_type: InputType) -> Result<Self, InputError> {
        if input.len() > 128 {
            Err(InputError::new(input, input, "too many columns"))
        } else if let Some(index) = input.find(|c| c != '.' && c != '^') {
            Err(InputError::new(input, index, "expected '.' or '^'"))
        } else {
            Ok(Self {
                input: input
                    .bytes()
                    .fold(0, |acc, b| (acc << 1) | u128::from(b == b'^')),
                columns: input.len() as u32,
                input_type,
            })
        }
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.count_safe(match self.input_type {
            InputType::Example => 10,
            InputType::Real => 40,
        })
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.count_safe(400000)
    }

    #[inline]
    fn count_safe(&self, rows: u32) -> u32 {
        let mask = u128::MAX.wrapping_shr(128 - self.columns);

        let mut row = self.input;
        let mut traps = 0;
        for _ in 0..rows {
            traps += row.count_ones();

            // Tiles on the next row are traps if only one of the left and right tiles on the
            // previous row are traps.
            row = ((row << 1) ^ (row >> 1)) & mask;
        }

        (rows * self.columns) - traps
    }
}

examples!(Day18 -> (u32, u32) [
    {input: ".^^.^.^^^^", part1: 38},
]);
