use std::collections::VecDeque;
use utils::grid;
use utils::prelude::*;

/// Finding shortcuts phasing through walls in a maze.
#[derive(Clone, Debug)]
pub struct Day20 {
    distances: Vec<u16>,
    cols: usize,
}

impl Day20 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let ((_, cols, grid), start, end) = grid::parse_maze(input, 20)?;

        let mut distances = vec![u16::MAX; grid.len()];
        distances[start] = 0;
        let mut queue = VecDeque::new();
        queue.push_back(start);
        while let Some(index) = queue.pop_front()
            && index != end
        {
            let Some(next_distance) = distances[index].checked_add(1) else {
                return Err(InputError::new(input, 0, "path too long"));
            };

            for offset in [1, cols as isize, -1, -(cols as isize)] {
                let next = index.wrapping_add_signed(offset);
                if grid[next] == b'.' && distances[next] == u16::MAX {
                    distances[next] = next_distance;
                    queue.push_back(next);
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
        let cheat_length = (x_offset.unsigned_abs() + y_offset.unsigned_abs()) as u16;
        let threshold = 101 + cheat_length;
        if cheat_length == 0 {
            return 0;
        }

        let start_index = self.cols * 20 + 20;
        let end_index = self.distances.len() - start_index;
        let offset = y_offset * (self.cols as isize) + x_offset;

        self.distances[start_index..end_index]
            .iter()
            .zip(self.distances[start_index.wrapping_add_signed(offset)..].iter())
            .map(|(&current, &target)| {
                u16::from(target.wrapping_add(1).saturating_sub(current) >= threshold)
            })
            .sum::<u16>() as u32
    }
}

examples!(Day20 -> (u32, u32) []);
