use utils::prelude::*;

/// Finding matching keys and locks.
#[derive(Clone, Debug)]
pub struct Day25 {
    locks: Vec<u32>,
    keys: Vec<u32>,
}

impl Day25 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut locks = Vec::with_capacity(250);
        let mut keys = Vec::with_capacity(250);

        for item in parser::byte_map!(b'#' => true, b'.' => false)
            .repeat_n::<5, _>(parser::noop())
            .repeat_n::<7, _>(parser::eol())
            .with_consumed()
            .with_eol()
            .with_eol()
            .parse_iterator(input)
        {
            let (grid, grid_str) = item?;

            let top = grid[0][0];
            let bottom = grid[6][0];
            if top == bottom
                || grid[0][1..].iter().any(|&x| x != top)
                || grid[6][1..].iter().any(|&x| x != bottom)
            {
                return Err(InputError::new(input, grid_str, "expected lock or key"));
            }

            let mask = grid[1..6]
                .as_flattened()
                .iter()
                .enumerate()
                .fold(0, |acc, (i, &x)| acc | (u32::from(x == top) << i));

            if top {
                locks.push(mask);
            } else {
                keys.push(mask);
            }
        }

        Ok(Self { locks, keys })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let mut total = 0;
        for &lock in &self.locks {
            for &key in &self.keys {
                if lock & key == lock {
                    total += 1;
                }
            }
        }
        total
    }

    #[must_use]
    pub fn part2(&self) -> &'static str {
        "🎄"
    }
}

examples!(Day25 -> (u32, &'static str) [
    {file: "day25_example0.txt", part1: 3},
]);
