use utils::prelude::*;

/// Divisor function.
#[derive(Clone, Debug)]
pub struct Day20 {
    part1: u32,
    part2: u32,
}

impl Day20 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let threshold: u32 = input
            .trim_ascii_end()
            .parse()
            .map_err(|_| InputError::new(input, 0, "expected u32"))?;

        // (threshold/10) + 1 should always find the solutions, but attempt to find the solutions
        // with smaller & significantly faster upper bounds first
        for max_bound in [
            threshold.div_ceil(40) + 1,
            threshold.div_ceil(20) + 1,
            threshold.div_ceil(10) + 1,
        ] {
            if let Some((part1, part2)) = Self::try_solve(threshold, max_bound) {
                return Ok(Day20 { part1, part2 });
            }
        }

        unreachable!();
    }

    fn try_solve(threshold: u32, len: u32) -> Option<(u32, u32)> {
        let mut houses = vec![0; len as usize];

        // Elves < len/50 would visit more than 50 houses, visit the first 50 for part 2 solution
        for elf in 1..len / 50 {
            for house in (elf..len).step_by(elf as usize).take(50) {
                houses[house as usize] += elf;
            }
        }

        // Elves > len/50 will visit at most 50 houses
        for elf in (len / 50).max(1)..len / 2 {
            for house in (elf..len).step_by(elf as usize) {
                houses[house as usize] += elf;
            }
        }

        // Elves >= len/2 will visit 1 house
        for elf in (len / 2).max(1)..len {
            houses[elf as usize] += elf;
        }

        let part2_threshold = threshold.div_ceil(11);
        let part2 = houses.iter().position(|&c| c >= part2_threshold)?;

        // After finding part 2 solution, process the previously skipped houses for elves < len/50
        for elf in 1..len / 50 {
            for house in (elf * 51..len).step_by(elf as usize) {
                houses[house as usize] += elf;
            }
        }

        let part1_threshold = threshold.div_ceil(10);
        let part1 = houses.iter().position(|&c| c >= part1_threshold)?;

        Some((part1 as u32, part2 as u32))
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

examples!(Day20 -> (usize, usize) []);
