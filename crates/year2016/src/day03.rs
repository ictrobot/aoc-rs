use utils::prelude::*;

/// Checking triangle validity.
#[derive(Clone, Debug)]
pub struct Day03 {
    input: Vec<[u32; 3]>,
}

impl Day03 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            input: parser::u32()
                .with_prefix(parser::take_while(u8::is_ascii_whitespace))
                .repeat()
                .parse_lines(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        self.input
            .iter()
            .filter(|&&[a, b, c]| Self::valid_triangle(a, b, c))
            .count()
    }

    #[must_use]
    pub fn part2(&self) -> usize {
        self.input
            .chunks_exact(3)
            .flat_map(|w| {
                (0..3).map(|i| usize::from(Self::valid_triangle(w[0][i], w[1][i], w[2][i])))
            })
            .sum()
    }

    #[allow(clippy::overflow_check_conditional)]
    fn valid_triangle(a: u32, b: u32, c: u32) -> bool {
        let sum = a + b + c;
        let max = a.max(b).max(c);
        // This isn't checking for overflow, but that the shortest 2 sides (sum - max) are longer
        // than the longest side
        sum - max > max
    }
}

examples!(Day03 -> (usize, usize) [
    {input: "5 10 25", part1: 0},
]);
