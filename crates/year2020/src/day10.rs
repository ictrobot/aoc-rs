use utils::bit::BitIterator;
use utils::prelude::*;

/// Counting paths through a sorted list of numbers.
#[derive(Clone, Debug)]
pub struct Day10 {
    part1: u64,
    part2: u64,
}

const MAX: usize = 255;

impl Day10 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (mut count, mut adapters) = (0, [0u64; MAX.div_ceil(64)]);
        for entry in parser::number_range(1..=MAX)
            .with_eol()
            .parse_iterator(input)
        {
            let n = entry?;
            adapters[n / 64] |= 1 << (n % 64);
            count += 1;
        }

        if count == 0 {
            return Err(InputError::new(input, 0, "expected at least one adapter"));
        }
        if count != adapters.iter().map(|w| w.count_ones()).sum::<u32>() {
            return Err(InputError::new(input, 0, "duplicate adapter"));
        }

        let (mut ones, mut threes) = (0u64, 1u64);

        // ways[n] = arrangements ending n jolts below the previous adapter
        let (mut previous, mut ways) = (0, [1u64, 0, 0]);

        for (i, word) in adapters.into_iter().enumerate() {
            for (bit, _) in BitIterator::ones(word) {
                let jolts = i * 64 + bit as usize;

                match jolts - previous {
                    1 => {
                        ones += 1;
                        ways = [ways[0] + ways[1] + ways[2], ways[0], ways[1]];
                    }
                    2 => ways = [ways[0] + ways[1], 0, ways[0]],
                    3 => {
                        threes += 1;
                        ways = [ways[0], 0, 0];
                    }
                    _ => return Err(InputError::new(input, 0, "expected a valid adapter chain")),
                }
                previous = jolts;
            }
        }

        Ok(Self {
            part1: ones * threes,
            part2: ways[0],
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

examples!(Day10 -> (u64, u64) [
    {file: "day10_example0.txt", part1: 35, part2: 8},
    {file: "day10_example1.txt", part1: 220, part2: 19208},
]);
