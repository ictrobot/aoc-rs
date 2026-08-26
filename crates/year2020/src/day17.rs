use utils::bit::carry_save_adder;
use utils::grid;
use utils::prelude::*;

/// Simulating a cellular automaton in three and four dimensions.
///
/// The key optimization is that the initial state is at z = w = 0 and the update rule treats all
/// directions the same, so layers z and -z are always equal, as well as w and -w, and (z, w) and
/// (w, z). Therefore, only layers with z >= 0 for part 1 and 0 <= w <= z for part 2 are simulated.
///
/// Each cycle first counts the neighbours of each cell within its own layer, then adds the counts
/// from the adjacent w and z layers to get each cell's total.
#[derive(Clone, Debug)]
pub struct Day17 {
    initial: Vec<u32>,
}

const CYCLES: usize = 6;
const PADDING: usize = CYCLES + 1;
const DEPTH: usize = CYCLES + 2;
const MAX_ROWS: usize = 8;
const MAX_COLS: usize = u32::BITS as usize - 2 * CYCLES;
// Rounding up to a multiple of 8/16 rows helps with vectorization in AVX2/AVX512 builds
const INNER_ROWS: usize = (MAX_ROWS + 2 * CYCLES).next_multiple_of(cfg_select! {
    target_feature = "avx512f" => 16,
    target_feature = "avx2" => 8,
    _ => 1,
});
const HEIGHT: usize = INNER_ROWS + 2;
const LAYERS: usize = DEPTH * (DEPTH + 1) / 2;

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
        let mut first = [[0u32; HEIGHT]; DEPTH];
        let mut second = [[0u32; HEIGHT]; DEPTH];
        let mut layer_counts = [NeighbourCounts::default(); DEPTH];
        let (mut current, mut next) = (&mut first, &mut second);

        let rows = self.initial.len();
        current[0][PADDING..PADDING + rows].copy_from_slice(&self.initial);

        for cycle in 1..=CYCLES {
            // Only layers below cycle can contain active cells
            for z in 0..cycle {
                layer_counts[z] = NeighbourCounts::within_layer(&current[z]);
            }

            // Add the counts from the adjacent z layers to get each cell's total
            for z in 0..=cycle {
                let adjacent = Self::adjacent(z).map(|z| &layer_counts[z]);
                next[z] = Self::next_layer(&current[z], adjacent);
            }

            (current, next) = (next, current);
        }

        // Each layer other than z = 0 also counts for the equal -z layer
        let mut total = 0;
        for (z, layer) in current.iter().enumerate().take(CYCLES + 1) {
            let multiplier = 1 + u32::from(z != 0);
            total += multiplier * layer.iter().map(|row| row.count_ones()).sum::<u32>();
        }
        total
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut first = [[0u32; HEIGHT]; LAYERS];
        let mut second = [[0u32; HEIGHT]; LAYERS];
        let mut layer_counts = [NeighbourCounts::default(); LAYERS];
        let mut z_sums = [[NeighbourCounts::default(); DEPTH]; DEPTH];
        let (mut current, mut next) = (&mut first, &mut second);

        let rows = self.initial.len();
        current[0][PADDING..PADDING + rows].copy_from_slice(&self.initial);

        for cycle in 1..=CYCLES {
            // Only layers with w <= z < cycle can contain active cells
            for w in 0..cycle {
                for z in w..cycle {
                    let layer = Self::layer_index(w, z);
                    layer_counts[layer] = NeighbourCounts::within_layer(&current[layer]);
                }
            }

            // Add the counts from the adjacent z layers
            for (w, sums) in z_sums.iter_mut().enumerate().take(cycle) {
                for (z, sum) in sums[..cycle]
                    .iter_mut()
                    .enumerate()
                    .skip(w.saturating_sub(1))
                {
                    let adjacent = Self::adjacent(z).map(|z| Self::layer_index(w, z));
                    *sum = NeighbourCounts::sum(adjacent.map(|layer| &layer_counts[layer]));
                }

                // Only the layer below the new layer can contain active cells
                sums[cycle] = layer_counts[Self::layer_index(w, cycle - 1)];
            }

            // Add the counts from the adjacent w layers to get each cell's total
            for w in 0..=cycle {
                let [inner, middle, outer] = Self::adjacent(w).map(|w| &z_sums[w]);
                for z in w..=cycle {
                    let layer = Self::layer_index(w, z);
                    let adjacent = [&inner[z], &middle[z], &outer[z]];
                    next[layer] = Self::next_layer(&current[layer], adjacent);
                }
            }
            (current, next) = (next, current);
        }

        // Each (w, z) layer also counts for the equal layers with -w, -z, and w and z swapped
        let mut total = 0;
        for w in 0..=CYCLES {
            for z in w..=CYCLES {
                let multiplier =
                    (1 + u32::from(w != 0)) * (1 + u32::from(z != 0)) * (1 + u32::from(w != z));
                let layer = &current[Self::layer_index(w, z)];
                total += multiplier * layer.iter().map(|row| row.count_ones()).sum::<u32>();
            }
        }
        total
    }

    fn next_layer(current: &[u32; HEIGHT], adjacent: [&NeighbourCounts; 3]) -> [u32; HEIGHT] {
        let mut next = [0; HEIGHT];
        for row in 0..INNER_ROWS {
            let [ones, twos, fours] = NeighbourCounts::sum_row(adjacent, row);

            // Counts include the cell itself, so 3 is always active and 4 only if already active
            let exactly_three = ones & twos & !fours;
            let exactly_four = fours & !(ones | twos);
            next[row + 1] = exactly_three | (current[row + 1] & exactly_four);
        }
        next
    }

    #[inline]
    fn adjacent(z: usize) -> [usize; 3] {
        if z == 0 { [1, 0, 1] } else { [z - 1, z, z + 1] }
    }

    #[inline]
    fn layer_index(w: usize, z: usize) -> usize {
        let (w, z) = if w < z { (w, z) } else { (z, w) };
        z * (z + 1) / 2 + w
    }
}

