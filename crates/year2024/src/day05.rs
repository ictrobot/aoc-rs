use std::cmp::Ordering;
use utils::prelude::*;

/// Sorting lists using constraints.
///
/// The solution assumes that the rules form a total order for the elements in each sequence.
#[derive(Clone, Debug)]
pub struct Day05 {
    before: Rules,
    sorted: Vec<Vec<u32>>,
    unsorted: Vec<Vec<u32>>,
}

const MIN_NUM: usize = 10;
const MAX_NUM: usize = 99;
const RANGE: usize = MAX_NUM - MIN_NUM + 1;
type Rules = [[bool; RANGE]; RANGE];

impl Day05 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let num = parser::number_range(MIN_NUM as u32..=MAX_NUM as u32);
        let rules_parser = num.then(num.with_prefix(b'|')).repeat(parser::eol(), 1);
        let updates_parser = num.repeat(b',', 1).repeat(parser::eol(), 1);

        let (rule_list, updates) = rules_parser
            .with_eol()
            .then(updates_parser)
            .parse_complete(input)?;

        let mut before: Rules = [[false; RANGE]; RANGE];
        for (a, b) in rule_list {
            if before[a as usize - MIN_NUM][b as usize - MIN_NUM] {
                return Err(InputError::new(input, 0, "duplicate rule"));
            } else if before[b as usize - MIN_NUM][a as usize - MIN_NUM] {
                return Err(InputError::new(input, 0, "contradictory pair of rules"));
            }

            before[a as usize - MIN_NUM][b as usize - MIN_NUM] = true;
        }

        let (sorted, unsorted) = updates.into_iter().partition(|update| {
            update.is_sorted_by(|&a, &b| before[a as usize - MIN_NUM][b as usize - MIN_NUM])
        });

        Ok(Self {
            before,
            sorted,
            unsorted,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.sorted
            .iter()
            .map(|update| update[update.len() / 2])
            .sum()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.unsorted
            .iter()
            .cloned()
            .map(|mut update| {
                let index = update.len() / 2;
                // Will panic if the provided rules are not a total order
                let (_, middle, _) = update.select_nth_unstable_by(index, |&a, &b| {
                    if a == b {
                        Ordering::Equal
                    } else if self.before[a as usize - MIN_NUM][b as usize - MIN_NUM] {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                });
                *middle
            })
            .sum()
    }
}

examples!(Day05 -> (u32, u32) [
    {file: "day05_example0.txt", part1: 143, part2: 123},
]);
