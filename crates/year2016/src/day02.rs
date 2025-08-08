use utils::prelude::*;

/// Finding keypad codes.
#[derive(Clone, Debug)]
pub struct Day02<'a> {
    input: &'a str,
}

impl<'a> Day02<'a> {
    pub fn new(input: &'a str, _: InputType) -> Result<Self, InputError> {
        if let Some(b) = input
            .bytes()
            .find(|b| !matches!(b, b'U' | b'D' | b'L' | b'R' | b'\r' | b'\n'))
        {
            return Err(InputError::new(
                input,
                b as char,
                "expected U, D, L, R or newline",
            ));
        }

        Ok(Self { input })
    }

    #[must_use]
    pub fn part1(&self) -> String {
        let mut num = 5;
        let mut result = String::new();

        for line in self.input.lines() {
            for c in line.bytes() {
                num = match (c, num) {
                    (b'U', 4..=9) => num - 3,
                    (b'D', 1..=6) => num + 3,
                    (b'L', 2..=3 | 5..=6 | 8..=9) => num - 1,
                    (b'R', 1..=2 | 4..=5 | 7..=8) => num + 1,
                    (_, _) => num,
                };
            }
            result.push((b'0' + num) as char);
        }

        result
    }

    #[must_use]
    pub fn part2(&self) -> String {
        let mut num = 5;
        let mut result = String::new();

        for line in self.input.lines() {
            for c in line.bytes() {
                num = match (c, num) {
                    (b'U', 3) => 1,
                    (b'U', 6..=8 | 10..=12) => num - 4,
                    (b'U', 13) => 11,
                    (b'D', 1) => 3,
                    (b'D', 2..=4 | 6..=8) => num + 4,
                    (b'D', 11) => 13,
                    (b'L', 3..=4 | 6..=9 | 11..=12) => num - 1,
                    (b'R', 2..=3 | 5..=8 | 10..=11) => num + 1,
                    (_, _) => num,
                };
            }
            result.push((if num >= 10 { b'A' - 10 } else { b'0' } + num) as char);
        }

        result
    }
}

examples!(Day02<'_> -> (&'static str, &'static str) [
    {input: "ULL\nRRDDD\nLURDL\nUUUUD", part1: "1985", part2: "5DB3"},
]);
