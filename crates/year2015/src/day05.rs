use utils::prelude::*;

/// Matching string patterns.
#[derive(Clone, Debug)]
pub struct Day05<'a> {
    lines: Vec<&'a [u8]>,
}

impl<'a> Day05<'a> {
    pub fn new(input: &'a str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            lines: parser::take_while1(u8::is_ascii_lowercase)
                .error_msg("expected a-z")
                .parse_lines(input)?,
        })
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
            .filter(|&&l| l.windows(2).any(|w| w[0] == w[1]))
            // At least 3 vowels
            .filter(|&&l| {
                l.iter()
                    // Using a mask to match vowels instead of chained equals is ~2x faster
                    .filter(|&&b| VOWELS & (1 << (b - b'a')) != 0)
                    .count()
                    >= 3
            })
            // Not any of these strings
            .filter(|&&l| l.windows(2).all(|w| w != b"ab"))
            .filter(|&&l| l.windows(2).all(|w| w != b"cd"))
            .filter(|&&l| l.windows(2).all(|w| w != b"pq"))
            .filter(|&&l| l.windows(2).all(|w| w != b"xy"))
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
            .filter(|&&l| l.windows(3).any(|w| w[0] == w[2]))
            // Contains a repeated pair of letters (without overlapping)
            .filter(|&&l| {
                let string_start = pos;
                l.windows(2).any(|w| {
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
