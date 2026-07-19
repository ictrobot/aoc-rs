use utils::prelude::*;

/// Finding pairs and triples that sum to a target value.
#[derive(Clone, Debug)]
pub struct Day01 {
    present: [bool; TARGET as usize + 1],
    values: Vec<u32>,
}

const TARGET: u32 = 2020;

impl Day01 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut present = [false; TARGET as usize + 1];
        let mut values = Vec::with_capacity(1024);

        for entry in parser::number_range(0..=TARGET)
            .with_consumed()
            .with_eol()
            .parse_iterator(input)
        {
            let (num, line) = entry?;

            if present[num as usize] {
                return Err(InputError::new(input, line, "duplicate number"));
            }
            present[num as usize] = true;

            if num < TARGET / 2 {
                values.push(num);
            }
        }
        values.sort_unstable();

        Ok(Self { present, values })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        for &a in &self.values {
            let b = TARGET - a;
            if self.present[b as usize] {
                return u64::from(a) * u64::from(b);
            }
        }
        panic!("no solution found")
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        for (i, &a) in self.values.iter().enumerate() {
            for &b in &self.values[i + 1..] {
                let c = TARGET - a - b;
                if c <= b {
                    break;
                }
                if self.present[c as usize] {
                    return u64::from(a) * u64::from(b) * u64::from(c);
                }
            }
        }
        panic!("no solution found")
    }
}

examples!(Day01 -> (u64, u64) [
    {input: "1721\n979\n366\n299\n675\n1456", part1: 514579, part2: 241861950},
]);
