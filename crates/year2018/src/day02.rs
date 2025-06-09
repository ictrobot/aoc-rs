use std::collections::HashSet;
use utils::array::ArrayVec;
use utils::prelude::*;

/// Finding near-match IDs.
#[derive(Clone, Debug)]
pub struct Day02 {
    ids: Vec<ArrayVec<u8, 32>>,
}

impl Day02 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            ids: parser::byte_range(b'a'..=b'z')
                .repeat_arrayvec(parser::noop(), 1)
                .parse_lines(input)?,
        })
    }

    #[must_use]
    #[expect(clippy::manual_contains, reason = ".iter().any() is faster here")]
    pub fn part1(&self) -> u32 {
        let mut twos = 0;
        let mut threes = 0;
        for id in &self.ids {
            let mut freq = [0u8; 26];
            for &i in id {
                freq[(i - b'a') as usize] += 1;
            }
            twos += u32::from(freq.iter().any(|&x| x == 2));
            threes += u32::from(freq.iter().any(|&x| x == 3));
        }
        twos * threes
    }

    #[must_use]
    pub fn part2(&self) -> String {
        let mut seen = HashSet::with_capacity(self.ids.len());
        for column in 0..self.ids[0].capacity() {
            for mut id in self.ids.iter().map(|id| id.clone().into_array()) {
                id[column] = 0;
                if !seen.insert(id) {
                    return id.iter().filter(|&&x| x != 0).map(|&x| x as char).collect();
                }
            }
            seen.clear();
        }
        panic!("no solution found")
    }
}

examples!(Day02 -> (u32, &'static str) [
    {input: "abcdef\nbababc\nabbcde\nabcccd\naabcdd\nabcdee\nababab", part1: 12},
    {input: "abcde\nfghij\nklmno\npqrst\nfguij\naxcye\nwvxyz", part2: "fgij"},
]);
