use utils::prelude::*;

/// Simulating water flow in a 2D grid.
#[derive(Clone, Debug)]
pub struct Day17 {
    part1: u32,
    part2: u32,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Empty,
    Flowing,
    Filled,
}

impl Day17 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let segments = parser::byte_range(b'x'..=b'y')
            .with_suffix(b'=')
            .then(parser::number_range(1..=2999).with_suffix(", "))
            .then(parser::byte_range(b'x'..=b'y').with_suffix(b'='))
            .then(parser::number_range(1..=2999).with_suffix(".."))
            .then(parser::number_range(1..=2999))
            .map_res(|(axis1, c1, axis2, c2, c3)| {
                if axis1 == axis2 {
                    Err("expected line segment")
                } else if c2 > c3 {
                    Err("expected range to be sorted")
                } else {
                    Ok((axis1, c1, c2, c3))
                }
            })
            .parse_lines(input)?;

        if segments.is_empty() {
            return Err(InputError::new(input, 0, "expected at least one line"));
        }

        let (mut x_min, mut x_max, mut y_min, mut y_max) = (500, 500, usize::MAX, 0);
        for &(axis, c1, c2, c3) in &segments {
            if axis == b'x' {
                x_min = x_min.min(c1);
                x_max = x_max.max(c1);
                y_min = y_min.min(c2);
                y_max = y_max.max(c3);
            } else {
                x_min = x_min.min(c2);
                x_max = x_max.max(c3);
                y_min = y_min.min(c1);
                y_max = y_max.max(c1);
            }
        }

        // Reserve the top row for the spring
        y_min -= 1;
        // Padding to avoid wrapping around rows
        x_min -= 1;
        x_max += 1;

        let width = x_max - x_min + 1;
        let height = y_max - y_min + 1;
        let mut grid = vec![State::Empty; width * height];
        for &(axis, c1, c2, c3) in &segments {
            if axis == b'x' {
                let x = c1 - x_min;
                for y in c2 - y_min..=c3 - y_min {
                    grid[y * width + x] = State::Filled;
                }
            } else {
                let y = c1 - y_min;
                for x in c2 - x_min..=c3 - x_min {
                    grid[y * width + x] = State::Filled;
                }
            }
        }

        let mut counts = [0; 3];
        Self::flow(&mut grid, width, &mut counts, 500 - x_min);

        Ok(Self {
            part1: counts[State::Filled as usize] + counts[State::Flowing as usize],
            part2: counts[State::Filled as usize],
        })
    }

    fn flow(grid: &mut [State], width: usize, counts: &mut [u32; 3], index: usize) -> State {
        if index >= grid.len() {
            return State::Flowing;
        }
        if grid[index] != State::Empty {
            return grid[index];
        }

        if Self::flow(grid, width, counts, index + width) == State::Flowing {
            grid[index] = State::Flowing;
            if index >= width {
                counts[State::Flowing as usize] += 1;
            }
            return State::Flowing;
        }

        let mut left = index;
        while grid[left - 1] == State::Empty {
            left -= 1;
            if Self::flow(grid, width, counts, left + width) == State::Flowing {
                break;
            }
        }

        let mut right = index;
        while grid[right + 1] == State::Empty {
            right += 1;
            if Self::flow(grid, width, counts, right + width) == State::Flowing {
                break;
            }
        }

        let state = if grid[left - 1] == State::Filled && grid[right + 1] == State::Filled {
            State::Filled
        } else {
            State::Flowing
        };

        grid[left..=right].fill(state);
        if index >= width {
            counts[state as usize] += (right - left + 1) as u32;
        }
        state
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.part2
    }
}

examples!(Day17 -> (u32, u32) [
    {
        input: "x=495, y=2..7\n\
            y=7, x=495..501\n\
            x=501, y=3..7\n\
            x=498, y=2..4\n\
            x=506, y=1..2\n\
            x=498, y=10..13\n\
            x=504, y=10..13\n\
            y=13, x=498..504",
        part1: 57,
        part2: 29,
    },
]);
