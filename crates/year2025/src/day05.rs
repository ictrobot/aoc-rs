use std::cmp::Ordering;
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

        Ok(Self {
            ranges: Self::merge_ranges(ranges),
            ids,
        })
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        self.ids
            .iter()
            .filter(|&&id| {
                self.ranges
                    .binary_search_by(|&[s, e]| {
                        if e < id {
                            Ordering::Less
                        } else if s > id {
                            Ordering::Greater
                        } else {
                            Ordering::Equal
                        }
                    })
                    .is_ok()
            })
            .count()
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        self.ranges.iter().map(|[s, e]| e - s + 1).sum()
    }

    fn merge_ranges(mut ranges: Vec<[u64; 2]>) -> Vec<[u64; 2]> {
        if ranges.is_empty() {
            return ranges;
        }

        ranges.sort_unstable_by_key(|&[s, _]| s);

        let [mut start, mut end] = ranges[0];
        ranges = ranges
            .into_iter()
            .flat_map(|[s, e]| {
                if s > end.saturating_add(1) {
                    let result = [start, end];
                    [start, end] = [s, e];
                    Some(result)
                } else {
                    end = end.max(e);
                    None
                }
            })
            .collect();
        ranges.push([start, end]);

        ranges
    }
}

examples!(Day05 -> (usize, u64) [
    {input: "3-5\n10-14\n16-20\n12-18\n\n1\n5\n8\n11\n17\n32", part1: 3, part2: 14},
]);
