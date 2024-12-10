use utils::grid;
use utils::prelude::*;

/// Counting increasing paths through a grid.
#[derive(Clone, Debug)]
pub struct Day10 {
    rows: usize,
    cols: usize,
    grid: Vec<u32>,
}

impl Day10 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (rows, cols, grid) = grid::from_str(input, |b| match b {
            b'0'..=b'9' => Some((b - b'0') as u32),
            _ => None,
        })?;

        Ok(Self { rows, cols, grid })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let mut total = 0;
        let mut visited = vec![usize::MAX; self.grid.len()];
        for r in 0..self.rows {
            for c in 0..self.cols {
                if self.grid[r * self.cols + c] != 0 {
                    continue;
                }
                total += self.trailhead_score(r, c, 1, &mut visited, r * self.cols + c);
            }
        }
        total
    }

    fn trailhead_score(
        &self,
        r: usize,
        c: usize,
        next: u32,
        visited: &mut [usize],
        visited_key: usize,
    ) -> u32 {
        let index = r * self.cols + c;
        visited[index] = visited_key;
        if next == 10 {
            return 1;
        }

        let mut total = 0;
        if r > 0
            && self.grid[index - self.cols] == next
            && visited[index - self.cols] != visited_key
        {
            total += self.trailhead_score(r - 1, c, next + 1, visited, visited_key);
        }
        if r < self.rows - 1
            && self.grid[index + self.cols] == next
            && visited[index + self.cols] != visited_key
        {
            total += self.trailhead_score(r + 1, c, next + 1, visited, visited_key);
        }
        if c > 0 && self.grid[index - 1] == next && visited[index - 1] != visited_key {
            total += self.trailhead_score(r, c - 1, next + 1, visited, visited_key);
        }
        if c < self.cols - 1 && self.grid[index + 1] == next && visited[index + 1] != visited_key {
            total += self.trailhead_score(r, c + 1, next + 1, visited, visited_key);
        }

        total
    }

    pub fn part2(&self) -> u32 {
        let mut total = 0;
        let mut cache = vec![None; self.grid.len()];
        for r in 0..self.rows {
            for c in 0..self.cols {
                if self.grid[r * self.cols + c] != 0 {
                    continue;
                }
                total += self.count_trails(r, c, 1, &mut cache);
            }
        }
        total
    }

    fn count_trails(&self, r: usize, c: usize, next: u32, cache: &mut [Option<u32>]) -> u32 {
        let index = r * self.cols + c;
        if let Some(cache) = cache[index] {
            return cache;
        }

        if next == 10 {
            cache[index] = Some(1);
            return 1;
        }

        let mut total = 0;
        if r > 0 && self.grid[index - self.cols] == next {
            total += self.count_trails(r - 1, c, next + 1, cache);
        }
        if r < self.rows - 1 && self.grid[index + self.cols] == next {
            total += self.count_trails(r + 1, c, next + 1, cache);
        }
        if c > 0 && self.grid[index - 1] == next {
            total += self.count_trails(r, c - 1, next + 1, cache);
        }
        if c < self.cols - 1 && self.grid[index + 1] == next {
            total += self.count_trails(r, c + 1, next + 1, cache);
        }

        cache[index] = Some(total);
        total
    }
}

examples!(Day10 -> (u32, u32) [
    {file: "day10_example0.txt", part1: 36, part2: 81},
]);
