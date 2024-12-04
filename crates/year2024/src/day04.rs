use utils::grid;
use utils::prelude::*;

/// Counting matches in a word search.
#[derive(Clone, Debug)]
pub struct Day04 {
    rows: usize,
    cols: usize,
    grid: Vec<u8>,
}

impl Day04 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (rows, cols, grid) = grid::from_str_padded(input, 3, b'\0', |c| match c {
            b'X' | b'M' | b'A' | b'S' => Some(c),
            _ => None,
        })?;
        Ok(Self { rows, cols, grid })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let offsets = [
            self.cols as isize,
            -(self.cols as isize),
            1,
            -1,
            (self.cols as isize) + 1,
            (self.cols as isize) - 1,
            -(self.cols as isize) + 1,
            -(self.cols as isize) - 1,
        ];

        let mut count = 0;
        for r in 3..self.rows - 3 {
            for c in 3..self.cols - 3 {
                let i = r * self.cols + c;
                if self.grid[i] != b'X' {
                    continue;
                }

                for o in offsets {
                    if self.grid[i.wrapping_add_signed(o)] == b'M'
                        && self.grid[i.wrapping_add_signed(o * 2)] == b'A'
                        && self.grid[i.wrapping_add_signed(o * 3)] == b'S'
                    {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut count = 0;
        for r in 4..self.rows - 4 {
            for c in 4..self.cols - 4 {
                let i = r * self.cols + c;
                if self.grid[i] != b'A' {
                    continue;
                }

                let (nw, ne, sw, se) = (
                    self.grid[i.wrapping_add_signed(-(self.cols as isize) - 1)],
                    self.grid[i.wrapping_add_signed(-(self.cols as isize) + 1)],
                    self.grid[i.wrapping_add_signed((self.cols as isize) - 1)],
                    self.grid[i.wrapping_add_signed((self.cols as isize) + 1)],
                );

                // Given each variable is one of (b'\0', b'X', b'M', b'A', b'S') this is
                // equivalent to and slightly faster than
                //  ((nw == b'M' && se == b'S') || (nw == b'S' && se == b'M'))
                //      && ((ne == b'M' && sw == b'S') || (ne == b'S' && sw == b'M'))
                // As no other pair XORed equals b'M' ^ b'S'
                if (nw ^ se) == (b'M' ^ b'S') && (ne ^ sw) == (b'M' ^ b'S') {
                    count += 1;
                }
            }
        }
        count
    }
}

examples!(Day04 -> (u32, u32) [
    {file: "day04_example0.txt", part1: 18, part2: 9},
]);
