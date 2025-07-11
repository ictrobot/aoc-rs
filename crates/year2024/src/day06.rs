use utils::geometry::Vec2;
use utils::grid;
use utils::prelude::*;

/// Finding obstructions to cause infinite loops.
#[derive(Clone, Debug)]
pub struct Day06 {
    pub rows: usize,
    pub cols: usize,
    pub grid: Vec<u8>,
    pub start: Vec2<usize>,
}

const DIRECTIONS: [Vec2<isize>; 4] = [Vec2::DOWN, Vec2::RIGHT, Vec2::UP, Vec2::LEFT];

impl Day06 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (rows, cols, mut grid) = grid::from_str(input, |b| match b {
            b'.' | b'#' | b'^' => Some(b),
            _ => None,
        })?;

        let start_index = grid.iter().position(|&c| c == b'^').unwrap();
        let start = Vec2::new(start_index % cols, start_index / cols);
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
        let mut cached_step_counts = vec![[0; 4]; self.grid.len()];
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
                    && self.check_cycle(next, pos, dir, &visited, &mut cached_step_counts)
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
        obstruction: Vec2<usize>,
        pos: Vec2<usize>,
        dir: usize,
        visited: &[u8],
        cache: &mut [[isize; 4]],
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

            // Advance to the next obstruction
            if hare_pos.x == obstruction.x || hare_pos.y == obstruction.y {
                // On the same X or Y line as the temporary obstruction, loop without caching
                loop {
                    let next = hare_pos.wrapping_add_signed(DIRECTIONS[hare_dir]);
                    if next.x >= self.cols || next.y >= self.rows {
                        // No cycle, hare has left the grid
                        return false;
                    }
                    if self.grid[next.y * self.cols + next.x] == b'#' || next == obstruction {
                        break;
                    }
                    hare_pos = next;
                }
            } else {
                // Temporary obstruction can be ignored as not on the same X or Y line as it
                let cached_count = &mut cache[hare_pos.y * self.cols + hare_pos.x][hare_dir];
                if *cached_count > 0 {
                    // Advanced by the previously cached count
                    hare_pos = hare_pos.wrapping_add_signed(DIRECTIONS[hare_dir] * *cached_count);
                    if hare_pos.x >= self.cols || hare_pos.y >= self.rows {
                        // No cycle, hare has left the grid
                        return false;
                    }
                } else {
                    // Loop, caching the step count until the next obstruction
                    loop {
                        let next = hare_pos.wrapping_add_signed(DIRECTIONS[hare_dir]);
                        if next.x >= self.cols || next.y >= self.rows {
                            // No cycle, hare has left the grid
                            *cached_count += 1;
                            return false;
                        }
                        if self.grid[next.y * self.cols + next.x] == b'#' {
                            break;
                        }
                        hare_pos = next;
                        *cached_count += 1;
                    }
                }
            }

            hare_dir = (hare_dir + 1) % 4;

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
