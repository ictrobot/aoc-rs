use utils::prelude::*;

/// Simulating a pseudorandom number generator.
#[derive(Clone, Debug)]
pub struct Day22 {
    input: Vec<u32>,
}

impl Day22 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            input: parser::number_range(0..=0xFFFFFF).parse_lines(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        let mut numbers = self.input.clone();
        for _ in 0..2000 {
            // Inner loop over numbers allows for vectorization
            for n in &mut numbers {
                *n = Self::next(*n);
            }
        }
        numbers.iter().map(|&n| n as u64).sum()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut bananas = [0; 130321]; // 19 ** 4
        let mut seen = [0; 130321];
        for (i, &(mut n)) in self.input.iter().enumerate() {
            let mut index = 0;
            let mut last_digit = n % 10;
            for j in 0..2000 {
                let next = Self::next(n);
                let digit = next % 10;

                index = (9 + digit - last_digit) as usize + ((index % 6859) * 19);
                if j >= 3 && seen[index] < i + 1 {
                    bananas[index] += next % 10;
                    seen[index] = i + 1;
                }

                n = next;
                last_digit = digit;
            }
        }
        bananas.iter().max().copied().unwrap()
    }

    #[inline(always)]
    fn next(mut n: u32) -> u32 {
        n = (n ^ (n << 6)) % 0x1000000;
        n = (n ^ (n >> 5)) % 0x1000000;
        n = (n ^ (n << 11)) % 0x1000000;
        n
    }
}

examples!(Day22 -> (u64, u32) [
    {input: "1\n10\n100\n2024", part1: 37327623},
    {input: "1\n2\n3\n2024", part2: 23},
]);
