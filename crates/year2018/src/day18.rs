use utils::bit::bitwise_count8;
use utils::prelude::*;

/// Simulating a cyclic forest cellular automaton.
#[derive(Clone, Debug)]
pub struct Day18 {
    masks: [[u64; 52]; 2],
}

impl Day18 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        // 50x50 bitmasks, with 1 bit/row of padding on each side
        let mut tree_mask = [0u64; 52];
        let mut lumberyard_mask = [0u64; 52];

        let mut row = 0;
        for line in input.lines() {
            if row >= 50 {
                return Err(InputError::new(input, line, "too many rows"));
            }
            if line.len() > 50 {
                return Err(InputError::new(input, line, "too many columns"));
            }
            for (col, b) in line.bytes().enumerate() {
                match b {
                    b'.' => {}
                    b'|' => tree_mask[row + 1] |= 1 << (col + 1),
                    b'#' => lumberyard_mask[row + 1] |= 1 << (col + 1),
                    _ => {
                        return Err(InputError::new(input, b as char, "expected '.', '|', '#'"));
                    }
                };
            }
            row += 1;
        }

        if row != 50 {
            return Err(InputError::new(input, 0, "expected 50 rows"));
        }

        Ok(Self {
            masks: [tree_mask, lumberyard_mask],
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let mut masks = &mut self.masks.clone();
        let mut temp = &mut [[0u64; _]; _];

        for _ in 0..10 {
            Self::advance(masks, temp);
            (masks, temp) = (temp, masks);
        }

        Self::resource_value(masks)
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        // Brent's algorithm for cycle detection, as used in 2017 day 6.
        let (mut power, mut lambda) = (1, 1);

        let mut tortoise = &mut self.masks.clone();
        let mut hare = &mut self.masks.clone();
        let mut temp = &mut [[0u64; _]; _];

        Self::advance(hare, temp);
        (hare, temp) = (temp, hare);

        while tortoise != hare {
            if power == lambda {
                *tortoise = *hare;
                power *= 2;
                lambda = 0;
            }

            Self::advance(hare, temp);
            (hare, temp) = (temp, hare);

            lambda += 1;
        }

        *tortoise = self.masks;
        *hare = self.masks;
        for _ in 0..lambda {
            Self::advance(hare, temp);
            (hare, temp) = (temp, hare);
        }

        let mut mu = 0;
        while tortoise != hare {
            Self::advance(tortoise, temp);
            (tortoise, temp) = (temp, tortoise);

            Self::advance(hare, temp);
            (hare, temp) = (temp, hare);

            mu += 1;
        }

        let mut minutes = 1_000_000_000 - mu;
        minutes -= (minutes / lambda) * lambda;

        while minutes > 0 {
            Self::advance(hare, temp);
            (hare, temp) = (temp, hare);

            minutes -= 1;
        }

        Self::resource_value(hare)
    }

    #[inline(never)]
    fn advance([trees, yards]: &[[u64; 52]; 2], [next_trees, next_yards]: &mut [[u64; 52]; 2]) {
        const COLUMNS_MASK: u64 = ((1 << 50) - 1) << 1;

        // This loop should be vectorized by the compiler
        for row in 1..51 {
            let (gte1_trees, gte3_trees) = Self::adjacent_gte1_gte3(trees, row);
            let (gte1_yards, gte3_yards) = Self::adjacent_gte1_gte3(yards, row);

            let open_to_trees = !trees[row] & !yards[row] & gte3_trees;
            let trees_to_yard = trees[row] & gte3_yards;
            let yard_to_open = yards[row] & !(gte1_trees & gte1_yards);

            next_trees[row] = ((trees[row] & !trees_to_yard) | open_to_trees) & COLUMNS_MASK;
            next_yards[row] = ((yards[row] & !yard_to_open) | trees_to_yard) & COLUMNS_MASK;
        }
    }

    #[inline]
    pub fn adjacent_gte1_gte3(mask: &[u64; 52], row: usize) -> (u64, u64) {
        let adjacent = [
            mask[row - 1] << 1,
            mask[row - 1],
            mask[row - 1] >> 1,
            mask[row] << 1,
            mask[row] >> 1,
            mask[row + 1] << 1,
            mask[row + 1],
            mask[row + 1] >> 1,
        ];

        let [bit0, bit1, bit2, bit3] = bitwise_count8(&adjacent);
        let gte1 = bit0 | bit1 | bit2 | bit3;
        let gte3 = (bit0 & bit1) | bit2 | bit3;

        (gte1, gte3)
    }

    #[inline]
    fn resource_value([trees, yards]: &[[u64; 52]; 2]) -> u32 {
        let tree_count = trees.iter().map(|m| m.count_ones()).sum::<u32>();
        let yard_count = yards.iter().map(|m| m.count_ones()).sum::<u32>();
        tree_count * yard_count
    }
}

examples!(Day18 -> (u32, u32) []);
