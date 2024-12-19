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

const MAX_PATTERN_LENGTH: usize = 8;
const MAX_DESIGN_LENGTH: usize = 64;

impl Day19 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let letter = parser::literal_map!(
            "w" => Stripe::W,
            "u" => Stripe::U,
            "b" => Stripe::B,
            "r" => Stripe::R,
            "g" => Stripe::G,
        );

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
        let mut cache = [None; MAX_DESIGN_LENGTH];
        for item in letter
            .repeat_arrayvec::<MAX_DESIGN_LENGTH, _>(parser::noop(), 1)
            .with_suffix(parser::eol())
            .parse_iterator(designs)
        {
            cache.fill(None);
            let ways = Self::ways(&item?, &trie, &mut cache);
            part1 += if ways > 0 { 1 } else { 0 };
            part2 += ways;
        }

        Ok(Self { part1, part2 })
    }

    fn ways(design: &[Stripe], trie: &[TrieNode], cache: &mut [Option<u64>]) -> u64 {
        if design.is_empty() {
            return 1;
        }
        if let Some(result) = cache[design.len() - 1] {
            return result;
        }

        let mut total = 0;
        let mut trie_index = 0;
        for (i, &stripe) in design.iter().take(MAX_PATTERN_LENGTH).enumerate() {
            match trie[trie_index].child_offsets[stripe as usize] {
                None => break,
                Some(offset) => trie_index += offset.get() as usize,
            }

            if trie[trie_index].is_terminal {
                total += Self::ways(&design[i + 1..], trie, cache);
            }
        }

        cache[design.len() - 1] = Some(total);
        total
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
