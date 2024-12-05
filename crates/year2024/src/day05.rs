use utils::prelude::*;

/// Sorting lists using a partial order of constraints.
#[derive(Clone, Debug)]
pub struct Day05 {
    rules: Rules,
    sorted: Vec<Vec<u32>>,
    unsorted: Vec<Vec<u32>>,
}

const MIN_NUM: usize = 10;
const MAX_NUM: usize = 99;
const RANGE: usize = MAX_NUM - MIN_NUM + 1;
type Rules = [bool; RANGE * RANGE];

impl Day05 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let num = parser::number_range(MIN_NUM as u32..=MAX_NUM as u32);
        let rules_parser = num.then(num.with_prefix(b'|')).repeat(b'\n', 1);
        let updates_parser = num.repeat(b',', 1).repeat(b'\n', 1);

        let (rules_list, updates) = rules_parser
            .then(updates_parser.with_prefix(b'\n'))
            .parse_complete(input)?;

        let mut rules = [false; RANGE * RANGE];
        for (a, b) in rules_list {
            rules[(a as usize - MIN_NUM) * RANGE + b as usize] = true;
        }

        let (sorted, unsorted) = updates.into_iter().partition(|update| {
            for (i, &page1) in update.iter().enumerate() {
                for &page2 in update.iter().skip(i + 1) {
                    if Self::must_be_before(&rules, page2, page1) {
                        return false;
                    }
                }
            }
            true
        });

        Ok(Self {
            rules,
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
        let mut result = 0;
        let mut list = Vec::new();
        for original in &self.unsorted {
            original.clone_into(&mut list);

            for _ in 0..(list.len() / 2) {
                let i = Self::find_first_index(&self.rules, &list);
                list.swap_remove(i);
            }

            let i = Self::find_first_index(&self.rules, &list);
            result += list[i];
        }

        result
    }

    fn must_be_before(rules: &Rules, page1: u32, page2: u32) -> bool {
        rules[(page1 as usize - MIN_NUM) * RANGE + page2 as usize]
    }

    fn find_first_index(rules: &Rules, list: &[u32]) -> usize {
        'outer: for (i, &page1) in list.iter().enumerate() {
            for (j, &page2) in list.iter().enumerate() {
                if i != j && Self::must_be_before(rules, page2, page1) {
                    continue 'outer;
                }
            }

            return i;
        }
        panic!("no solution found");
    }
}

examples!(Day05 -> (u32, u32) [
    {file: "day05_example0.txt", part1: 143, part2: 123},
]);
