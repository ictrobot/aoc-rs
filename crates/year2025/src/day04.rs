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
            |_, _| Err("expected '.' or '@'"),
        )?;

        Ok(Self { rows, cols, grid })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        // Avoids bounds checks, allowing the inner loop to be vectorized
        let mut total = 0;
        for ((above, row), below) in self
            .grid
            .chunks_exact(self.cols)
            .zip(self.grid.chunks_exact(self.cols).skip(1))
            .zip(self.grid.chunks_exact(self.cols).skip(2))
        {
            for i in 1..self.cols - 1 {
                let neighbours = u8::from(above[i - 1])
                    + u8::from(above[i])
                    + u8::from(above[i + 1])
                    + u8::from(row[i - 1])
                    + u8::from(row[i + 1])
                    + u8::from(below[i - 1])
                    + u8::from(below[i])
                    + u8::from(below[i + 1]);
                total += u32::from(row[i] & (neighbours < 4));
            }
        }
        total
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut total = 0;
        let mut grid = self.grid.clone();

        // Store which rows need to be recomputed, reducing the number of row updates by >50%
        let mut rows_to_update = vec![true; self.rows];

        // Each cell can be updated in-place as the update rule only ever removes rolls and the
        // order they are removed does not matter.
        // However, doing this prevents vectorization, as each inner loop iteration now depends on
        // the previous iteration.
        // Instead, write the new row to a separate buffer to allow vectorization, then copy it back
        // before processing the following row.
        // This still allows more removals to be done per grid iteration compared to writing to a
        // second grid.
        let mut new_row = vec![false; self.cols];

        loop {
            let prev_total = total;

            for r in 1..self.rows - 1 {
                if !rows_to_update[r] {
                    continue;
                }

                let above = &grid[(r - 1) * self.cols..r * self.cols];
                let row = &grid[r * self.cols..(r + 1) * self.cols];
                let below = &grid[(r + 1) * self.cols..(r + 2) * self.cols];

                let before_row_total = total;
                for i in 1..self.cols - 1 {
                    let neighbours = u8::from(above[i - 1])
                        + u8::from(above[i])
                        + u8::from(above[i + 1])
                        + u8::from(row[i - 1])
                        + u8::from(row[i + 1])
                        + u8::from(below[i - 1])
                        + u8::from(below[i])
                        + u8::from(below[i + 1]);
                    let remove = row[i] & (neighbours < 4);
                    total += u32::from(remove);
                    new_row[i] = row[i] & !remove;
                }

                grid[r * self.cols..(r + 1) * self.cols].copy_from_slice(&new_row);

                if before_row_total != total {
                    // Row has been updated, need to update it again as well as its neighbours
                    rows_to_update[r - 1] = true;
                    rows_to_update[r + 1] = true;
                } else {
                    rows_to_update[r] = false;
                }
            }

            if total == prev_total {
                return total;
            }
        }
    }
}

examples!(Day04 -> (u32, u32) [
    {file: "day04_example0.txt", part1: 13, part2: 43},
]);
