use utils::grid;
use utils::prelude::*;

/// Counting matches in a word search.
#[derive(Clone, Debug)]
pub struct Day04 {
    cols: usize,
    grid: Vec<u8>,
}

impl Day04 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (_, cols, grid) = grid::parse(
            input,
            3,
            0,
            |b| b,
            |b| matches!(b, b'X' | b'M' | b'A' | b'S'),
            |_, _| Err("expected 'X', 'M', 'A', 'S'"),
        )?;
        Ok(Self { cols, grid })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.check_offset(self.cols as isize)
            + self.check_offset(-(self.cols as isize))
            + self.check_offset(1)
            + self.check_offset(-1)
            + self.check_offset((self.cols as isize) + 1)
            + self.check_offset((self.cols as isize) - 1)
            + self.check_offset(-(self.cols as isize) + 1)
            + self.check_offset(-(self.cols as isize) - 1)
    }

    fn check_offset(&self, offset: isize) -> u32 {
        let start = 3 * self.cols + 3;
        let mut count = 0;
        for (((first, second), third), fourth) in self.grid[start..]
            .iter()
            .zip(&self.grid[start.wrapping_add_signed(offset)..])
            .zip(&self.grid[start.wrapping_add_signed(offset * 2)..])
            .zip(&self.grid[start.wrapping_add_signed(offset * 3)..])
        {
            count += u32::from(
                (*first == b'X') & (*second == b'M') & (*third == b'A') & (*fourth == b'S'),
            );
        }
        count
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut count = 0;
        for ((((middle, nw), ne), sw), se) in self.grid[self.cols * 4 + 4..]
            .iter()
            .zip(&self.grid[self.cols * 3 + 3..])
            .zip(&self.grid[self.cols * 3 + 5..])
            .zip(&self.grid[self.cols * 5 + 3..])
            .zip(&self.grid[self.cols * 5 + 5..])
        {
            count += u32::from(
                (*middle == b'A') & ((*nw ^ *se) == (b'M' ^ b'S')) & ((*ne ^ *sw) == (b'M' ^ b'S')),
            );
        }
        count
    }
}

examples!(Day04 -> (u32, u32) [
    {file: "day04_example0.txt", part1: 18, part2: 9},
]);
