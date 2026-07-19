use utils::prelude::*;

/// Counting valid passwords.
#[derive(Clone, Debug)]
pub struct Day02 {
    part1: usize,
    part2: usize,
}

impl Day02 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let entry = parser::nonzero_u32()
            .with_suffix(b'-')
            .then(parser::nonzero_u32().with_suffix(b' '))
            .then(parser::byte_range(b'a'..=b'z').with_suffix(": "))
            .then(parser::take_while1(u8::is_ascii_lowercase))
            .with_consumed()
            .with_eol();

        let (mut part1, mut part2) = (0, 0);
        for entry in entry.parse_iterator(input) {
            let ((min, max, letter, password), line) = entry?;
            let (min, max) = (min.get() as usize, max.get() as usize);

            if min >= max {
                return Err(InputError::new(
                    input,
                    line,
                    "expected min to be less than max",
                ));
            }
            if max > password.len() {
                return Err(InputError::new(
                    input,
                    line,
                    "expected max to be less than password length",
                ));
            }

            let count = password.iter().filter(|&&b| b == letter).count();
            part1 += usize::from((min..=max).contains(&count));

            let first = password[min - 1];
            let second = password[max - 1];
            part2 += usize::from((first == letter) != (second == letter));
        }

        Ok(Self { part1, part2 })
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> usize {
        self.part2
    }
}

examples!(Day02 -> (usize, usize) [
    {input: "1-3 a: abcde\n1-3 b: cdefg\n2-9 c: ccccccccc", part1: 2, part2: 1},
]);
