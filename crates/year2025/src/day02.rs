use utils::prelude::*;

/// Finding numbers with repeated digit patterns within ranges.
#[derive(Clone, Debug)]
pub struct Day02 {
    input: Vec<[u64; 2]>,
}

impl Day02 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            input: parser::u64()
                .repeat_n(b'-')
                .repeat(b',', 1)
                .parse_complete(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        self.invalid_id_sum(|repeats| repeats == 2)
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        self.invalid_id_sum(|repeats| repeats > 1)
    }

    fn invalid_id_sum(&self, repeat_filter: impl Fn(u32) -> bool) -> u64 {
        let mut total = 0;
        let mut patterns = Vec::new();

        for &[start, end] in &self.input {
            let min_digits = start.checked_ilog10().unwrap_or(0) + 1;
            let max_digits = end.checked_ilog10().unwrap_or(0) + 1;
            for repeat_digits in 1..=max_digits / 2 {
                for digits in (min_digits.next_multiple_of(repeat_digits)..=max_digits)
                    .step_by(repeat_digits as usize)
                {
                    let repeats = digits / repeat_digits;
                    if !repeat_filter(repeats) {
                        continue;
                    }

                    let pow10 = 10u64.pow(repeat_digits);

                    // Mask that repeats the pattern (e.g. 1,001,001 for 3x 3 digits)
                    let repeat_mask = (0..repeats).fold(0, |acc, _| acc * pow10 + 1);

                    // Smallest number matching the repeated pattern that is >= start
                    let range_start = if digits == min_digits {
                        // Repeat the highest N digits of start, + repeat_mask if smaller than the start
                        let x = (start / 10u64.pow(min_digits - repeat_digits)) * repeat_mask;
                        if x < start { x + repeat_mask } else { x }
                    } else {
                        (pow10 / 10) * repeat_mask
                    };

                    // Largest number matching the repeated pattern that is <= end
                    let range_end = if digits == max_digits {
                        let x = (end / 10u64.pow(max_digits - repeat_digits)) * repeat_mask;
                        x.min(end)
                    } else {
                        (pow10 - 1) * repeat_mask
                    };

                    if range_start > range_end || range_end > end {
                        continue;
                    }

                    patterns.push((range_start, range_end, repeat_mask));
                }
            }

            // Merge and deduplicate multiple sequences
            while patterns.len() > 1 {
                let min = patterns.iter().map(|&(n, _, _)| n).min().unwrap();
                total += min;

                patterns.retain_mut(|(n, end, offset)| {
                    if *n == min {
                        *n += *offset;
                    }
                    *n <= *end
                });
            }

            // Use the formula for the sum of the arithmetic sequence to compute the final sequence
            if let Some((start, end, step)) = patterns.pop() {
                let n = ((end - start) / step) + 1;
                let last = start + (n - 1) * step;
                total += n * (start + last) / 2;
            }
        }

        total
    }
}

examples!(Day02 -> (u64, u64) [
    {
        input: "11-22,95-115,998-1012,1188511880-1188511890,222220-222224,\
            1698522-1698528,446443-446449,38593856-38593862,565653-565659,\
            824824821-824824827,2121212118-2121212124",
        part1: 1227775554,
        part2: 4174379265
    },
]);
