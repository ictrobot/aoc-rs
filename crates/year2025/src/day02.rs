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
        let mut streams = Vec::new();

        for &[start, end] in &self.input {
            let min_digits = start.ilog10() + 1;
            let max_digits = end.ilog10() + 1;
            for pattern_digits in 1..=max_digits / 2 {
                for digits in (min_digits.next_multiple_of(pattern_digits)..=max_digits)
                    .step_by(pattern_digits as usize)
                {
                    let repeats = digits / pattern_digits;
                    if !repeat_filter(repeats) {
                        continue;
                    }

                    let pow10 = 10u64.pow(pattern_digits);
                    let block = if digits == min_digits {
                        start / 10u64.pow(min_digits - pattern_digits)
                    } else {
                        pow10 / 10
                    };

                    streams.push(
                        RepeatingIdStream {
                            block,
                            pow10,
                            repeats,
                            range_start: start,
                            range_end: end,
                        }
                        .peekable(),
                    )
                }
            }

            let mut previous_min = None;
            loop {
                let mut min = None;

                // Advance past previous_min (if any), find the next min, and remove any empy streams
                streams.retain_mut(|s| {
                    if s.peek().copied() == previous_min {
                        let _ = s.next();
                    }

                    if let Some(&next) = s.peek() {
                        if min.is_none_or(|n| n > next) {
                            min = Some(next);
                        }
                        true
                    } else {
                        false
                    }
                });

                let Some(min) = min else {
                    break;
                };

                total += min;
                previous_min = Some(min);
            }
        }

        total
    }
}

#[derive(Clone, Debug)]
struct RepeatingIdStream {
    block: u64,
    pow10: u64,
    repeats: u32,
    range_start: u64,
    range_end: u64,
}

impl Iterator for RepeatingIdStream {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        while self.block < self.pow10 {
            let mut n = 0;
            for _ in 0..self.repeats {
                n = n * self.pow10 + self.block;
            }
            self.block += 1;

            if n > self.range_end {
                // No more solutions in this stream
                self.block = self.pow10;
                return None;
            }
            if n >= self.range_start {
                return Some(n);
            }
        }
        None
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
