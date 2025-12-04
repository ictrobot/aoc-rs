use utils::grid;
use utils::prelude::*;

/// Iteratively removing cells with less than 4 neighbours.
#[derive(Clone, Debug)]
pub struct Day04 {
    rows: usize,
    cols: usize,
    grid: Vec<bool>,
}

impl Day04 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (rows, cols, grid) = grid::parse(
            input,
            1,
            false,
            |c| c == b'@',
            |c| c == b'@' || c == b'.',
            |_, _| Err("Expected '.' or '@'"),
        )?;

        Ok(Self { rows, cols, grid })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        let mut total = 0;
        for r in 1..self.rows - 1 {
            for c in 1..self.cols - 1 {
                let index = r * self.cols + c;
                if !self.grid[index] {
                    continue;
                }

                let count = u64::from(self.grid[index - self.cols - 1])
                    + u64::from(self.grid[index - self.cols])
                    + u64::from(self.grid[index - self.cols + 1])
                    + u64::from(self.grid[index - 1])
                    + u64::from(self.grid[index + 1])
                    + u64::from(self.grid[index + self.cols - 1])
                    + u64::from(self.grid[index + self.cols])
                    + u64::from(self.grid[index + self.cols + 1]);

                if count < 4 {
                    total += 1;
                }
            }
        }
        total
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        let mut grid = self.grid.clone();
        let mut new_grid = self.grid.clone();
        let mut total = 0;
        loop {
            for r in 1..self.rows - 1 {
                for c in 1..self.cols - 1 {
                    let index = r * self.cols + c;
                    if !grid[index] {
                        continue;
                    }

                    let count = u64::from(grid[index - self.cols - 1])
                        + u64::from(grid[index - self.cols])
                        + u64::from(grid[index - self.cols + 1])
                        + u64::from(grid[index - 1])
                        + u64::from(grid[index + 1])
                        + u64::from(grid[index + self.cols - 1])
                        + u64::from(grid[index + self.cols])
                        + u64::from(grid[index + self.cols + 1]);

                    if count < 4 {
                        total += 1;
                        new_grid[index] = false;
                    }
                }
            }

            if new_grid == grid {
                return total;
            }
            grid = new_grid.clone();
        }
    }
}

examples!(Day04 -> (u64, u64) [
    {file: "day04_example0.txt", part1: 13, part2: 43},
]);
