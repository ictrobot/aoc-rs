use utils::prelude::*;

/// Reducing the input by removing matching letter pairs.
#[derive(Clone, Debug)]
pub struct Day05 {
    reacted: Vec<u8>,
}

impl Day05 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        if let Some(index) = input.find(|c: char| !c.is_ascii_alphabetic()) {
            return Err(InputError::new(input, index, "expected ascii letter"));
        }
        if input.is_empty() {
            return Err(InputError::new(input, 0, "expected at least one letter"));
        }

        Ok(Self {
            reacted: Self::react(input.bytes()),
        })
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        self.reacted.len()
    }

    #[must_use]
    pub fn part2(&self) -> usize {
        (b'a'..=b'z')
            .map(|l| Self::react(self.reacted.iter().copied().filter(|&b| b | 32 != l)).len())
            .min()
            .unwrap()
    }

    fn react(polymer: impl Iterator<Item = u8>) -> Vec<u8> {
        let mut stack = Vec::with_capacity(polymer.size_hint().1.unwrap_or(0));
        for b in polymer {
            if let Some(last) = stack.last()
                && last ^ b == 32
            {
                stack.pop();
                continue;
            }
            stack.push(b);
        }
        stack
    }
}

examples!(Day05 -> (usize, usize) [
    {input: "aA", part1: 0},
    {input: "abBA", part1: 0},
    {input: "abAB", part1: 4},
    {input: "aabAAB", part1: 6},
    {input: "dabAcCaCBAcCcaDA", part1: 10, part2: 4},
]);
