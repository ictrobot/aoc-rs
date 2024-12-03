use utils::prelude::*;

/// Extracting instructions from corrupted input.
#[derive(Clone, Debug)]
pub struct Day03 {
    part1: u32,
    part2: u32,
}

enum Instruction {
    Mul(u32, u32),
    Do,
    Dont,
}

impl Day03 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let matches = parser::parse_tree!(
            ("mul(", a @ parser::u32(), ",", b @ parser::u32(), ")") => Instruction::Mul(a, b),
            ("don't()") => Instruction::Dont,
            ("do()") => Instruction::Do,
        )
        .matches_iterator(input);

        let (mut part1, mut part2) = (0, 0);
        let mut enabled = true;
        for instruction in matches {
            match instruction {
                Instruction::Mul(a, b) => {
                    part1 += a * b;
                    part2 += if enabled { a * b } else { 0 };
                }
                Instruction::Do => enabled = true,
                Instruction::Dont => enabled = false,
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
