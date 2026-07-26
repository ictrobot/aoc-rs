use utils::prelude::*;

/// Finding the missing sum.
#[derive(Clone, Debug)]
pub struct Day09 {
    numbers: Vec<u64>,
    part1: u64,
}

impl Day09 {
    pub fn new(input: &str, input_type: InputType) -> Result<Self, InputError> {
        let numbers = parser::u64().parse_lines(input)?;

        let part1 = match input_type {
            InputType::Example => Self::find_invalid::<5>(&numbers),
            InputType::Real => Self::find_invalid::<25>(&numbers),
        };
        let Some(part1) = part1 else {
            return Err(InputError::new(input, 0, "expected invalid number"));
        };

        Ok(Self { numbers, part1 })
    }

    fn find_invalid<const PREAMBLE: usize>(numbers: &[u64]) -> Option<u64> {
        numbers
            .array_windows::<PREAMBLE>()
            .zip(&numbers[PREAMBLE..])
            .find_map(|(window, &target)| {
                for i in 0..PREAMBLE - 1 {
                    for j in i + 1..PREAMBLE {
                        if window[i] + window[j] == target {
                            return None;
                        }
                    }
                }
                Some(target)
            })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        let (mut start, mut end, mut total) = (0, 2, self.numbers[0] + self.numbers[1]);
        while total != self.part1 || end - start < 2 {
            if total < self.part1 {
                if end >= self.numbers.len() {
                    panic!("no solution found")
                }
                total += self.numbers[end];
                end += 1;
            } else {
                total -= self.numbers[start];
                start += 1;
            }
        }

        let (min, max) = self.numbers[start..end]
            .iter()
            .fold((u64::MAX, 0), |(min, max), &value| {
                (min.min(value), max.max(value))
            });
        min + max
    }
}

examples!(Day09 -> (u64, u64) [
    {file: "day09_example0.txt", part1: 127, part2: 62},
]);
