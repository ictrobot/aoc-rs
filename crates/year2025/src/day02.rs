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
        let mut total = 0;
        for &[start, end] in &self.input {
            'num_loop: for i in start..=end {
                let num_digits = i.ilog10() + 1;

                if num_digits % 2 != 0 {
                    continue;
                }

                let pattern_digits = num_digits / 2;
                let divisor = 10u64.pow(pattern_digits);
                let pattern = i % divisor;

                let mut remaining = i;
                while remaining > 0 {
                    let chunk = remaining % divisor;
                    if chunk != pattern {
                        continue 'num_loop;
                    }
                    remaining /= divisor;
                }

                total += i;
            }
        }
        total
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        let mut total = 0;
        for &[start, end] in &self.input {
            for i in start..=end {
                let num_digits = i.ilog10() + 1;

                'pattern_loop: for pattern_digits in 1..=num_digits / 2 {
                    if num_digits % pattern_digits != 0 {
                        continue;
                    }
                    let divisor = 10u64.pow(pattern_digits);
                    let pattern = i % divisor;

                    let mut remaining = i;
                    while remaining > 0 {
                        let chunk = remaining % divisor;
                        if chunk != pattern {
                            continue 'pattern_loop;
                        }
                        remaining /= divisor;
                    }

                    total += i;
                    break;
                }
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
