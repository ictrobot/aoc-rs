use utils::prelude::*;

/// Searching for a pattern in an infinite recipe list.
///
/// This implementation is based on
/// [/u/askalski's post "Breaking the 1 billion recipes per second barrier"](https://www.reddit.com/r/adventofcode/comments/a6wpwa/2018_day_14_breaking_the_1_billion_recipes_per/)
/// and accompanying [gist](https://gist.github.com/Voltara/5069980afbf6cf0762fcbb09948e5649),
/// adapted to safe Rust without intrinsics or explicit SIMD while achieving similar performance.
///
/// The key optimization is that all the elves converge on the same subset of recipes after the
/// first 23 recipes, allowing them to be processed in bulk using SIMD.
#[derive(Clone, Debug)]
pub struct Day14 {
    part1: [u8; 10],
    part2: usize,
}

impl Day14 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let pattern = parser::digit()
            .repeat_n(parser::noop())
            .parse_complete(input)?;
        let index = input.parse().unwrap();

        let mut searcher = Searcher::new(pattern);
        let part2 = searcher.run();

        // Use a string for part 1 to preserve leading zeros
        let mut part1 = [0u8; 10];
        part1.copy_from_slice(&searcher.recipes[index..index + 10]);
        part1.iter_mut().for_each(|x| *x += b'0');

        Ok(Self { part1, part2 })
    }

    #[must_use]
    pub fn part1(&self) -> &str {
        str::from_utf8(&self.part1).unwrap()
    }

    #[must_use]
    pub fn part2(&self) -> usize {
        self.part2
    }
}

struct Searcher {
    pattern: [u8; 6],
    recipes: Vec<u8>,
    recipes_len: usize,
    packed: Vec<u8>,
    packed_len: usize,
    packed_next_idx: usize,
    packed_last_idx: usize,
    elves: [usize; 2],
    initial: [i32; 2],
    searched: usize,
}

// Pre-calculated sequences of recipes processed before index 23 where they all converge
const INITIAL_SEQUENCES: [u32; 9] = [
    0xfff94113, 0xffff0657, 0xfff94111, 0xfff94110, 0xffff9411, 0xffff9410, 0xfffff941, 0xffff1812,
    0xffffff94,
];

// The first 23 recipes
const RECIPES: [u8; 23] = [
    3, 7, 1, 0, 1, 0, 1, 2, 4, 5, 1, 5, 8, 9, 1, 6, 7, 7, 9, 2, 5, 1, 0,
];

const RECIPES_LEN: usize = 22_000_000;
const PACKED_LEN: usize = RECIPES_LEN / 4;
const SEARCH_INTERVAL: usize = 16384;

impl Searcher {
    fn new(pattern: [u8; 6]) -> Self {
        // Prefill the recipes array with ones so 2-digit recipes can be written with a single
        // write by skipping over the leading 1.
        let mut recipes = vec![1u8; RECIPES_LEN];
        recipes[..RECIPES.len()].copy_from_slice(&RECIPES);

        Self {
            pattern,
            recipes,
            recipes_len: RECIPES.len(),
            packed: vec![0u8; PACKED_LEN],
            packed_len: 0,
            packed_next_idx: RECIPES.len(),
            packed_last_idx: RECIPES.len(),
            elves: [0, 0],
            initial: [INITIAL_SEQUENCES[0] as i32, INITIAL_SEQUENCES[8] as i32],
            searched: 0,
        }
    }

    #[inline]
    fn append_recipe(&mut self, mut recipe: u8) {
        // Tens digit
        if recipe >= 10 {
            recipe -= 10;
            if self.recipes_len == self.packed_next_idx {
                self.packed[self.packed_len] = 1;
                self.packed_len += 1;
                self.packed_last_idx = self.packed_next_idx;
                self.packed_next_idx += 2;
            }
            // No write to self.recipes as the array is prefilled with 1
            self.recipes_len += 1;
        }

        // Ones digit
        if self.recipes_len == self.packed_next_idx {
            self.packed[self.packed_len] = recipe;
            self.packed_len += 1;
            self.packed_last_idx = self.packed_next_idx;
            self.packed_next_idx += recipe as usize + 1;
        }
        self.recipes[self.recipes_len] = recipe;
        self.recipes_len += 1;

        // Handle wrapping around to the start
        for i in 0..2 {
            if self.elves[i] == self.packed_len {
                let initial_index = self.packed_last_idx
                    + (self.packed[self.packed_len - 1] as usize + 1)
                    - self.recipes_len;
                self.initial[i] = INITIAL_SEQUENCES[initial_index] as i32;
                self.elves[i] = 0;
            }
        }
    }

