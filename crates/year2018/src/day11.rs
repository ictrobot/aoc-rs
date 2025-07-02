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
        let (size, (_, x, y)) = (1..=300)
            .map(|s| (s, self.largest_total_power(s)))
            .max_by_key(|&(_, (total, _, _))| total)
            .unwrap();
        format!("{x},{y},{size}")
    }

    fn largest_total_power(&self, size: usize) -> (i32, u32, u32) {
        let (mut max_total, mut max_x, mut max_y) = (i32::MIN, 0, 0);
        for y in 0..301 - size {
            for x in 0..301 - size {
                let index = y * 301 + x;
                let total = self.summed_area_table[index]
                    + self.summed_area_table[index + 302 * size]
                    - self.summed_area_table[index + size]
                    - self.summed_area_table[index + 301 * size];
                if total > max_total {
                    max_total = total;
                    max_x = x as u32 + 1;
                    max_y = y as u32 + 1;
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
