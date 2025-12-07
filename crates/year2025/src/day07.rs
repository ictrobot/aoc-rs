use utils::prelude::*;

/// Counting splitting paths in a grid.
#[derive(Clone, Debug)]
pub struct Day07 {
    part1: u64,
    part2: u64,
}

impl Day07 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut seen_start = false;
        let mut timeline_counts = Vec::new();
        let mut split_count = 0;

        for line in input.lines() {
            if timeline_counts.is_empty() {
                // Add 1 padding column on each side so c +- 1 below is always safe
                timeline_counts.resize(line.len() + 2, 0);
            } else if timeline_counts.len() != line.len() + 2 {
                return Err(InputError::new(
                    input,
                    line,
                    "expected consistent line lengths",
                ));
            }

            for (i, b) in line.bytes().enumerate() {
                let c = i + 1;

                match b {
                    b'^' if timeline_counts[c] > 0 => {
                        timeline_counts[c - 1] += timeline_counts[c];
                        timeline_counts[c + 1] += timeline_counts[c];
                        timeline_counts[c] = 0;
                        split_count += 1;
                    }
                    b'.' | b'^' => {}
                    b'S' if !seen_start => {
                        timeline_counts[c] += 1;
                        seen_start = true;
                    }
                    b'S' => return Err(InputError::new(input, line, "expected one 'S'")),
                    _ => return Err(InputError::new(input, line, "expected '.', '^' or 'S'")),
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
