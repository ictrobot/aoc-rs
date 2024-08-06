use utils::prelude::*;

/// Divisor function.
#[derive(Clone, Debug)]
pub struct Day20 {
    threshold: u32,
}

impl Day20 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            threshold: input
                .trim_ascii_end()
                .parse()
                .map_err(|_| InputError::new(input, 0, "expected u32"))?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        let threshold = self.threshold / 10;
        let mut buckets = vec![0; threshold as usize];

        for elf in 1..threshold {
            for i in (elf..threshold).step_by(elf as usize) {
                buckets[i as usize] += elf;
            }
        }

        buckets.iter().position(|&x| x >= threshold).unwrap_or(0)
    }

    #[must_use]
    pub fn part2(&self) -> usize {
        let threshold = self.threshold / 11;
        let mut buckets = vec![0u32; threshold as usize];

        for elf in 1..threshold {
            for i in (elf..threshold).step_by(elf as usize).take(50) {
                buckets[i as usize] += elf;
            }
        }

        buckets.iter().position(|&x| x >= threshold).unwrap_or(0)
    }
}

examples!(Day20 -> (usize, usize) []);
