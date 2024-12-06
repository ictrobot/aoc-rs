use utils::grid;
use utils::point::Point2D;
use utils::prelude::*;

/// Finding obstructions to cause infinite loops.
#[derive(Clone, Debug)]
pub struct Day06 {
    pub rows: usize,
    pub cols: usize,
    pub grid: Vec<u8>,
    pub start: Point2D<usize>,
}

const DIRECTIONS: [Point2D<isize>; 4] = [Point2D::DOWN, Point2D::RIGHT, Point2D::UP, Point2D::LEFT];

impl Day06 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (rows, cols, mut grid) = grid::from_str(input, |b| match b {
            b'.' | b'#' | b'^' => Some(b),
            _ => None,
        })?;

        let start_index = grid.iter().position(|&c| c == b'^').unwrap();
        let start = Point2D::new(start_index % cols, start_index / cols);
        grid[start_index] = b'.';

        Ok(Self {
            rows,
            cols,
            grid,
            start,
        })
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        let mut pos = self.start;
        let mut dir = 0;
        let mut visited = vec![false; self.grid.len()];
        loop {
            visited[pos.y * self.cols + pos.x] = true;

            let next = pos.wrapping_add_signed(DIRECTIONS[dir]);
            if next.x >= self.cols || next.y >= self.rows {
                break;
            }
            if self.grid[next.y * self.cols + next.x] == b'#' {
                dir = (dir + 1) % 4;
            } else {
                pos = next;
            }
        }
        visited.iter().filter(|&&c| c).count()
    }

    #[must_use]
    pub fn part2(&self) -> usize {
        let mut pos = self.start;
        let mut dir = 0;
        let mut visited = vec![0u8; self.grid.len()];
        let mut obstructions = vec![false; self.grid.len()];
        loop {
            visited[pos.y * self.cols + pos.x] |= 1 << dir;

            let next = pos.wrapping_add_signed(DIRECTIONS[dir]);
            if next.x >= self.cols || next.y >= self.rows {
                break;
            }

            if self.grid[next.y * self.cols + next.x] == b'#' {
                dir = (dir + 1) % 4;
            } else {
                if !obstructions[next.y * self.cols + next.x]
                    && visited[next.y * self.cols + next.x] == 0
                    && self.check_cycle(next, pos, dir, &visited)
                {
                    obstructions[next.y * self.cols + next.x] = true;
                }

                pos = next;
            }
        }
        obstructions.iter().filter(|&&c| c).count()
    }

    // Combination of two algorithms starting from the current position:
    // 1) Checking against previously visited states/if position leaves grid
    // 2) The start of Brent's algorithm for cycle detection as used in 2017 day 6
    // This also avoids allocating/zeroing/copying a new visited vec
    fn check_cycle(
        &self,
        obstruction: Point2D<usize>,
        pos: Point2D<usize>,
        dir: usize,
        visited: &[u8],
    ) -> bool {
        let (mut power, mut lambda) = (1, 1);
        let (mut tortoise_pos, mut tortoise_dir) = (pos, dir);
        let (mut hare_pos, mut hare_dir) = (pos, dir);

        loop {
            if power == lambda {
                tortoise_pos = hare_pos;
                tortoise_dir = hare_dir;
                power *= 2;
                lambda = 0;
            }
            lambda += 1;

            let next = hare_pos.wrapping_add_signed(DIRECTIONS[hare_dir]);
            if next.x >= self.cols || next.y >= self.rows {
                // No cycle, hare has left the grid
                return false;
            }
            if self.grid[next.y * self.cols + next.x] == b'#' || next == obstruction {
                hare_dir = (hare_dir + 1) % 4;
            } else {
                hare_pos = next;
            }

            if visited[hare_pos.y * self.cols + hare_pos.x] & (1 << hare_dir) != 0 {
                // Cycle, hare has reached a previous state from before adding the obstacle
                return true;
            }
            if hare_pos == tortoise_pos && hare_dir == tortoise_dir {
                // Cycle, hare and tortoise are in the same state
                return true;
            }
        }
    }
}

examples!(Day06 -> (usize, usize) [
    {file: "day06_example0.txt", part1: 41, part2: 6},
]);
