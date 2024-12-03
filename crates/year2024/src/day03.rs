use utils::prelude::*;

/// Extracting instructions from corrupted input.
#[derive(Clone, Debug)]
pub struct Day03 {
    part1: u32,
    part2: u32,
}

impl Day03 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (mut part1, mut part2) = (0, 0);
        let mut enabled = true;

        let mut input = input.as_bytes();
        while !input.is_empty() {
            if let Ok(([a, b], remaining)) = parser::u32()
                .repeat_n(b',')
                .with_prefix("mul(")
                .with_suffix(")")
                .parse(input)
            {
                part1 += a * b;
                if enabled {
                    part2 += a * b;
                }
                input = remaining;
            } else if let Some(remaining) = input.strip_prefix(b"do()") {
                enabled = true;
                input = remaining;
            } else if let Some(remaining) = input.strip_prefix(b"don't()") {
                enabled = false;
                input = remaining;
            } else {
                input = &input[1..];
            }
        }

        Ok(Self { part1, part2 })
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

examples!(Day03 -> (u32, u32) [
    {input: "xmul(2,4)%&mul[3,7]!@^do_not_mul(5,5)+mul(32,64]then(mul(11,8)mul(8,5))", part1: 161},
    {input: "xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))", part2: 48},
]);
