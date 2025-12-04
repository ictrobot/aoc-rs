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
        let mut prev_total = 0;

        let mut grid = self.grid.clone();
        let mut new_grid = self.grid.clone();

        // Store which rows need to be recomputed, reducing the number of row updates by 50%
        let mut rows_to_update = vec![true; self.rows];
        let mut new_rows_to_update = vec![false; self.rows];

        loop {
            for (above_index, (((above, row), below), out)) in grid
                .chunks_exact(self.cols)
                .zip(grid.chunks_exact(self.cols).skip(1))
                .zip(grid.chunks_exact(self.cols).skip(2))
                .zip(new_grid.chunks_exact_mut(self.cols).skip(1))
                .enumerate()
            {
                if !rows_to_update[above_index + 1] {
                    continue;
                }
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
                    out[i] = row[i] & !remove;
                }

                if before_row_total != total {
                    new_rows_to_update[above_index] = true;
                    new_rows_to_update[above_index + 1] = true;
                    new_rows_to_update[above_index + 2] = true;
                }
            }

            if total == prev_total {
                return total;
            }

            grid.clone_from(&new_grid);
            rows_to_update.clone_from(&new_rows_to_update);
            new_rows_to_update.fill(false);
            prev_total = total;
        }
    }
}

examples!(Day04 -> (u32, u32) [
    {file: "day04_example0.txt", part1: 13, part2: 43},
]);
