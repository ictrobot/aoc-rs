use std::collections::HashMap;
use utils::prelude::*;

/// Counting dividing stones.
#[derive(Clone, Debug)]
pub struct Day11 {
    stones: HashMap<u64, u64>,
}

impl Day11 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut stones = HashMap::new();
        for v in parser::u64().repeat(b' ', 1).parse_complete(input)? {
            *stones.entry(v).or_insert(0) += 1;
        }
        Ok(Self { stones })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        self.blink(25)
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        self.blink(75)
    }

    fn blink(&self, times: u32) -> u64 {
        let mut stones = self.stones.clone();
        let mut next = HashMap::new();
        for _ in 0..times {
            for (&x, &c) in stones.iter() {
                if x == 0 {
                    next.entry(1).and_modify(|v| *v += c).or_insert(c);
                } else {
                    let log = x.ilog10() + 1;
                    if log % 2 == 0 {
                        let pow = 10u64.pow(log / 2);
                        next.entry(x / pow).and_modify(|v| *v += c).or_insert(c);
                        next.entry(x % pow).and_modify(|v| *v += c).or_insert(c);
                    } else {
                        next.entry(x * 2024).and_modify(|v| *v += c).or_insert(c);
                    }
                }
            }

            (stones, next) = (next, stones);
            next.clear();
        }
        stones.values().sum()
    }
}

examples!(Day11 -> (u64, u64) [
    {input: "125 17", part1: 55312},
]);
