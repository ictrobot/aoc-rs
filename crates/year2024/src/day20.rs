use std::collections::VecDeque;
use utils::grid;
use utils::prelude::*;

/// Finding shortcuts phasing through walls in a maze.
#[derive(Clone, Debug)]
pub struct Day20 {
    distances: Vec<Distance>,
    cols: usize,
}

// Depending on AVX support, u16 or u32 can be faster
type Distance = u16;

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

        let mut distances = vec![Distance::MAX; grid.len()];
        distances[start] = 0;
        let mut queue = VecDeque::new();
        queue.push_back((start, 0 as Distance));
        while let Some((index, distance)) = queue.pop_front() {
            if index == end {
                break;
            }

            let Some(next_distance) = distance.checked_add(1) else {
                return Err(InputError::new(input, 0, "path too long"));
            };

            for offset in [1, cols as isize, -1, -(cols as isize)] {
                let next = index.wrapping_add_signed(offset);
                if grid[next] == b'.' && distances[next] == Distance::MAX {
                    distances[next] = next_distance;
                    queue.push_back((next, next_distance));
                }
            }
        }

        Ok(Self { distances, cols })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        [(0, -2), (2, 0), (0, 2), (-2, 0)]
            .into_iter()
            .map(|(x, y)| self.cheat_count(x, y))
            .sum()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut total = 0;
        for x_offset in -20isize..=20 {
            let y_limit = 20 - x_offset.abs();
            for y_offset in -y_limit..=y_limit {
                total += self.cheat_count(x_offset, y_offset);
            }
        }
        total
    }

    fn cheat_count(&self, x_offset: isize, y_offset: isize) -> u32 {
        let cheat_length = (x_offset.unsigned_abs() + y_offset.unsigned_abs()) as Distance;
        if cheat_length == 0 {
            return 0;
        }

        let mut cheats = 0;
        let offset = y_offset * (self.cols as isize) + x_offset;
        for (index, target) in (self.cols * 20..self.distances.len() - (self.cols * 20))
            .zip((self.cols * 20).wrapping_add_signed(offset)..)
        {
            let this_distance = self.distances[index];
            let target_distance = self.distances[target];
            cheats += u32::from(
                (target_distance != Distance::MAX)
                    & (this_distance != Distance::MAX)
                    & (target_distance > this_distance.wrapping_add(cheat_length))
                    & (target_distance
                        .wrapping_sub(this_distance)
                        .wrapping_sub(cheat_length)
                        >= 100),
            );
        }
        cheats
    }
}

examples!(Day20 -> (u32, u32) []);
