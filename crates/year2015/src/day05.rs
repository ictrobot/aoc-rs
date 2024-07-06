use utils::prelude::*;

/// Matching string patterns.
#[derive(Clone, Debug)]
pub struct Day05<'a> {
    lines: Vec<&'a str>,
}

impl<'a> Day05<'a> {
    pub fn new(input: &'a str, _: InputType) -> Result<Self, InvalidInputError> {
        // Validate input is ASCII lowercase and newlines
        if let Some(c) = input
            .chars()
            .find(|&c| !c.is_ascii_lowercase() && c != '\n')
        {
            Err(InvalidInputError::UnexpectedChar(c))
        } else {
            Ok(Self {
                lines: input.lines().collect(),
            })
        }
    }

    #[must_use]
    #[allow(clippy::eq_op)]
    pub fn part1(&self) -> usize {
        const VOWELS: u32 = 1 << (b'a' - b'a')
            | 1 << (b'e' - b'a')
            | 1 << (b'i' - b'a')
            | 1 << (b'o' - b'a')
            | 1 << (b'u' - b'a');

        self.lines
            .iter()
            // At least one letter that appears twice in a row
            .filter(|&&l| l.as_bytes().windows(2).any(|w| w[0] == w[1]))
            // At least 3 vowels
            .filter(|&&l| {
                l.as_bytes()
                    .iter()
                    // Using a mask to match vowels instead of chained equals is ~2x faster
                    .filter(|&&b| VOWELS & (1 << (b - b'a')) != 0)
                    .count()
                    >= 3
            })
            // Not any of these strings
            .filter(|&&l| {
                !l.contains("ab") && !l.contains("cd") && !l.contains("pq") && !l.contains("xy")
            })
            .count()
    }

    #[must_use]
    pub fn part2(&self) -> usize {
        // Share an array to avoid clearing it for each string
        let mut pair_positions = [0u32; 729];
        let mut pos = 0;

        self.lines
            .iter()
            // Contains a letter that repeats 2 characters later
            .filter(|&&l| l.as_bytes().windows(3).any(|w| w[0] == w[2]))
            // Contains a repeated pair of letters (without overlapping)
            .filter(|&&l| {
                let string_start = pos;
                l.as_bytes().windows(2).any(|w| {
                    let pair = 26 * (w[0] - b'a') as usize + (w[1] - b'a') as usize;
                    if pair_positions[pair] > string_start {
                        // Already seen the pair earlier in this string
                        if pair_positions[pair] < pos {
                            // Match found as pairs don't overlap
                            return true;
                        }
                    } else {
                        // First occurrence of the pair in this string
                        pair_positions[pair] = pos + 1;
                    }
                    pos += 1;
                    false
                })
            })
            .count()
    }
}

examples!(Day05<'_> -> (usize, usize) [
    {input: "ugknbfddgicrmopn", part1: 1},
    {input: "aaa", part1: 1, part2: 0},
    {input: "aaaa", part1: 1, part2: 1},
    {input: "jchzalrnumimnmhp", part1: 0},
    {input: "haegwjzuvuyypxyu", part1: 0},
    {input: "dvszwmarrgswjxmb", part1: 0},
    {input: "qjhvhtzxzqqjkmpb", part2: 1},
    {input: "xxyxx", part2: 1},
    {input: "uurcxstgmygtbstg", part2: 0},
    {input: "ieodomkazucvgmuy", part2: 0},
]);
