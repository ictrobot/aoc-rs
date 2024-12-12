use std::collections::{HashMap, VecDeque};
use utils::prelude::*;

/// Counting dividing stones.
#[derive(Clone, Debug)]
pub struct Day11 {
    pub counts: Vec<u64>,
    pub next: Vec<(usize, usize)>,
    pub max_idx: Vec<usize>,
}

// Placeholder stone number, used as the second stone when a stone only splits into one
const PLACEHOLDER: u64 = u64::MAX;

impl Day11 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut builder = Builder::new(75);

        let mut counts = Vec::new();
        for n in parser::u64().repeat(b' ', 1).parse_complete(input)? {
            let idx = builder.index(n, 0);
            if idx >= counts.len() {
                counts.resize(idx + 1, 0);
            }
            counts[idx] += 1;
        }

        // Precompute all stone divisions
        let (next, max_idx) = builder.finish();

        Ok(Self {
            counts,
            next,
            max_idx,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        self.stones(25)
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        self.stones(75)
    }

    fn stones(&self, blinks: usize) -> u64 {
        let mut counts = vec![0; self.next.len()];
        let mut next = vec![0; self.next.len()];
        counts[..self.counts.len()].copy_from_slice(&self.counts);

        for blink in 0..blinks {
            for (c, &(a, b)) in counts[..=self.max_idx[blink]].iter_mut().zip(&self.next) {
                if *c > 0 {
                    next[a] += *c;
                    next[b] += *c;
                    *c = 0;
                }
            }

            // Clear placeholder stones
            next[0] = 0;

            (counts, next) = (next, counts);
        }

        counts.iter().sum()
    }
}

struct Builder {
    num_map: HashMap<u64, usize>,
    next: Vec<(usize, usize)>,
    max_idx: Vec<usize>,
    todo: VecDeque<(u64, usize, u32)>,
}

impl Builder {
    fn new(blinks: u32) -> Self {
        let mut num_map = HashMap::with_capacity(5000);
        let mut next = Vec::with_capacity(5000);

        // Always insert placeholder stone as index 0
        num_map.insert(PLACEHOLDER, 0);
        next.push((0, 0));

        Self {
            num_map,
            next,
            todo: VecDeque::with_capacity(500),
            max_idx: vec![0; blinks as usize],
        }
    }

    fn index(&mut self, n: u64, blinks: u32) -> usize {
        let next_idx = self.num_map.len();
        *self.num_map.entry(n).or_insert_with(|| {
            self.next.push((0, 0));
            self.max_idx[blinks as usize] = self.max_idx[blinks as usize].max(next_idx);
            if (blinks as usize) < self.max_idx.len() {
                self.todo.push_back((n, next_idx, blinks));
            }
            next_idx
        })
    }

    fn finish(mut self) -> (Vec<(usize, usize)>, Vec<usize>) {
        while let Some((n, idx, blink)) = self.todo.pop_front() {
            self.next[idx] = if n == 0 {
                (self.index(1, blink + 1), 0)
            } else {
                let log = n.ilog10() + 1;
                if log % 2 == 0 {
                    let pow = 10u64.pow(log / 2);
                    (
                        self.index(n / pow, blink + 1),
                        self.index(n % pow, blink + 1),
                    )
                } else {
                    (self.index(n * 2024, blink + 1), 0)
                }
            };
        }

        // Max index is an optimization to reduce the number of indexes iterated over in the first
        // blinks. Ensure it is always increasing, as the insert function only updates it when
        // adding a new number, which means that blinks with no new numbers will have max_idx = 0
        // without this.
        for i in 1..self.max_idx.len() {
            self.max_idx[i] = self.max_idx[i].max(self.max_idx[i - 1]);
        }

        (self.next, self.max_idx)
    }
}

examples!(Day11 -> (u64, u64) [
    {input: "125 17", part1: 55312},
]);
