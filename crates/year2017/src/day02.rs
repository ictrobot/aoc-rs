use utils::prelude::*;

/// Calculating spreadsheet checksums.
#[derive(Clone, Debug)]
pub struct Day02 {
    rows: Vec<Vec<u32>>,
}

impl Day02 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            rows: parser::u32()
                .repeat(b' '.or(b'\t'), 2)
                .map(|mut v| {
                    v.sort_unstable();
                    v
                })
                .parse_lines(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.rows
            .iter()
            .map(|row| row[row.len() - 1] - row[0])
            .sum()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.rows
            .iter()
            .map(|row| {
                for i in 0..row.len() - 1 {
                    for j in i + 1..row.len() {
                        if row[j].is_multiple_of(row[i]) {
                            return row[j] / row[i];
                        }
                    }
                }
                panic!("no solution found")
            })
            .sum()
    }
}

examples!(Day02 -> (u32, u32) [
    {input: "5 1 9 5\n7 5 3\n2 4 6 8", part1: 18},
    {input: "5 9 2 8\n9 4 7 3\n3 8 6 5", part2: 9},
]);
