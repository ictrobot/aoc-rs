use utils::prelude::*;

/// Finding matching keys and locks.
#[derive(Clone, Debug)]
pub struct Day25 {
    locks: Vec<u32>,
    keys: Vec<u32>,
}

impl Day25 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut locks = Vec::new();
        let mut keys = Vec::new();

        for item in parser::one_of((b'.'.map(|_| false), b'#'.map(|_| true)))
            .repeat_n::<5, _>(parser::noop())
            .repeat_n::<7, _>(parser::eol())
            .with_suffix(parser::eol().then(parser::eol()))
            .parse_iterator(input)
        {
            let grid = item?;

            let top = grid[0][0];
            let bottom = grid[6][0];
            if top == bottom
                || grid[0][1..].iter().any(|&x| x != top)
                || grid[6][1..].iter().any(|&x| x != bottom)
            {
                return Err(InputError::new(input, 0, "expected lock or key"));
            }

            let mut mask = 0u32;
            for c in 0..5 {
                let mut h = 0;
                for row in grid[1..].iter() {
                    if row[c] != top {
                        break;
                    }
                    h += 1;
                }
                mask |= ((1 << h) - 1) << (c * 5);
            }

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
