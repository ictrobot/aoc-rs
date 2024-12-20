use std::collections::VecDeque;
use utils::grid;
use utils::prelude::*;

/// Finding shortcuts phasing through walls in a maze.
#[derive(Clone, Debug)]
pub struct Day20 {
    grid: Vec<u8>,
    distances: Vec<u32>,
    cols: usize,
}

impl Day20 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (_, cols, mut grid) = grid::from_str_padded(input, 20, b'#', |b| match b {
            b'.' | b'#' | b'S' | b'E' => Some(b),
            _ => None,
        })?;

        let mut starts = grid.iter().enumerate().filter(|(_, &b)| b == b'S');
        let Some((start, _)) = starts.next() else {
            return Err(InputError::new(input, 0, "expected one start"));
        };
        if starts.count() > 0 {
            return Err(InputError::new(input, 0, "expected one start"));
        }
        grid[start] = b'.';

        let mut ends = grid.iter().enumerate().filter(|(_, &b)| b == b'E');
        let Some((end, _)) = ends.next() else {
            return Err(InputError::new(input, 0, "expected one end"));
        };
        if ends.count() > 0 {
            return Err(InputError::new(input, 0, "expected one end"));
        }
        grid[end] = b'.';

        let mut distances = vec![u32::MAX; grid.len()];
        distances[start] = 0;
        let mut queue = VecDeque::new();
        queue.push_back((start, 0));
        while let Some((index, distance)) = queue.pop_front() {
            if index == end {
                break;
            }

            for offset in [1, cols as isize, -1, -(cols as isize)] {
                let next = index.wrapping_add_signed(offset);
                if grid[next] == b'.' && distances[next] == u32::MAX {
                    distances[next] = distance + 1;
                    queue.push_back((next, distance + 1));
                }
            }
        }

        Ok(Self {
            grid,
            distances,
            cols,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let mut cheats = 0;
        for index in (self.cols * 20)..self.grid.len() - (self.cols * 20) {
            let this_distance = self.distances[index];
            if this_distance == u32::MAX {
                continue;
            }

            for offset in [1, self.cols as isize, -1, -(self.cols as isize)] {
                if self.grid[index.wrapping_add_signed(offset)] != b'#' {
                    continue;
                }

                let Some(next) = index.checked_add_signed(offset * 2) else {
                    continue;
                };
                let Some(&target_distance) = self.distances.get(next) else {
                    continue;
                };
                if target_distance == u32::MAX || target_distance < this_distance + 2 {
                    continue;
                }
                let diff = target_distance - this_distance - 2;
                if diff >= 100 {
                    cheats += 1;
                }
            }
        }
        cheats
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut cheats = 0;
        for index in (self.cols * 20)..self.grid.len() - (self.cols * 20) {
            let this_distance = self.distances[index];
            if this_distance == u32::MAX {
                continue;
            }

            for x_offset in -20isize..=20 {
                let y_limit = 20 - x_offset.abs();
                for y_offset in -y_limit..=y_limit {
                    let cheat_length = (x_offset.unsigned_abs() + y_offset.unsigned_abs()) as u32;
                    if cheat_length == 0 {
                        continue;
                    }

                    let offset = y_offset * (self.cols as isize) + x_offset;
                    let next = index.wrapping_add_signed(offset);
                    let target_distance = self.distances[next];
                    if target_distance == u32::MAX || target_distance < this_distance + cheat_length
                    {
                        continue;
                    }

                    let diff = target_distance - this_distance - cheat_length;
                    if diff >= 100 {
                        cheats += 1;
                    }
                }
            }
        }

        cheats
    }
}

examples!(Day20 -> (u32, u32) []);
