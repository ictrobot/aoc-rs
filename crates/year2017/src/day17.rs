use utils::prelude::*;

/// Simulating a circular buffer.
#[derive(Clone, Debug)]
pub struct Day17 {
    step_size: u32,
}

impl Day17 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            step_size: parser::u32().parse_complete(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let mut buffer = Vec::with_capacity(2018);
        buffer.push(0);

        let mut i = 0;
        for iteration in 1..=2017u16 {
            i = (i + self.step_size as usize) % buffer.len();
            buffer.insert(i + 1, iteration);
            i += 1;
        }

        buffer[(i + 1) % buffer.len()] as u32
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut result = 0u32;

        // 0 is always at the start of the buffer, so the answer is the last value in position 1,
        // which is the last value inserted when the spinlock is at 0.
        let (mut i, mut iteration) = (0, 1);
        while iteration <= 50_000_000 {
            if i == 0 {
                result = iteration;
            }

            // Skip iterations until the spinlock wraps around to the start of the buffer again
            let skip_iterations = (iteration - i).div_ceil(self.step_size + 1);
            iteration += skip_iterations;
            i = (i + skip_iterations * (self.step_size + 1)) % iteration;
        }

        result
    }
}

examples!(Day17 -> (u32, u32) [
    {input: "3", part1: 638},
]);
