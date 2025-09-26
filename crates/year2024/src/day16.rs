use std::cmp::Reverse;
use std::collections::BinaryHeap;
use utils::grid;
use utils::prelude::*;

/// Finding the shortest paths through a maze.
#[derive(Clone, Debug)]
pub struct Day16 {
    grid: Vec<u8>,
    start: usize,
    end: usize,
    offsets: [isize; 4],
    cheapest: Vec<[u32; 4]>,
    part1: u32,
}

impl Day16 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (rows, cols, mut grid) = grid::from_str(input, |b| match b {
            b'.' | b'#' | b'S' | b'E' => Some(b),
            _ => None,
        })?;

        if !grid::is_enclosed(rows, cols, &grid, |&b| b == b'#') {
            return Err(InputError::new(
                input,
                0,
                "expected grid to be enclosed by walls",
            ));
        }

        let mut starts = grid.iter().enumerate().filter(|&(_, &b)| b == b'S');
        let Some((start, _)) = starts.next() else {
            return Err(InputError::new(input, 0, "expected one start"));
        };
        if starts.count() > 0 {
            return Err(InputError::new(input, 0, "expected one start"));
        }
        grid[start] = b'.';

        let mut ends = grid.iter().enumerate().filter(|&(_, &b)| b == b'E');
        let Some((end, _)) = ends.next() else {
            return Err(InputError::new(input, 0, "expected one end"));
        };
        if ends.count() > 0 {
            return Err(InputError::new(input, 0, "expected one end"));
        }
        grid[end] = b'.';

        let mut instance = Self {
            cheapest: vec![[u32::MAX; 4]; grid.len()],
            part1: 0,
            grid,
            start,
            end,
            offsets: [1, cols as isize, -1, -(cols as isize)],
        };

        // Precompute part 1 as dijkstra output is needed for both parts
        if !instance.dijkstra() {
            return Err(InputError::new(input, 'E', "no path"));
        }

        Ok(instance)
    }

    fn dijkstra(&mut self) -> bool {
        let mut queue = BinaryHeap::new();
        queue.push(Reverse((0, self.start, 0)));
        self.cheapest[self.start][0] = 0;

        while let Some(Reverse((score, index, dir))) = queue.pop() {
            if score > self.cheapest[index][dir] {
                continue;
            }
            if index == self.end {
                self.part1 = score;
                return true;
            }

            // Advancing to the next branch each time instead of the neighbor reduces the number
            // of items pushed to the priority queue significantly
            if let Some(branch) =
                self.find_branch(index.wrapping_add_signed(self.offsets[dir]), dir, score + 1)
            {
                // Reverse needed to use BinaryHeap as a min heap and order by the lowest score
                queue.push(Reverse(branch));
            }

            for next_dir in [(dir + 1) % 4, (dir + 3) % 4] {
                // Only turn if it will be the cheapest way to reach the turned state
                if score + 1000 < self.cheapest[index][next_dir] {
                    self.cheapest[index][next_dir] = score + 1000;

                    if let Some(branch) = self.find_branch(
                        index.wrapping_add_signed(self.offsets[next_dir]),
                        next_dir,
                        score + 1001,
                    ) {
                        queue.push(Reverse(branch));
                    }
                }
            }
        }

        false
    }

    fn find_branch(
        &mut self,
        mut index: usize,
        mut dir: usize,
        mut score: u32,
    ) -> Option<(u32, usize, usize)> {
        if self.grid[index] != b'.' {
            return None;
        }

        loop {
            if score < self.cheapest[index][dir] {
                self.cheapest[index][dir] = score;
            } else if score > self.cheapest[index][dir] {
                return None;
            }

            if index == self.end {
                break;
            }

            let mut count = 0;
            let mut next_index = 0;
            let mut next_dir = 0;
            for d in [dir, (dir + 1) % 4, (dir + 3) % 4] {
                let i = index.wrapping_add_signed(self.offsets[d]);
                if self.grid[i] == b'.' {
                    count += 1;
                    next_index = i;
                    next_dir = d;
                }
            }

            if count == 0 {
                return None;
            } else if count > 1 {
                break;
            }

            score += if dir == next_dir { 1 } else { 1001 };
            index = next_index;
            dir = next_dir;
        }

        Some((score, index, dir))
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut on_best = vec![0u8; self.grid.len()];
        on_best[self.start] = 0b1111;
        on_best[self.end] = 0b1111;
        for d in 0..4 {
            if self.cheapest[self.end][d] == self.part1 {
                let prev = self.end.wrapping_add_signed(-self.offsets[d]);
                self.reverse(prev, d, self.part1 - 1, &mut on_best);
            }
        }
        on_best.iter().filter(|&&b| b != 0).count() as u32
    }

    fn reverse(&self, index: usize, dir: usize, score: u32, on_best: &mut [u8]) {
        if on_best[index] & (1 << dir) != 0 {
            return;
        }
        on_best[index] |= 1 << dir;

        let mut count = 0;
        let mut next_index = 0;
        let mut next_dir = 0;
        for d in [dir, (dir + 1) % 4, (dir + 3) % 4] {
            let i = index.wrapping_add_signed(-self.offsets[d]);
            if self.grid[i] == b'.' {
                count += 1;
                next_index = i;
                next_dir = d;
            }
        }
        assert!(count > 0);

        if count == 1 {
            let next_score = score - if dir == next_dir { 1 } else { 1001 };
            self.reverse(next_index, next_dir, next_score, on_best);
        } else {
            // At a branch, only continue down directions where the cheapest seen score matches
            for (next_dir, next_score) in [
                (dir, score),
                ((dir + 1) % 4, score - 1000),
                ((dir + 3) % 4, score - 1000),
            ] {
                let next_index = index.wrapping_add_signed(-self.offsets[next_dir]);
                if self.cheapest[index][next_dir] == next_score
                    && self.cheapest[next_index][next_dir] == next_score - 1
                {
                    self.reverse(next_index, next_dir, next_score - 1, on_best);
                }
            }
        }
    }
}

examples!(Day16 -> (u32, u32) [
    {file: "day16_example0.txt", part1: 7036, part2: 45},
    {file: "day16_example1.txt", part1: 11048, part2: 64},
]);
