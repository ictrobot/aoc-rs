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

        for item in parser::bitmask_n::<5, u32>(b".#")
            .repeat_n::<7, _>(parser::eol())
            .with_consumed()
            .with_eol()
            .with_eol()
            .parse_iterator(input)
        {
            let (rows, grid_str) = item?;

            let [top, .., bottom] = rows;
            if !matches!((top, bottom), (0b11111, 0) | (0, 0b11111)) {
                return Err(InputError::new(input, grid_str, "expected lock or key"));
            }

            let mask = rows[1..6]
                .iter()
                .enumerate()
                .fold(0, |acc, (i, &row)| acc | row << (i * 5));

            if top != 0 {
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
                if lock & key == 0 {
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
