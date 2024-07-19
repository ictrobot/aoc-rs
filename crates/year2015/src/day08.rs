use utils::prelude::*;

/// Escape sequences.
#[derive(Clone, Debug)]
pub struct Day08<'a> {
    input: &'a str,
}

impl<'a> Day08<'a> {
    pub fn new(input: &'a str, _: InputType) -> Result<Self, InputError> {
        Ok(Self { input })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.input
            .bytes()
            .fold((false, 0), |(escaped, diff), b| match (escaped, b) {
                (false, b'"') => (false, diff + 1),
                (false, b'\\') => (true, diff),
                (true, b'\\') => (false, diff + 1),
                (true, b'"') => (false, diff + 1),
                (true, b'x') => (false, diff + 3),
                _ => (false, diff),
            })
            .1
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.input
            .bytes()
            .fold((false, 0), |(escaped, diff), b| match (escaped, b) {
                (false, b'"') => (false, diff + 2),
                (false, b'\\') => (true, diff + 1),
                (true, b'\\') => (false, diff + 1),
                (true, b'"') => (false, diff + 1),
                _ => (false, diff),
            })
            .1
    }
}

examples!(Day08<'_> -> (u32, u32) [
    {file: "day08_example0.txt", part1: 12, part2: 19},
]);
