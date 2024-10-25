use std::collections::HashSet;
use utils::prelude::*;

/// Checking passphrase validity.
#[derive(Clone, Debug)]
pub struct Day04<'a> {
    input: &'a str,
}

impl<'a> Day04<'a> {
    pub fn new(input: &'a str, _: InputType) -> Result<Self, InputError> {
        if let Some(w) = input.find(|c: char| !c.is_ascii_lowercase() && !c.is_ascii_whitespace()) {
            Err(InputError::new(input, w, "expected lowercase letters"))
        } else {
            Ok(Self { input })
        }
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        let mut seen = HashSet::new();
        self.input
            .lines()
            .filter(|line| {
                seen.clear();
                line.split_ascii_whitespace().all(|word| seen.insert(word))
            })
            .count()
    }

    #[must_use]
    pub fn part2(&self) -> usize {
        let mut seen = HashSet::new();
        self.input
            .lines()
            .filter(|line| {
                seen.clear();
                line.split_ascii_whitespace().all(|word| {
                    let mut letters = [0u8; 26];
                    for b in word.bytes() {
                        letters[b as usize - b'a' as usize] += 1;
                    }
                    seen.insert(letters)
                })
            })
            .count()
    }
}

examples!(Day04<'_> -> (usize, usize) [
    {input: "aa bb cc dd ee", part1: 1},
    {input: "aa bb cc dd aa", part1: 0},
    {input: "aa bb cc dd aaa", part1: 1},
    {input: "abcde fghij", part2: 1},
    {input: "abcde xyz ecdab", part2: 0},
    {input: "a ab abc abd abf abj", part2: 1},
    {input: "iiii oiii ooii oooi oooo", part2: 1},
    {input: "oiii ioii iioi iiio", part2: 0},
]);
