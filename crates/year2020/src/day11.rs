use utils::bit::bitwise_count8;
use utils::grid;
use utils::prelude::*;

/// Simulating a cellular automaton with line-of-sight rules.
#[derive(Clone, Debug)]
pub struct Day11 {
    cols: usize,
    seats: Vec<u128>,
}

const MAX_COLS: usize = u128::BITS as usize - 2;

impl Day11 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        // Bitmask per row with 1 bit of padding on each side, plus padding rows above and below
        let mut seats = vec![0u128];
        let (_, cols) = grid::for_each_row(
            input,
            |b| matches!(b, b'.' | b'L'),
            || "expected '.' or 'L'",
            |row, cols, row_bytes| {
                if row == 0 && cols > MAX_COLS {
                    return Err(InputError::new(
                        input,
                        row_bytes,
                        format!("expected at most {MAX_COLS} columns"),
                    ));
                }

                let bits = row_bytes
                    .iter()
                    .rfold(0u128, |acc, &b| (acc << 1) | u128::from(b == b'L'));
                seats.push(bits << 1);

                Ok(())
            },
        )?;
        seats.push(0);

        Ok(Self { cols, seats })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let rows = self.seats.len();
        let mut first = self.seats.clone();
        let mut second = vec![0u128; rows];
        let (mut grid, mut next) = (&mut first, &mut second);

        loop {
            for row in 1..rows - 1 {
                let (above, middle, below) = (grid[row - 1], grid[row], grid[row + 1]);

                let [bit0, bit1, bit2, bit3] = bitwise_count8(&[
                    above << 1,
                    above,
                    above >> 1,
                    middle << 1,
                    middle >> 1,
                    below << 1,
                    below,
                    below >> 1,
                ]);

                let none = !(bit0 | bit1 | bit2 | bit3);
                let gte4 = bit2 | bit3;
                next[row] = self.seats[row] & (none | (middle & !gte4));
            }

            if next == grid {
                return grid.iter().map(|m| m.count_ones()).sum();
            }
            (grid, next) = (next, grid);
        }
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let rows = self.seats.len();
        let col_mask = ((1u128 << self.cols) - 1) << 1;
        let floor = self
            .seats
            .iter()
            .map(|&s| col_mask & !s)
            .collect::<Vec<_>>();

        let mut first = self.seats.clone();
        let mut second = vec![0u128; rows];
        let (mut grid, mut next) = (&mut first, &mut second);
        let mut above_nw = vec![0u128; rows];
        let mut above_n = vec![0u128; rows];
        let mut above_ne = vec![0u128; rows];

        loop {
            // Iterate down the grid to find the seats visible above each row
            let (mut nw, mut n, mut ne) = (0, 0, 0);
            for row in 1..rows - 1 {
                let (above, above_floor) = (grid[row - 1], floor[row - 1]);
                nw = (above | (above_floor & nw)) << 1;
                n = above | (above_floor & n);
                ne = (above | (above_floor & ne)) >> 1;
                above_nw[row] = nw;
                above_n[row] = n;
                above_ne[row] = ne;
            }

            // Iterate back up the grid to find the seats visible below each row
            let (mut sw, mut s, mut se) = (0, 0, 0);
            for row in (1..rows - 1).rev() {
                let (below, below_floor) = (grid[row + 1], floor[row + 1]);
                sw = (below | (below_floor & sw)) << 1;
                s = below | (below_floor & s);
                se = (below | (below_floor & se)) >> 1;

                let (nw, n, ne) = (above_nw[row], above_n[row], above_ne[row]);
                let middle = grid[row];

                // Adding the occupied mask shifted 1 bit east to the floor mask carries through
                // the floor and stops at the next seat. XOR with the floor mask then sets every
                // cell that can see an occupied seat to its west.
                let w = floor[row].wrapping_add(middle << 1) ^ floor[row];

                // Carries only propagate east, so find cells that can see an occupied seat to
                // their east by repeatedly shifting the occupied mask west
                let mut e = middle >> 1;
                let mut pending = e;
                while pending != 0 {
                    pending = (pending >> 1) & (floor[row] >> 1);
                    e |= pending;
                }

                let [bit0, bit1, bit2, bit3] = bitwise_count8(&[nw, n, ne, w, e, sw, s, se]);
                let none = !(bit0 | bit1 | bit2 | bit3);
                let gte5 = bit3 | (bit2 & (bit0 | bit1));
                next[row] = self.seats[row] & (none | (middle & !gte5));
            }

            if next == grid {
                return grid.iter().map(|m| m.count_ones()).sum();
            }
            (grid, next) = (next, grid);
        }
    }
}

examples!(Day11 -> (u32, u32) [
    {file: "day11_example0.txt", part1: 37, part2: 26},
]);
