use utils::prelude::*;

/// Merging integer ranges.
#[derive(Clone, Debug)]
pub struct Day05 {
    ranges: Vec<[u64; 2]>,
    ids: Vec<u64>,
}

impl Day05 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (ranges, ids) = parser::u64()
            .repeat_n(b'-')
            .repeat(parser::eol(), 1)
            .with_eol()
            .with_eol()
            .then(parser::u64().repeat(parser::eol(), 1))
            .parse_complete(input)?;

        Ok(Self { ranges, ids })
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        self.ids
            .iter()
            .filter(|&&id| {
                self.ranges
                    .iter()
                    .any(|range| range[0] <= id && id <= range[1])
            })
            .count()
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        let mut ranges = self.ranges.clone();
        ranges.sort_unstable();

        let mut total = 0u64;
        let [mut start, mut end] = ranges[0];
        for &[s, e] in ranges.iter().skip(1) {
            if s > end + 1 {
                total += end - start + 1;
                start = s;
                end = e;
            } else if e > end {
                end = e;
            }
        }
        total += end - start + 1;

        total
    }
}

examples!(Day05 -> (usize, u64) [
    {input: "3-5\n10-14\n16-20\n12-18\n\n1\n5\n8\n11\n17\n32", part1: 3, part2: 14},
]);
