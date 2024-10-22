use utils::grid::from_str_padded;
use utils::prelude::*;

/// Game of Life.
#[derive(Clone, Debug)]
pub struct Day18 {
    data: Vec<bool>,
    size: usize,
    part1_steps: u32,
    part2_steps: u32,
}

impl Day18 {
    pub fn new(input: &str, input_type: InputType) -> Result<Self, InputError> {
        let (rows, columns, data) = from_str_padded(input, 1, false, |c| match c {
            b'#' => Some(true),
            b'.' => Some(false),
            _ => None,
        })?;

        if rows != columns {
            return Err(InputError::new(input, input, "expected square grid"));
        }

        let (part1_steps, part2_steps) = match input_type {
            InputType::Example => (4, 5),
            InputType::Real => (100, 100),
        };

        Ok(Self {
            size: rows,
            data,
            part1_steps,
            part2_steps,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.count_lights(self.part1_steps, |_| {})
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let top_left = self.size + 1;
        let top_right = self.size + (self.size - 2);
        let bottom_left = (self.size - 2) * self.size + 1;
        let bottom_right = (self.size - 2) * self.size + (self.size - 2);

        self.count_lights(self.part2_steps, |grid| {
            grid[top_left] = true;
            grid[top_right] = true;
            grid[bottom_left] = true;
            grid[bottom_right] = true;
        })
    }

    fn count_lights(&self, steps: u32, callback: impl Fn(&mut [bool])) -> u32 {
        let mut grid = self.data.clone();
        let mut grid2 = vec![false; grid.len()];

        callback(&mut grid);

        for _ in 0..steps {
            self.advance(&grid, &mut grid2);
            callback(&mut grid2);
            (grid, grid2) = (grid2, grid);
        }

        grid.iter().copied().filter(|&x| x).count() as u32
    }

    fn advance(&self, input: &[bool], output: &mut [bool]) {
        // Avoids bounds checks, allowing the inner loop to be vectorized
        for (((above, row), below), out) in input
            .chunks_exact(self.size)
            .zip(input.chunks_exact(self.size).skip(1))
            .zip(input.chunks_exact(self.size).skip(2))
            .zip(output.chunks_exact_mut(self.size).skip(1))
        {
            for i in 1..self.size - 1 {
                let neighbours = u8::from(above[i - 1])
                    + u8::from(above[i])
                    + u8::from(above[i + 1])
                    + u8::from(row[i - 1])
                    + u8::from(row[i + 1])
                    + u8::from(below[i - 1])
                    + u8::from(below[i])
                    + u8::from(below[i + 1]);
                out[i] = (neighbours | u8::from(row[i])) == 3;
            }
        }
    }
}

examples!(Day18 -> (u32, u32) [
    {input: ".#.#.#\n...##.\n#....#\n..#...\n#.#..#\n####..", part1: 4, part2: 17},
]);
