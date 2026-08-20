use crate::knot_hash::knot_hash;
use core::fmt::NumBuffer;
use utils::bit::BitIterator;
use utils::prelude::*;

/// Finding connected regions in a hash-derived grid.
///
/// This puzzle is a combination of [`Day10`](crate::Day10), which introduced the custom knot hash
/// function used to create the grid, and [`Day12`](crate::Day12), which also involved finding
/// connected components.
#[derive(Clone, Debug)]
pub struct Day14 {
    grid: [u128; 128],
}

impl Day14 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut grid = [0u128; 128];

        let mut num_buf = NumBuffer::new();
        for (i, row) in grid.iter_mut().enumerate() {
            let suffix = i.format_into(&mut num_buf);
            let lengths = input.bytes().chain(*b"-").chain(suffix.bytes());
            *row = u128::from_be_bytes(knot_hash(lengths));
        }

        Ok(Day14 { grid })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.grid.iter().map(|x| x.count_ones()).sum()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut regions = 0;
        let mut visited = [0u128; 128];

        for (r, &row) in self.grid.iter().enumerate() {
            for (_, bit) in BitIterator::ones(row) {
                if visited[r] & bit == 0 {
                    regions += 1;
                    self.visit(&mut visited, r, bit)
                }
            }
        }

        regions
    }

    fn visit(&self, visited: &mut [u128; 128], r: usize, bit: u128) {
        visited[r] |= bit;

        if r > 0 && self.grid[r - 1] & bit != 0 && visited[r - 1] & bit == 0 {
            self.visit(visited, r - 1, bit);
        }
        if r < 127 && self.grid[r + 1] & bit != 0 && visited[r + 1] & bit == 0 {
            self.visit(visited, r + 1, bit);
        }

        let left = bit << 1;
        if left != 0 && self.grid[r] & left != 0 && visited[r] & left == 0 {
            self.visit(visited, r, left);
        }
        let right = bit >> 1;
        if right != 0 && self.grid[r] & right != 0 && visited[r] & right == 0 {
            self.visit(visited, r, right);
        }
    }
}

examples!(Day14 -> (u32, u32) [
    {input: "flqrgnkx", part1: 8108, part2: 1242},
]);
