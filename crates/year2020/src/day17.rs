use utils::bit::carry_save_adder;
use utils::grid;
use utils::prelude::*;

/// Simulating a cellular automaton in three and four dimensions.
///
/// The key optimization is that the initial state is at z = w = 0 and the update rule treats all
/// directions the same, so layers z and -z are always equal, as well as w and -w, and (z, w) and
/// (w, z). Therefore, only layers with z >= 0 for part 1 and 0 <= w <= z for part 2 are simulated.
#[derive(Clone, Debug)]
pub struct Day17 {
    initial: Vec<u32>,
}

const CYCLES: usize = 6;
const PADDING: usize = CYCLES + 1;
const DEPTH: usize = CYCLES + 2;
const MAX_ROWS: usize = 8;
const MAX_COLS: usize = u32::BITS as usize - 2 * CYCLES;
const HEIGHT: usize = MAX_ROWS + 2 * PADDING;
const CUBE: usize = HEIGHT * DEPTH;
const CELLS: usize = CUBE * DEPTH;

impl Day17 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut initial = Vec::with_capacity(MAX_ROWS);
        grid::for_each_row(
            input,
            |b| matches!(b, b'.' | b'#'),
            || "expected '.' or '#'",
            |row, cols, row_bytes| {
                if row == MAX_ROWS {
                    return Err(InputError::new(
                        input,
                        row_bytes,
                        format!("expected at most {MAX_ROWS} rows"),
                    ));
                }
                if row == 0 && cols > MAX_COLS {
                    return Err(InputError::new(
                        input,
                        row_bytes,
                        format!("expected at most {MAX_COLS} columns"),
                    ));
                }

                let bits = row_bytes
                    .iter()
                    .rfold(0u32, |bits, &b| (bits << 1) | u32::from(b == b'#'));
                initial.push(bits << CYCLES);
                Ok(())
            },
        )?;
        Ok(Self { initial })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let mut first = [0u32; CUBE];
        let mut second = [0u32; CUBE];
        let (mut active, mut next) = (&mut first, &mut second);

        let rows = self.initial.len();
        active[PADDING..PADDING + rows].copy_from_slice(&self.initial);

        for cycle in 1..=CYCLES {
            for z in 0..=cycle {
                for y in PADDING - cycle..PADDING + rows + cycle {
                    let index = z * HEIGHT + y;
                    let mut counts = [0u32; 4];
                    for dz in -1..=1 {
                        let plane = (z as isize + dz).unsigned_abs() * HEIGHT;
                        for dy in -1..=1 {
                            Self::add_row(&mut counts, active[plane + y.wrapping_add_signed(dy)]);
                        }
                    }
                    next[index] = Self::next_row(counts, active[index]);
                }
            }
            (active, next) = (next, active);
        }

        let mut total = 0;
        for z in 0..DEPTH {
            let multiplier = 1 + u32::from(z != 0);
            let plane = &active[z * HEIGHT..][..HEIGHT];
            total += multiplier * plane.iter().map(|row| row.count_ones()).sum::<u32>();
        }
        total
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut first = [0u32; CELLS];
        let mut second = [0u32; CELLS];
        let (mut active, mut next) = (&mut first, &mut second);

        let rows = self.initial.len();
        active[PADDING..PADDING + rows].copy_from_slice(&self.initial);

        for cycle in 1..=CYCLES {
            for w in 0..=cycle {
                for z in w..=cycle {
                    for y in PADDING - cycle..PADDING + rows + cycle {
                        let index = w * CUBE + z * HEIGHT + y;
                        let mut counts = [0u32; 4];
                        for dw in -1..=1 {
                            let nw = (w as isize + dw).unsigned_abs();
                            for dz in -1..=1 {
                                let nz = (z as isize + dz).unsigned_abs();
                                let plane = nw.min(nz) * CUBE + nw.max(nz) * HEIGHT;
                                for dy in -1..=1 {
                                    Self::add_row(
                                        &mut counts,
                                        active[plane + y.wrapping_add_signed(dy)],
                                    );
                                }
                            }
                        }
                        next[index] = Self::next_row(counts, active[index]);
                    }
                }
            }
            (active, next) = (next, active);
        }

        let mut total = 0;
        for w in 0..DEPTH {
            for z in w..DEPTH {
                let multiplier =
                    (1 + u32::from(w != 0)) * (1 + u32::from(z != 0)) * (1 + u32::from(w != z));
                let plane = &active[w * CUBE + z * HEIGHT..][..HEIGHT];
                total += multiplier * plane.iter().map(|row| row.count_ones()).sum::<u32>();
            }
        }
        total
    }

    #[inline]
    fn next_row([ones, twos, fours, gte8]: [u32; 4], current: u32) -> u32 {
        // Counts include the cell itself, so 3 is active either way and 4 only if already active
        let exactly_three = ones & twos & !(fours | gte8);
        let exactly_four = fours & !(ones | twos | gte8);
        exactly_three | (current & exactly_four)
    }

    #[inline]
    fn add_row([ones, twos, fours, gte8]: &mut [u32; 4], row: u32) {
        let (row_ones, row_twos) = carry_save_adder(row << 1, row, row >> 1);

        let (next_ones, carry) = carry_save_adder(*ones, row_ones, 0);
        let (next_twos, carry) = carry_save_adder(*twos, row_twos, carry);
        let (next_fours, overflow) = carry_save_adder(*fours, 0, carry);

        (*ones, *twos, *fours) = (next_ones, next_twos, next_fours);
        *gte8 |= overflow;
    }
}

examples!(Day17 -> (u32, u32) [
    {input: ".#.\n..#\n###", part1: 112, part2: 848},
]);
