use utils::grid::parse;
use utils::prelude::*;

/// Counting splitting paths in a grid.
#[derive(Clone, Debug)]
pub struct Day07 {
    cols: usize,
    grid: Vec<u8>,
    start: usize,
}

impl Day07 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut start = None;
        let (_rows, cols, grid) = parse(
            input,
            1,
            b'.',
            |b| b,
            |b| matches!(b, b'.' | b'^'),
            |i, b| {
                match b {
                    b'S' if start.is_none() => start = Some(i),
                    b'S' => return Err("expected one 'S'"),
                    _ => return Err("expected '.', '^' or 'S'"),
                }
                Ok(b'.')
            },
        )?;
        let Some(start) = start else {
            return Err(InputError::new(input, 0, "expected one 'S'"));
        };
        Ok(Self { cols, grid, start })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        let mut grid = self.grid.clone();
        let mut split_count = 0;
        self.split_beams(&mut grid, &mut split_count, self.start + self.cols);
        split_count
    }

    fn split_beams(&self, grid: &mut [u8], count: &mut u64, mut pos: usize) {
        while pos < grid.len() && grid[pos] == b'.' {
            grid[pos] = b'|';
            pos += self.cols;
        }
        if pos >= grid.len() || grid[pos] != b'^' {
            return;
        }

        *count += 1;

        let left = pos - 1;
        if grid[left] == b'.' {
            self.split_beams(grid, count, left);
        }

        let right = pos + 1;
        if grid[right] == b'.' {
            self.split_beams(grid, count, right);
        }
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        let mut timeline_cache = vec![0u64; self.grid.len()];
        self.explore_timelines(&mut timeline_cache, self.start)
    }

    fn explore_timelines(&self, timeline_cache: &mut [u64], mut pos: usize) -> u64 {
        if timeline_cache[pos] > 0 {
            return timeline_cache[pos];
        }
        let start = pos;

        while pos < self.grid.len() && self.grid[pos] == b'.' {
            pos += self.cols;
        }
        if pos >= self.grid.len() || self.grid[pos] != b'^' {
            timeline_cache[start] = 1;
            return 1;
        }

        if timeline_cache[pos] > 0 {
            timeline_cache[start] = timeline_cache[pos];
            return timeline_cache[pos];
        }

        let timelines = self.explore_timelines(timeline_cache, pos - 1)
            + self.explore_timelines(timeline_cache, pos + 1);
        timeline_cache[pos] = timelines;
        timeline_cache[start] = timelines;
        timelines
    }
}

examples!(Day07 -> (u64, u64) [
    {file: "day07_example0.txt", part1: 21, part2: 40},
]);
