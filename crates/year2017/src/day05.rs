use utils::prelude::*;

/// Counting steps through a maze of jump instructions.
#[derive(Clone, Debug)]
pub struct Day05 {
    jumps: Vec<i32>,
}

impl Day05 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            jumps: parser::i32().parse_lines(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        let mut jumps = self.jumps.clone();
        let mut steps = 0;
        let mut pc = 0;

        while pc < jumps.len() {
            let offset = jumps[pc];
            jumps[pc] += 1;
            pc = pc.wrapping_add_signed(offset as isize);
            steps += 1;
        }

        steps
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        let mut jumps = self.jumps.clone();
        let mut steps = 0;
        let mut pc = 0;

        while pc < jumps.len() {
            let offset = jumps[pc];
            jumps[pc] += if offset >= 3 { -1 } else { 1 };
            pc = pc.wrapping_add_signed(offset as isize);
            steps += 1;
        }

        steps
    }
}

examples!(Day05 -> (u64, u64) [
    {input: "0\n3\n0\n1\n-3", part1: 5, part2: 10},
]);
