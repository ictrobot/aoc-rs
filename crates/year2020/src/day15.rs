use utils::prelude::*;

/// Generating a sequence from previous occurrences.
#[derive(Clone, Debug)]
pub struct Day15 {
    starting: Vec<u32>,
}

const DENSE_THRESHOLD: u32 = 65_536;

impl Day15 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            starting: parser::number_range(0..=2019)
                .repeat(b',', 1)
                .parse_complete(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.number_at(2020)
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.number_at(30_000_000)
    }

    #[inline]
    fn number_at(&self, target: u32) -> u32 {
        if target as usize <= self.starting.len() {
            return self.starting[target as usize - 1];
        }

        let mut last_turn = vec![0u32; target as usize];
        let mut seen = vec![0u64; target.div_ceil(64) as usize];
        for (turn, &number) in self.starting[..self.starting.len() - 1].iter().enumerate() {
            last_turn[number as usize] = turn as u32 + 1;
            seen[number as usize / 64] |= 1 << (number % 64);
        }

        let mut number = *self.starting.last().unwrap();
        for turn in self.starting.len() as u32..target {
            let previous = if number < DENSE_THRESHOLD {
                std::mem::replace(&mut last_turn[number as usize], turn)
            } else {
                // Track seen numbers in a bitset to reduce random timestamp reads
                let base = number as usize / 64;
                let mask = 1 << (number % 64);
                if seen[base] & mask == 0 {
                    seen[base] |= mask;
                    last_turn[number as usize] = turn;
                    0
                } else {
                    std::mem::replace(&mut last_turn[number as usize], turn)
                }
            };
            number = if previous == 0 { 0 } else { turn - previous };
        }
        number
    }
}

examples!(Day15 -> (u32, u32) [
    {input: "0,3,6", part1: 436, part2: 175594},
    {input: "1,3,2", part1: 1, part2: 2578},
    {input: "2,1,3", part1: 10, part2: 3544142},
    {input: "1,2,3", part1: 27, part2: 261214},
    {input: "2,3,1", part1: 78, part2: 6895259},
    {input: "3,2,1", part1: 438, part2: 18},
    {input: "3,1,2", part1: 1836, part2: 362},
]);
