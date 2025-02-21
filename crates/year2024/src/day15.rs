use utils::grid;
use utils::prelude::*;

/// Moving boxes around a grid.
#[derive(Clone, Debug)]
pub struct Day15<'a> {
    cols: usize,
    grid: Vec<u8>,
    robot: usize,
    moves: &'a str,
}

impl<'a> Day15<'a> {
    pub fn new(input: &'a str, _: InputType) -> Result<Self, InputError> {
        let Some((grid, moves)) = input.split_once("\n\n") else {
            return Err(InputError::new(input, 0, "expected grid and moves"));
        };

        let (rows, cols, mut grid) = grid::from_str(grid, |b| match b {
            b'.' | b'#' | b'O' | b'@' => Some(b),
            _ => None,
        })?;
        if !grid::is_enclosed(rows, cols, &grid, |&b| b == b'#') {
            return Err(InputError::new(
                input,
                0,
                "expected grid to be enclosed by walls",
            ));
        }

        let mut robots = grid.iter().enumerate().filter(|&(_, &b)| b == b'@');
        let Some((robot, _)) = robots.next() else {
            return Err(InputError::new(input, 0, "expected a robot"));
        };
        if robots.count() > 0 {
            return Err(InputError::new(input, 0, "expected only one robot"));
        }
        grid[robot] = b'.';

        if let Some(idx) = moves.find(|b| !matches!(b, '^' | 'v' | '<' | '>' | '\n')) {
            return Err(InputError::new(
                input,
                &moves[idx..],
                "expected ^, v, <, or >",
            ));
        }

        Ok(Self {
            cols,
            grid,
            robot,
            moves,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let mut grid = self.grid.clone();
        let mut robot = self.robot;

        for offset in self.moves_iterator(self.cols) {
            let next = robot.wrapping_add_signed(offset);
            if grid[next] == b'.' {
                robot = next;
            } else if grid[next] == b'O' {
                let mut next_free = next.wrapping_add_signed(offset);
                while grid[next_free] == b'O' {
                    next_free = next_free.wrapping_add_signed(offset);
                }
                if grid[next_free] == b'.' {
                    grid[next_free] = b'O';
                    grid[next] = b'.';
                    robot = next;
                }
            }
        }

        Self::sum_box_coords(&grid, self.cols, b'O')
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut grid = self
            .grid
            .iter()
            .flat_map(|&b| match b {
                b'#' => [b'#', b'#'],
                b'.' => [b'.', b'.'],
                b'O' => [b'[', b']'],
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();
        let cols = self.cols * 2;
        let mut robot = (self.robot / self.cols * cols) + (self.robot % self.cols * 2);

        for offset in self.moves_iterator(cols) {
            let next = robot.wrapping_add_signed(offset);
            if grid[next] == b'.' {
                robot = next;
            } else if (grid[next] == b'[' || grid[next] == b']')
                && Self::can_move_p2(&grid, next, offset)
            {
                Self::move_box_p2(&mut grid, next, offset);
                robot = next;
            }
        }

        Self::sum_box_coords(&grid, cols, b'[')
    }

    #[inline]
    fn moves_iterator(&self, cols: usize) -> impl Iterator<Item = isize> + use<'_> {
        self.moves.bytes().filter_map(move |c| match c {
            b'^' => Some(-(cols as isize)),
            b'v' => Some(cols as isize),
            b'<' => Some(-1),
            b'>' => Some(1),
            _ => None,
        })
    }

    #[inline]
    fn sum_box_coords(grid: &[u8], cols: usize, box_byte: u8) -> u32 {
        grid.iter()
            .enumerate()
            .map(|(i, &b)| {
                if b == box_byte {
                    100 * ((i / cols) as u32) + ((i % cols) as u32)
                } else {
                    0
                }
            })
            .sum()
    }

    fn can_move_p2(grid: &[u8], pos: usize, offset: isize) -> bool {
        let (left, right) = match grid[pos] {
            b'[' => (pos, pos + 1),
            b']' => (pos - 1, pos),
            b'.' => return true,
            b'#' => return false,
            _ => unreachable!(),
        };

        if offset == -1 || offset == 1 {
            Self::can_move_p2(grid, pos.wrapping_add_signed(offset), offset)
        } else if grid[left.wrapping_add_signed(offset)] == b'[' {
            // One box directly above/below, only need to recurse once
            Self::can_move_p2(grid, left.wrapping_add_signed(offset), offset)
        } else {
            Self::can_move_p2(grid, left.wrapping_add_signed(offset), offset)
                && Self::can_move_p2(grid, right.wrapping_add_signed(offset), offset)
        }
    }

    fn move_box_p2(grid: &mut [u8], pos: usize, offset: isize) {
        let (left, right) = match grid[pos] {
            b'[' => (pos, pos + 1),
            b']' => (pos - 1, pos),
            b'.' => return,
            _ => unreachable!(),
        };

        if offset == -1 || offset == 1 {
            Self::move_box_p2(grid, pos.wrapping_add_signed(offset * 2), offset);
        } else {
            Self::move_box_p2(grid, left.wrapping_add_signed(offset), offset);
            Self::move_box_p2(grid, right.wrapping_add_signed(offset), offset);
        }

        grid[left] = b'.';
        grid[right] = b'.';
        grid[left.wrapping_add_signed(offset)] = b'[';
        grid[right.wrapping_add_signed(offset)] = b']';
    }
}

examples!(Day15<'_> -> (u32, u32) [
    {file: "day15_example0.txt", part1: 10092, part2: 9021},
    {file: "day15_example1.txt", part1: 2028},
    {file: "day15_example2.txt", part2: 618},
]);
