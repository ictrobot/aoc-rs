use utils::prelude::*;

/// Counting steps through a maze of jump instructions.
#[derive(Clone, Debug)]
pub struct Day05 {
    jumps: Vec<i32>,
}

type Compressed = usize;
const BITS: usize = Compressed::BITS as usize;

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

        // After each jump instruction is run enough times the offset oscillates back and forth
        // between 2 and 3. Once this happens, represent the jump as a single bit in a compressed
        // bit mask, which allows processing multiple jumps at once without each one requiring a
        // random memory read.
        let mut threes: Vec<Compressed> = vec![0; jumps.len().next_multiple_of(BITS) / BITS];
        // boundary represents the point where all prior jumps have stabilized on oscillating
        // between 2 and 3
        let mut boundary = 0;

        while pc < jumps.len() {
            let offset = jumps[pc];
            jumps[pc] += if offset >= 3 { -1 } else { 1 };

            if pc == boundary && (jumps[pc] == 2 || jumps[pc] == 3) {
                // Next jump after the boundary stabilized on 2/3
                boundary += 1;

                let element_index = pc / BITS;
                let bit_index = pc % BITS;
                threes[element_index] |= ((jumps[pc] & 1) as Compressed) << bit_index;
            }

            pc = pc.wrapping_add_signed(offset as isize);
            steps += 1;

            while pc < boundary {
                // While inside the boundary loop over each compressed element and handle the jumps
                // in bulk
                let element_index = pc / BITS;
                let mut element = threes[element_index];

                let bit_index = pc % BITS;
                let mut bit = 1 << bit_index;

                let max = boundary.min((element_index + 1) * BITS);
                while pc < max {
                    let offset = 2 + usize::from(element & bit != 0);
                    element ^= bit;
                    bit <<= offset;
                    pc += offset;
                    steps += 1;
                }

                threes[element_index] = element;
            }
        }

        steps
    }
}

examples!(Day05 -> (u64, u64) [
    {input: "0\n3\n0\n1\n-3", part1: 5, part2: 10},
]);
