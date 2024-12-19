use std::num::NonZeroU32;
use utils::prelude::*;

/// Counting possible combinations of patterns to form designs.
#[derive(Clone, Debug)]
pub struct Day19 {
    part1: u64,
    part2: u64,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
enum Stripe {
    #[default]
    W,
    U,
    B,
    R,
    G,
}

#[derive(Clone, Debug, Default)]
struct TrieNode {
    child_offsets: [Option<NonZeroU32>; 5],
    is_terminal: bool,
}

const LETTER_LOOKUP: [Option<Stripe>; 256] = {
    let mut x = [None; 256];
    x[b'w' as usize] = Some(Stripe::W);
    x[b'u' as usize] = Some(Stripe::U);
    x[b'b' as usize] = Some(Stripe::B);
    x[b'r' as usize] = Some(Stripe::R);
    x[b'g' as usize] = Some(Stripe::G);
    x
};

const MAX_PATTERN_LENGTH: usize = 8;
const MAX_DESIGN_LENGTH: usize = 64;

impl Day19 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let letter = parser::byte()
            .map_res(|b| LETTER_LOOKUP[b as usize].ok_or("expected 'w', 'u', 'b', 'r' or 'g'"));

        let Some((patterns, designs)) = input.split_once("\n\n") else {
            return Err(InputError::new(input, 0, "expected patterns and designs"));
        };

        let mut trie = vec![TrieNode::default()];
        for item in letter
            .repeat_arrayvec::<MAX_PATTERN_LENGTH, _>(parser::noop(), 1)
            .with_suffix(", ".or(parser::eof()))
            .parse_iterator(patterns)
        {
            let mut index = 0;
            for &s in item?.iter() {
                match trie[index].child_offsets[s as usize] {
                    None => {
                        trie.push(TrieNode::default());
                        trie[index].child_offsets[s as usize] =
                            Some(NonZeroU32::new((trie.len() - 1 - index) as u32).unwrap());
                        index = trie.len() - 1;
                    }
                    Some(offset) => index += offset.get() as usize,
                }
            }
            trie[index].is_terminal = true;
        }

        let (mut part1, mut part2) = (0, 0);
        let mut combinations = [0; MAX_DESIGN_LENGTH + 1];
        combinations[0] = 1;
        for item in letter
            .repeat_arrayvec::<MAX_DESIGN_LENGTH, _>(parser::noop(), 1)
            .with_suffix(parser::eol())
            .parse_iterator(designs)
        {
            let design = item?;
            for len in 1..=design.len() {
                combinations[len] = 0;

                let mut trie_index = 0;
                for (i, &stripe) in design[design.len() - len..]
                    .iter()
                    .take(MAX_PATTERN_LENGTH)
                    .enumerate()
                {
                    match trie[trie_index].child_offsets[stripe as usize] {
                        None => break,
                        Some(offset) => trie_index += offset.get() as usize,
                    }

                    combinations[len] +=
                        u64::from(trie[trie_index].is_terminal) * combinations[len - 1 - i];
                }
            }

            let ways = combinations[design.len()];
            part1 += if ways > 0 { 1 } else { 0 };
            part2 += ways;
        }

        Ok(Self { part1, part2 })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        self.part2
    }
}

examples!(Day19 -> (u64, u64) [
    {file: "day19_example0.txt", part1: 6, part2: 16},
]);
