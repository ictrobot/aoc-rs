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
    pub fn part2(&self) -> u16 {
        let mut bananas = [0; 130321]; // 19 ** 4
        let mut seen = [0; 130321];
        for (i, &(mut n)) in self.input.iter().enumerate() {
            let mut prev = n % 10;
            let mut s4;

            n = Self::next(n);
            let mut s3 = ((9 + n % 10 - prev) as usize) * 19 * 19;
            prev = n % 10;

            n = Self::next(n);
            let mut s2 = ((9 + n % 10 - prev) as usize) * 19;
            prev = n % 10;

            n = Self::next(n);
            let mut s1 = (9 + n % 10 - prev) as usize;
            prev = n % 10;

            for _ in 3..2000 {
                n = Self::next(n);
                let digit = n % 10;
                (s1, s2, s3, s4) = ((9 + digit - prev) as usize, 19 * s1, 19 * s2, 19 * s3);

                let index = s4 + s3 + s2 + s1;
                if seen[index] != (i + 1) as u16 {
                    bananas[index] += digit as u16;
                    seen[index] = (i + 1) as u16;
                }

                prev = digit;
            }
        }
        bananas.iter().max().copied().unwrap()
    }

    #[inline(always)]
    fn next(mut n: u32) -> u32 {
        n = (n ^ (n << 6)) & 0xFFFFFF;
        n = (n ^ (n >> 5)) & 0xFFFFFF;
        (n ^ (n << 11)) & 0xFFFFFF
    }
}

examples!(Day22 -> (u64, u16) [
    {input: "1\n10\n100\n2024", part1: 37327623},
    {input: "1\n2\n3\n2024", part2: 23},
]);
