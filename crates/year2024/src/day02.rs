use utils::array::ArrayVec;
use utils::prelude::*;

/// Checking sequences of numbers.
#[derive(Clone, Debug)]
pub struct Day02 {
    reports: Vec<ArrayVec<u32, 8>>,
}

impl Day02 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            reports: parser::u32().repeat_arrayvec(b' ', 1).parse_lines(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        self.reports
            .iter()
            .filter(|&r| Self::is_safe(r.iter().copied()))
            .count()
    }

    #[must_use]
    pub fn part2(&self) -> usize {
        self.reports
            .iter()
            .filter(|&r| {
                if Self::is_safe(r.iter().copied()) {
                    return true;
                }

                (0..r.len())
                    .any(|i| Self::is_safe(r.iter().take(i).chain(r.iter().skip(i + 1)).copied()))
            })
            .count()
    }

    #[inline]
    fn is_safe(iter: impl Iterator<Item = u32> + Clone) -> bool {
        let mut w = iter.clone().zip(iter.skip(1));

        w.clone().all(|(a, b)| (1..=3).contains(&a.wrapping_sub(b)))
            || w.all(|(a, b)| (1..=3).contains(&b.wrapping_sub(a)))
    }
}

examples!(Day02 -> (usize, usize) [
    {input: "7 6 4 2 1", part1: 1, part2: 1},
    {input: "1 2 7 8 9", part1: 0, part2: 0},
    {input: "9 7 6 2 1", part1: 0, part2: 0},
    {input: "1 3 2 4 5", part1: 0, part2: 1},
    {input: "8 6 4 4 1", part1: 0, part2: 1},
    {input: "1 3 6 7 9", part1: 1, part2: 1},
]);
