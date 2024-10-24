use utils::prelude::*;

/// Calculating spiral patterns.
#[derive(Clone, Debug)]
pub struct Day03 {
    input: u32,
}

impl Day03 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            input: parser::u32().parse_complete(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let ring = (self.input as f64).sqrt().ceil() as u32 / 2;
        let side_length = ring * 2 + 1;
        let bottom_right = side_length * side_length;
        let middles = [
            bottom_right - ring,       // Bottom
            bottom_right - (ring * 3), // Left
            bottom_right - (ring * 5), // Top
            bottom_right - (ring * 7), // Right
        ];
        let offset = middles
            .iter()
            .map(|m| m.abs_diff(self.input))
            .min()
            .unwrap();
        ring + offset
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        // To cover 0..=u32::MAX only the first 150 values in the sequence are necessary. The 150th
        // value is in the 6th ring, meaning a grid of 13x13 is required, or 15x15 with an extra
        // ring around the edge to avoid needing bounds checks.
        const LEN: usize = 15;

        let mut grid = [[0; LEN]; LEN];
        let (mut x, mut y) = (LEN / 2, LEN / 2);
        grid[x][y] = 1;

        // Store the number of turns and remaining steps until the next turn. The number of previous
        // turns gives you the current direction and next number of steps by following the pattern:
        //  1x Right
        //  1x Up
        //  2x Left
        //  2x Down
        //  3x Right
        //  3x Up
        //  4x Left
        //  ...
        let mut turns = 0;
        let mut steps = 1;

        while grid[x][y] < self.input as u64 {
            if steps == 0 {
                turns += 1;
                steps = (turns / 2) + 1;
            }
            steps -= 1;
            match turns % 4 {
                0 => x += 1,
                1 => y += 1,
                2 => x -= 1,
                3 => y -= 1,
                _ => unreachable!(),
            }

            grid[x][y] = grid[x - 1][y - 1]
                + grid[x - 1][y]
                + grid[x - 1][y + 1]
                + grid[x][y - 1]
                + grid[x][y + 1]
                + grid[x + 1][y - 1]
                + grid[x + 1][y]
                + grid[x + 1][y + 1];
        }

        grid[x][y]
    }
}

examples!(Day03 -> (u32, u64) [
    {input: "1", part1: 0},
    {input: "12", part1: 3},
    {input: "23", part1: 2},
    {input: "1024", part1: 31},
    // Custom examples
    {input: "100", part2: 122},
    {input: "200", part2: 304},
    {input: "4294967295", part2: 4429173742},
]);