    #[inline]
    fn run(&mut self) -> usize {
        loop {
            // At least one elf is in the first 23 recipes, before the pattern coverages
            while self.initial[0] != -1 || self.initial[1] != -1 {
                let mut recipe = 0;
                for i in 0..2 {
                    if self.initial[i] != -1 {
                        recipe += (self.initial[i] & 0xF) as u8;
                        self.initial[i] >>= 4;
                    } else {
                        recipe += self.packed[self.elves[i]];
                        self.elves[i] += 1;
                    }
                }
                self.append_recipe(recipe);
            }

            // Both elves are after recipe 23, so recipes can be processed in bulk
            loop {
                let mut iterations = (self.packed_len - self.elves[0].max(self.elves[1])) / 32;
                if iterations == 0 {
                    break;
                }

                while iterations > 0 {
                    if iterations > 16 {
                        self.bulk_mix::<512>();
                        iterations -= 16;
                    } else {
                        self.bulk_mix::<32>();
                        iterations -= 1;
                    }

                    // Periodically search for the pattern
                    if self.searched + SEARCH_INTERVAL < self.recipes_len
                        && let Some(index) = self.search()
                    {
                        return index;
                    }
                }

                // Pack the new recipes
                while self.packed_next_idx < self.recipes_len {
                    self.packed_last_idx = self.packed_next_idx;
                    self.packed[self.packed_len] = self.recipes[self.packed_next_idx];
                    self.packed_len += 1;
                    self.packed_next_idx += self.recipes[self.packed_next_idx] as usize + 1;
                }
            }

            // Handle the remaining recipes before wrapping around to the start
            while self.initial[0] == -1 && self.initial[1] == -1 {
                let sum = self.packed[self.elves[0]] + self.packed[self.elves[1]];
                self.elves[0] += 1;
                self.elves[1] += 1;
                self.append_recipe(sum);
            }
        }
    }

    #[inline]
    fn bulk_mix<const N: usize>(&mut self) {
        const { assert!(N.is_multiple_of(32)) }

        let v0 = &self.packed[self.elves[0]..self.elves[0] + N];
        let v1 = &self.packed[self.elves[1]..self.elves[1] + N];
        self.elves[0] += N;
        self.elves[1] += N;

        // Calculate the ones digit and carry for each recipe sum.
        // This loop should be vectorized by the compiler.
        let mut digits = [0u8; N];
        let mut carry = [0u8; N];
        for i in 0..N {
            let s = v0[i] + v1[i];
            let c = u8::from(s >= 10);
            carry[i] = c;
            digits[i] = s - (c * 10);
        }

        // Process the carry and digits sequentially in chunks of 8. This allows calculating the
        // indexes for all 8 items in a few u64 operations, avoiding dependencies between each
        // recipe.
        for (digits, carry) in digits.chunks_exact(8).zip(carry.chunks_exact(8)) {
            let slice = &mut self.recipes[self.recipes_len..self.recipes_len + 256];

            let mut indexes = u64::from_le_bytes(carry.try_into().unwrap());
            // Each recipe after the first is offset by 1
            indexes += 0x0101_0101_0101_0100;
            // Bytewise prefix sum
            indexes += indexes << 8;
            indexes += indexes << 16;
            indexes += indexes << 32;

            for (i, &d) in digits.iter().enumerate() {
                // Use a u8 index into a 256 long slice to remove bounds checks here, reducing
                // the number of bounds checks to 1 per 8 recipes.
                let idx = (indexes >> (i * 8)) as u8;
                slice[idx as usize] = d;
            }

            self.recipes_len += (indexes >> 56) as usize + 1;
        }
    }

    #[inline]
    fn search(&mut self) -> Option<usize> {
        // Search for the pattern in sliding 64-byte windows
        while self.searched + 64 + 5 < self.recipes_len {
            // Find matches of the first two bytes across the window.
            // This loop should be vectorized by the compiler.
            let mut candidates = [0u8; 64];
            let mut any_matches = false;
            for (i, (&a, &b)) in self.recipes[self.searched..self.searched + 64]
                .iter()
                .zip(&self.recipes[self.searched + 1..self.searched + 65])
                .enumerate()
            {
                let matches = (a == self.pattern[0]) & (b == self.pattern[1]);
                // wrapping_neg produces 0xFF for matches and 0x00 for non-matches, which matches
                // x86 SIMD comparison function semantics, slightly improving codegen.
                candidates[i] = u8::from(matches).wrapping_neg();
                any_matches |= matches;
            }

            // If no matches were found, skip to the next window.
            // Do this before calculating the mask which can be expensive.
            if !any_matches {
                self.searched += 64;
                continue;
            }

            let mut mask = candidates
                .iter()
                .enumerate()
                .fold(0u64, |acc, (i, &x)| acc | u64::from(x != 0) << i);

            // For each bit in the mask, check if the full pattern matches.
            while mask != 0 {
                let m = mask.trailing_zeros() as usize;
                mask &= !(1 << m);

                let index = self.searched + m;
                if self.recipes[index..index + 6] == self.pattern {
                    return Some(index);
                }
            }

            self.searched += 64;
        }

        None
    }
}

examples!(Day14 -> (&'static str, usize) [
    // Custom examples
    {input: "100100", part1: "8239938101", part2: 377203},
    {input: "123456", part1: "1371276618", part2: 450697},
    {input: "924933", part1: "0012267210", part2: 16462928},
    {input: "054274", part1: "6112136872", part2: 21000203},
]);
