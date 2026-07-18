use utils::bit::{bitwise_count4, bitwise_count8};
use utils::prelude::*;

/// Simulating a cellular automaton on a recursive grid.
#[derive(Clone, Debug)]
pub struct Day24 {
    initial: u32,
    recursive_minutes: usize,
}

const SIDE: usize = 5;
const TILE_COUNT: usize = SIDE * SIDE;
const MIDDLE: usize = TILE_COUNT / 2;
const GRID_MASK: u32 = (1 << TILE_COUNT) - 1;
const EXAMPLE_MINUTES: usize = 10;
const REAL_MINUTES: usize = 200;
const LEVELS: usize = 2 * REAL_MINUTES + 3;
const ROW_MASK: u32 = 0b1_1111;
const COLUMN_MASK: u32 = 0b00001_00001_00001_00001_00001;

impl Day24 {
    pub fn new(input: &str, input_type: InputType) -> Result<Self, InputError> {
        let mut initial = 0;
        let mut lines = input.lines();

        for row in 0..SIDE {
            let Some(line) = lines.next() else {
                return Err(InputError::new(input, 0, "expected exactly 5 rows"));
            };
            if line.len() != SIDE {
                return Err(InputError::new(input, line, "expected exactly 5 columns"));
            }

            for (column, &byte) in line.as_bytes().iter().enumerate() {
                match byte {
                    b'.' => {}
                    b'#' => initial |= 1 << (row * SIDE + column),
                    _ => {
                        return Err(InputError::new(
                            input,
                            &line.as_bytes()[column..column + 1],
                            "expected '.' or '#'",
                        ));
                    }
                }
            }

            if row == SIDE / 2 && Self::bug_at(initial, MIDDLE) != 0 {
                return Err(InputError::new(
                    input,
                    &line.as_bytes()[SIDE / 2..],
                    "expected '.' in the middle",
                ));
            }
        }
        if let Some(line) = lines.next() {
            return Err(InputError::new(input, line, "expected exactly 5 rows"));
        }

        Ok(Self {
            initial,
            recursive_minutes: match input_type {
                InputType::Example => EXAMPLE_MINUTES,
                InputType::Real => REAL_MINUTES,
            },
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        // Brent's algorithm for cycle detection, as used in 2017 day 6.
        // The first repeated layout is the start of the cycle, and its bitmask is its rating.
        let (mut power, mut lambda) = (1, 1);
        let mut tortoise = self.initial;
        let mut hare = Self::flat_step(self.initial);

        while tortoise != hare {
            if power == lambda {
                tortoise = hare;
                power *= 2;
                lambda = 0;
            }
            hare = Self::flat_step(hare);
            lambda += 1;
        }

        tortoise = self.initial;
        hare = self.initial;
        for _ in 0..lambda {
            hare = Self::flat_step(hare);
        }
        while tortoise != hare {
            tortoise = Self::flat_step(tortoise);
            hare = Self::flat_step(hare);
        }

        tortoise
    }

    #[inline]
    fn flat_step(bugs: u32) -> u32 {
        let [ones, twos, fours] = bitwise_count4(&Self::cardinal_neighbors(bugs));
        Self::next_layout(bugs, [ones, twos, fours, 0]) & GRID_MASK
    }

    #[inline]
    fn cardinal_neighbors(current: u32) -> [u32; 4] {
        let left = (current & !(COLUMN_MASK << (SIDE - 1))) << 1;
        let right = (current & !COLUMN_MASK) >> 1;
        let above = current << SIDE;
        let below = current >> SIDE;
        [left, right, above, below]
    }

    #[inline]
    fn next_layout(current: u32, [ones, twos, fours, eights]: [u32; 4]) -> u32 {
        let at_least_three = (ones & twos) | fours | eights;
        let exactly_one = ones & !at_least_three;
        let exactly_two = twos & !at_least_three;
        exactly_one | (!current & exactly_two)
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut first = [0; LEVELS];
        let mut second = [0; LEVELS];
        let (mut levels, mut next) = (&mut first, &mut second);

        let level0 = REAL_MINUTES + 1;
        let mut start = level0 - 1;
        let mut end = level0 + 1;

        levels[level0] = self.initial;

        for _ in 0..self.recursive_minutes {
            for (next_level, adjacent) in next[start..=end]
                .iter_mut()
                .zip(levels[start - 1..=end + 1].windows(3))
            {
                *next_level = Self::recursive_step(adjacent[0], adjacent[1], adjacent[2]);
            }

            start -= usize::from(next[start] != 0);
            end += usize::from(next[end] != 0);
            (levels, next) = (next, levels);
        }

        levels[start..=end]
            .iter()
            .copied()
            .map(u32::count_ones)
            .sum()
    }

    #[inline]
    fn recursive_step(outer: u32, current: u32, inner: u32) -> u32 {
        // Tile numbering:
        //   0  1  2  3  4
        //   5  6  7  8  9
        //  10 11 12 13 14
        //  15 16 17 18 19
        //  20 21 22 23 24
        // Current tile 12 holds the entire inner level
        // Outer tile 12 holds the entire current level.

        let [mut left, mut right, mut above, mut below] = Self::cardinal_neighbors(current);

        // Every tile in this level's left column, right column, top row or bottom row gains a
        // neighbour from a bug in tile 11, 13, 7 or 17 of outer.
        left |= Self::bug_at(outer, MIDDLE - 1) * COLUMN_MASK;
        right |= Self::bug_at(outer, MIDDLE + 1) * (COLUMN_MASK << (SIDE - 1));
        above |= Self::bug_at(outer, MIDDLE - SIDE) * ROW_MASK;
        below |= Self::bug_at(outer, MIDDLE + SIDE) * (ROW_MASK << (TILE_COUNT - SIDE));

        // Tiles 7, 11, 13 and 17 gain up to 5 neighbours each from the outside edge of inner.
        // The bits for these tiles in the above masks are always zero (as they are shifted from
        // the middle which is always empty), which allows packing the 5 neighbours into the 4
        // existing masks + 4 additional masks, so bitwise_count8 can be used.
        let [l, r, a, b] = Self::inner_edge_neighbors(inner, 0);
        left |= l;
        right |= r;
        above |= a;
        below |= b;

        let mut additional = [0; 4];
        for (mask, i) in additional.iter_mut().zip(1..SIDE) {
            let [l, r, a, b] = Self::inner_edge_neighbors(inner, i);
            *mask = l | r | a | b;
        }

        // The middle tile holds the inner level, so it is never a bug
        Self::next_layout(
            current,
            bitwise_count8(&[
                left,
                right,
                above,
                below,
                additional[0],
                additional[1],
                additional[2],
                additional[3],
            ]),
        ) & (GRID_MASK ^ (1 << MIDDLE))
    }

    // The [left, right, above, below] neighbours added by tile i of each of the inner level's edges
    #[inline]
    fn inner_edge_neighbors(inner: u32, i: usize) -> [u32; 4] {
        [
            Self::bug_at(inner, i * SIDE + SIDE - 1) << (MIDDLE + 1),
            Self::bug_at(inner, i * SIDE) << (MIDDLE - 1),
            Self::bug_at(inner, TILE_COUNT - SIDE + i) << (MIDDLE + SIDE),
            Self::bug_at(inner, i) << (MIDDLE - SIDE),
        ]
    }

    #[inline]
    fn bug_at(bugs: u32, tile: usize) -> u32 {
        (bugs >> tile) & 1
    }
}

examples!(Day24 -> (u32, u32) [
    {input: "....#\n#..#.\n#..##\n..#..\n#....", part1: 2_129_920, part2: 99},
]);
