use utils::prelude::*;

/// Finding the maximum number from ordered digits.
#[derive(Clone, Debug)]
pub struct Day03<'a> {
    lines: Vec<&'a str>,
}

impl<'a> Day03<'a> {
    pub fn new(input: &'a str, _: InputType) -> Result<Self, InputError> {
        let lines: Vec<&str> = input.lines().collect();
        for &l in &lines {
            if let Some(b) = l.bytes().find(|x| !x.is_ascii_digit()) {
                return Err(InputError::new(input, b as char, "expected digit"));
            }
            if l.len() < 12 {
                return Err(InputError::new(input, l, "expected at least 12 digits"));
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
        for l in &self.lines {
            let mut nums = [0u8; N];
            let (first, last) = l.as_bytes().split_last_chunk::<N>().unwrap();

            for &b in first {
                for j in 0..N {
                    let x = b - b'0';
                    if x > nums[j] {
                        nums[j] = x;
                        nums[j + 1..].fill(0);
                        break;
                    }
                }
            }

            for (i, &b) in last.iter().enumerate() {
                for j in i..N {
                    let x = b - b'0';
                    if x > nums[j] {
                        nums[j] = x;
                        nums[j + 1..].fill(0);
                        break;
                    }
                }
            }

            sum += nums.iter().fold(0, |acc, &x| acc * 10 + x as u64);
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
