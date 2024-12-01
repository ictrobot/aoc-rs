use utils::prelude::*;

/// Comparing two lists of numbers.
#[derive(Clone, Debug)]
pub struct Day01 {
    left: Vec<u32>,
    right: Vec<u32>,
}

impl Day01 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let pairs = parser::u32()
            .then(parser::u32().with_prefix("   "))
            .parse_lines(input)?;

        let mut left = pairs.iter().map(|x| x.0).collect::<Vec<_>>();
        left.sort_unstable();

        let mut right = pairs.into_iter().map(|x| x.1).collect::<Vec<_>>();
        right.sort_unstable();

        Ok(Self { left, right })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.left
            .iter()
            .zip(&self.right)
            .map(|(&l, &r)| l.abs_diff(r))
            .sum()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut score = 0;
        let mut i = 0;
        for &l in &self.left {
            while i < self.right.len() && self.right[i] < l {
                i += 1;
            }

            let mut j = i;
            while j < self.right.len() && self.right[j] == l {
                score += l;
                j += 1;
            }
        }
        score
    }
}

examples!(Day01 -> (u32, u32) [
    {input: "3   4\n4   3\n2   5\n1   3\n3   9\n3   3", part1: 11, part2: 31},
]);
