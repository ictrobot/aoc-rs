use utils::prelude::*;

/// Locating the optimal square subregion.
///
/// See <https://en.wikipedia.org/wiki/Summed-area_table>, which allows computing the sum of any
/// square in constant time with only 4 lookups.
#[derive(Clone, Debug)]
pub struct Day11 {
    summed_area_table: Vec<i32>,
}

impl Day11 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let serial_num = parser::number_range(0..=9999).parse_complete(input)?;

        let mut table = vec![0i32; 301 * 301];
        for y in 1..301 {
            for x in 1..301 {
                let rack_id = x + 10;
                let power_level = ((rack_id * y + serial_num) * rack_id) / 100 % 10 - 5;
                let index = (y * 301 + x) as usize;
                table[index] =
                    power_level + table[index - 1] + table[index - 301] - table[index - 302];
            }
        }

        Ok(Self {
            summed_area_table: table,
        })
    }

    #[must_use]
    pub fn part1(&self) -> String {
        let (_, x, y) = self.largest_total_power(3);
        format!("{x},{y}")
    }

    #[must_use]
    pub fn part2(&self) -> String {
        let (mut max_size, mut max_total, mut max_x, mut max_y) = (0, i32::MIN, 0, 0);
        let mut sizes: Vec<i32> = Vec::with_capacity(300);

        for size in 1..=300 {
            // Try to split the N*N square into Y X*X squares to calculate an upper bound.
            // For example, if the best 5x5 is 100, then the best 10x10 must be <= 400.
            if let Some(divisor) = (2..=size / 2).rev().find(|&d| size % d == 0) {
                let copies = (size / divisor) * (size / divisor);
                let upper_bound = sizes[divisor - 1].saturating_mul(copies as i32);
                if upper_bound < max_total {
                    sizes.push(upper_bound);
                    continue;
                }
            };

            let (total, x, y) = self.largest_total_power(size);
            sizes.push(total);

            if total > max_total {
                max_size = size;
                max_total = total;
                max_x = x;
                max_y = y;
            }
        }

        format!("{max_x},{max_y},{max_size}")
    }

    fn largest_total_power(&self, size: usize) -> (i32, u32, u32) {
        let (mut max_total, mut max_x, mut max_y) = (i32::MIN, 0, 0);
        let mut row_totals = [0; 301];

        for y in 0..301 - size {
            // Avoids bounds checks, allowing the inner loop to be vectorized
            let mut found_new_max = false;
            for ((((total, &top_left), &top_right), &bottom_left), &bottom_right) in row_totals
                [..301 - size]
                .iter_mut()
                .zip(self.summed_area_table[y * 301..].iter())
                .zip(self.summed_area_table[y * 301 + size..].iter())
                .zip(self.summed_area_table[(y + size) * 301..].iter())
                .zip(self.summed_area_table[(y + size) * 301 + size..].iter())
            {
                *total = top_left + bottom_right - top_right - bottom_left;
                found_new_max |= *total > max_total;
            }

            // Only perform scalar comparisons when a new max has been found
            if found_new_max {
                for (x, &total) in row_totals[..301 - size].iter().enumerate() {
                    if total > max_total {
                        max_total = total;
                        max_x = x as u32 + 1;
                        max_y = y as u32 + 1;
                    }
                }
            }
        }
        (max_total, max_x, max_y)
    }
}

examples!(Day11 -> (&'static str, &'static str) [
    {input: "18", part1: "33,45", part2: "90,269,16"},
    {input: "42", part1: "21,61", part2: "232,251,12"},
]);
