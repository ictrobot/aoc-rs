use utils::prelude::*;

/// Finding cycles.
///
/// See <https://en.wikipedia.org/wiki/Cycle_detection#Brent's_algorithm>, which avoids storing and
/// hashing every visited state at the expense of calculating extra iterations.
#[derive(Clone, Debug)]
pub struct Day06 {
    part1: u32,
    part2: u32,
}

impl Day06 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let banks = parser::u32()
            .with_suffix(b' '.or(b'\t').optional())
            .parse_all(input)?;

        let (mut power, mut lambda) = (1, 1);
        let mut tortoise = banks.clone();
        let mut hare = banks.clone();
        Self::next(&mut hare);

        while tortoise != hare {
            if power == lambda {
                tortoise.copy_from_slice(hare.as_slice());
                power *= 2;
                lambda = 0;
            }
            Self::next(&mut hare);
            lambda += 1;
        }

        tortoise.copy_from_slice(banks.as_slice());
        hare.copy_from_slice(banks.as_slice());
        for _ in 0..lambda {
            Self::next(&mut hare);
        }

        let mut mu = 0;
        while tortoise != hare {
            Self::next(&mut tortoise);
            Self::next(&mut hare);
            mu += 1;
        }

        Ok(Self {
            part1: mu + lambda,
            part2: lambda,
        })
    }

    fn next(banks: &mut [u32]) {
        let (mut idx, mut remaining) = banks
            .iter()
            .copied()
            .enumerate()
            .rev()
            .max_by_key(|&(_, v)| v)
            .unwrap();

        banks[idx] = 0;

        while remaining > 0 {
            idx = (idx + 1) % banks.len();
            banks[idx] += 1;
            remaining -= 1;
        }
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.part2
    }
}

examples!(Day06 -> (u32, u32) [
    {input: "0\t2\t7\t0", part1: 5, part2: 4},
]);
