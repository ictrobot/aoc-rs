use utils::grid::parse;
use utils::prelude::*;

/// Counting splitting paths in a grid.
#[derive(Clone, Debug)]
pub struct Day07 {
    part1: u64,
    part2: u64,
}

impl Day07 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut start = None;
        let (_rows, cols, grid) = parse(
            input,
            1,
            b'.',
            |b| b,
            |b| matches!(b, b'.' | b'^'),
            |i, b| {
                match b {
                    b'S' if start.is_none() => start = Some(i),
                    b'S' => return Err("expected one 'S'"),
                    _ => return Err("expected '.', '^' or 'S'"),
                }
                Ok(b'.')
            },
        )?;
        let Some(start) = start else {
            return Err(InputError::new(input, 0, "expected one 'S'"));
        };

        let mut timeline_counts = vec![0u64; cols];
        timeline_counts[start % cols] = 1;

        let mut split_count = 0;

        for row in grid.chunks_exact(cols).skip(start.div_ceil(cols)) {
            for c in 1..cols - 1 {
                if timeline_counts[c] > 0 && row[c] == b'^' {
                    timeline_counts[c - 1] += timeline_counts[c];
                    timeline_counts[c + 1] += timeline_counts[c];
                    timeline_counts[c] = 0;
                    split_count += 1;
                }
            }
        }

        Ok(Self {
            part1: split_count,
            part2: timeline_counts.iter().sum(),
        })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        self.part2
    }
}

examples!(Day07 -> (u64, u64) [
    {file: "day07_example0.txt", part1: 21, part2: 40},
]);
