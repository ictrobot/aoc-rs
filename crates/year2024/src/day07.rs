use utils::prelude::*;

/// Finding operator combinations to make valid equations.
#[derive(Clone, Debug)]
pub struct Day07 {
    part1: u64,
    part2: u64,
}

impl Day07 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let parser = parser::u64()
            .with_suffix(": ")
            .then(parser::number_range(0u64..=999).repeat_arrayvec::<12, _>(b' ', 2))
            .with_suffix(parser::eol());

        let (mut part1, mut part2) = (0, 0);
        for item in parser.parse_iterator(input) {
            let (target, numbers) = item?;
            if Self::possible(target, &numbers, false) {
                part1 += target;
                part2 += target;
            } else if Self::possible(target, &numbers, true) {
                part2 += target;
            }
        }

        Ok(Self { part1, part2 })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        self.part2
    }

    #[inline]
    fn possible(target: u64, numbers: &[u64], concat: bool) -> bool {
        let (&next, numbers) = numbers.split_last().unwrap();
        if numbers.is_empty() {
            return target == next;
        }

        (target.is_multiple_of(next) && Self::possible(target / next, numbers, concat))
            || (concat && {
                // All the numbers are 1-3 digits, which makes this faster than
                //  let pow = 10u64.pow(next.ilog10() + 1);
                let pow = if next < 10 {
                    10
                } else if next < 100 {
                    100
                } else {
                    1000
                };
                target % pow == next && Self::possible(target / pow, numbers, concat)
            })
            || (target >= next && Self::possible(target - next, numbers, concat))
    }
}

examples!(Day07 -> (u64, u64) [
    {file: "day07_example0.txt", part1: 3749, part2: 11387},
]);
