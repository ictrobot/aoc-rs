use utils::prelude::*;

/// Finding the maximum number from ordered digits.
#[derive(Clone, Debug)]
pub struct Day03<'a> {
    lines: Vec<&'a [u8]>,
}

impl<'a> Day03<'a> {
    pub fn new(input: &'a str, _: InputType) -> Result<Self, InputError> {
        let lines: Vec<&[u8]> = input.lines().map(str::as_bytes).collect();
        for &line in &lines {
            if let Some(&b) = line.iter().find(|x| !matches!(x, b'1'..=b'9')) {
                return Err(InputError::new(input, b as char, "expected digit 1-9"));
            }
            if line.len() < 12 {
                return Err(InputError::new(input, line, "expected at least 12 digits"));
            }
        }
        Ok(Self { lines })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        self.output_joltage::<2>()
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        self.output_joltage::<12>()
    }

    fn output_joltage<const N: usize>(&self) -> u64 {
        let mut sum = 0;
        for line in &self.lines {
            let (mut start, mut joltage) = (0, 0);
            for i in 0..N {
                let (max_byte, start_offset) =
                    line[start..line.len() - N + 1 + i].iter().enumerate().fold(
                        (0, 0),
                        |(max, offset), (i, &b)| {
                            if b > max { (b, i) } else { (max, offset) }
                        },
                    );
                joltage = (joltage * 10) + (max_byte - b'0') as u64;
                start += start_offset + 1;
            }
            sum += joltage;
        }
        sum
    }
}

examples!(Day03<'_> -> (u64, u64) [
    {
        input: "987654321111111\n811111111111119\n234234234234278\n818181911112111",
        part1: 357,
        part2: 3121910778619,
    },
]);
