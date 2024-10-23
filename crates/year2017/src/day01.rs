use utils::prelude::*;

/// Adding matching digits.
#[derive(Clone, Debug)]
pub struct Day01<'a> {
    input: &'a [u8],
}

impl<'a> Day01<'a> {
    pub fn new(input: &'a str, _: InputType) -> Result<Self, InputError> {
        if let Some(b) = input.bytes().find(|b| !b.is_ascii_digit()) {
            Err(InputError::new(input, b as char, "expected digit"))
        } else {
            Ok(Self {
                input: input.as_bytes(),
            })
        }
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.input
            .iter()
            .zip(self.input.iter().cycle().skip(1))
            .map(|(&a, &b)| if a == b { (a - b'0') as u32 } else { 0 })
            .sum()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.input
            .iter()
            .zip(self.input.iter().cycle().skip(self.input.len() / 2))
            .map(|(&a, &b)| if a == b { (a - b'0') as u32 } else { 0 })
            .sum()
    }
}

examples!(Day01<'_> -> (u32, u32) [
    {input: "1122", part1: 3},
    {input: "1111", part1: 4},
    {input: "1234", part1: 0},
    {input: "91212129", part1: 9},
    {input: "1212", part2: 6},
    {input: "1221", part2: 0},
    {input: "123425", part2: 4},
    {input: "123123", part2: 12},
    {input: "12131415", part2: 4},
]);