// Saturating neighbour counts for the inner rows from one layer, stored as three arrays so the row
// loops can be vectorized by the compiler. Also aligned to 64 bytes when using 256+ bit AVX vectors
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(target_feature = "avx2", repr(align(64)))]
struct NeighbourCounts {
    ones: [u32; INNER_ROWS],
    twos: [u32; INNER_ROWS],
    fours: [u32; INNER_ROWS],
}

impl NeighbourCounts {
    // Counts each cell's neighbours within the layer, including itself
    #[inline]
    fn within_layer(layer: &[u32; HEIGHT]) -> Self {
        let mut counts = Self::default();
        let mut above = Self::within_row(layer[0]);
        let mut middle = Self::within_row(layer[1]);
        for r in 0..INNER_ROWS {
            let below = Self::within_row(layer[r + 2]);
            [counts.ones[r], counts.twos[r], counts.fours[r]] = Self::add(above, middle, below);
            (above, middle) = (middle, below);
        }
        counts
    }

    #[inline]
    fn within_row(row: u32) -> [u32; 3] {
        let (ones, twos) = carry_save_adder(row << 1, row, row >> 1);
        [ones, twos, 0]
    }

    // Adds the counts from three adjacent layers
    #[inline]
    fn sum(adjacent: [&Self; 3]) -> Self {
        let mut counts = Self::default();
        for r in 0..INNER_ROWS {
            [counts.ones[r], counts.twos[r], counts.fours[r]] = Self::sum_row(adjacent, r);
        }
        counts
    }

    // Adds one row of the counts from three adjacent layers
    #[inline]
    fn sum_row([a, b, c]: [&Self; 3], row: usize) -> [u32; 3] {
        Self::add(
            [a.ones[row], a.twos[row], a.fours[row]],
            [b.ones[row], b.twos[row], b.fours[row]],
            [c.ones[row], c.twos[row], c.fours[row]],
        )
    }

    #[inline]
    fn add([a1, a2, a4]: [u32; 3], [b1, b2, b4]: [u32; 3], [c1, c2, c4]: [u32; 3]) -> [u32; 3] {
        let (ones, carry2) = carry_save_adder(a1, b1, c1);
        let (twos, carry4a) = carry_save_adder(a2, b2, c2);
        let (twos, carry4b) = carry_save_adder(twos, carry2, 0);
        let (fours, carry8a) = carry_save_adder(a4, b4, c4);
        let (fours, carry8b) = carry_save_adder(fours, carry4a, carry4b);

        // carry8 means the value is at least 8, which is stored as 7
        let eights = carry8a | carry8b;

        [ones | eights, twos | eights, fours | eights]
    }
}

examples!(Day17 -> (u32, u32) [
    {input: ".#.\n..#\n###", part1: 112, part2: 848},
]);
