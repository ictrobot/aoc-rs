use std::iter;
use utils::prelude::*;

/// Finding the first repeat.
#[derive(Clone, Debug)]
pub struct Day01 {
    changes: Vec<i32>,
    total: i32,
}

impl Day01 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let changes = parser::i32().parse_lines(input)?;
        Ok(Self {
            total: changes.iter().sum(),
            changes,
        })
    }

    #[must_use]
    pub fn part1(&self) -> i32 {
        self.total
    }

    #[must_use]
    pub fn part2(&self) -> i32 {
        if self.total == 0 {
            let mut frequencies = iter::once(0)
                .chain(self.changes.iter().scan(0i32, |sum, &c| {
                    *sum += c;
                    Some(*sum)
                }))
                .enumerate()
                .collect::<Vec<_>>();

            frequencies.sort_unstable_by_key(|&(idx, freq)| (freq, idx));

            frequencies
                .windows(2)
                .filter_map(|w| {
                    let (_, freq1) = w[0];
                    let (idx2, freq2) = w[1];
                    if freq1 == freq2 {
                        Some((idx2, freq1))
                    } else {
                        None
                    }
                })
                .min()
        } else {
            let mut remainders = self
                .changes
                .iter()
                .enumerate()
                .scan(0i32, |freq, (t, &change)| {
                    let curr = *freq;
                    *freq += change;
                    Some((curr.rem_euclid(self.total), curr, t))
                })
                .collect::<Vec<_>>();

            remainders.sort_unstable();

            remainders
                .chunk_by(|&(r1, _, _), &(r2, _, _)| r1 == r2)
                .flat_map(|chunk| {
                    chunk.iter().enumerate().flat_map(|(i, &r1)| {
                        chunk.iter().skip(i + 1).filter_map(move |&r2| {
                            let ((_, freq1, t1), (_, freq2, _)) =
                                if self.total > 0 { (r1, r2) } else { (r2, r1) };

                            let freq_difference = freq2 - freq1;
                            let iterations = freq_difference / self.total;
                            if freq_difference.rem_euclid(self.total) == 0 && iterations >= 0 {
                                let iterations = iterations as usize;
                                Some((iterations * self.changes.len() + t1, freq2))
                            } else {
                                None
                            }
                        })
                    })
                })
                .min()
        }
        .map(|(_, freq)| freq)
        .expect("no solution found")
    }
}

examples!(Day01 -> (i32, i32) [
    {input: "+1\n-2\n+3\n+1", part1: 3, part2: 2},
    {input: "+1\n+1\n+1", part1: 3},
    {input: "+1\n+1\n-2", part1: 0},
    {input: "-1\n-2\n-3", part1: -6},
    {input: "+1\n-1", part2: 0},
    {input: "+3\n+3\n+4\n-2\n-4", part2: 10},
    {input: "-6\n+3\n+8\n+5\n-6", part2: 5},
    {input: "+7\n+7\n-2\n-7\n-4", part2: 14},
    // Negative versions of the above
    {input: "-1\n+2\n-3\n-1", part1: -3, part2: -2},
    {input: "-1\n+1", part2: 0},
    {input: "-3\n-3\n-4\n+2\n+4", part2: -10},
    {input: "+6\n-3\n-8\n-5\n+6", part2: -5},
    {input: "-7\n-7\n+2\n+7\n+4", part2: -14},
    // Custom examples
    {input: "+2\n+3\n-8", part2: 2},
    {input: "+0", part2: 0},
    {input: "+1000000\n-999999", part2: 1000000},
    {input: "+1\n-2\n+2\n-2\n+3", part2: 1},
    {input: "+1\n-2\n+2\n-2\n-5", part2: 1},
]);
