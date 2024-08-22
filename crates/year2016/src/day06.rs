use utils::prelude::*;

/// Decoding repetition code.
#[derive(Clone, Debug)]
pub struct Day06 {
    frequencies: Vec<[u32; 26]>,
}

impl Day06 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        if let Some(b) = input.bytes().find(|b| !matches!(b, b'a'..=b'z' | b'\n')) {
            return Err(InputError::new(
                input,
                b as char,
                "expected lowercase letter or newline",
            ));
        }

        let Some(length) = input.find('\n') else {
            return Err(InputError::new(input, 0, "expected at least one newline"));
        };

        let mut frequencies = vec![[0; 26]; length];
        for l in input.lines() {
            if l.len() != length {
                return Err(InputError::new(input, 0, "expected line length to match"));
            }

            for (i, c) in l.bytes().enumerate() {
                frequencies[i][(c - b'a') as usize] += 1;
            }
        }

        Ok(Self { frequencies })
    }

    #[must_use]
    pub fn part1(&self) -> String {
        // Find the most frequent letter in each column
        self.decode_message(|c| c)
    }

    #[must_use]
    pub fn part2(&self) -> String {
        // Find the least frequent letter in each column, correctly handling letters that don't
        // appear in the column (which happens in the example)
        self.decode_message(|c| if c == 0 { 0 } else { u32::MAX - c })
    }

    fn decode_message(&self, count_map_fn: impl Fn(u32) -> u32) -> String {
        self.frequencies
            .iter()
            .map(|counts| {
                (counts
                    .iter()
                    .enumerate()
                    .rev()
                    .max_by_key(|&(_, &c)| count_map_fn(c))
                    .unwrap()
                    .0 as u8
                    + b'a') as char
            })
            .collect()
    }
}

examples!(Day06 -> (&'static str, &'static str) [
    {file: "day06_example0.txt", part1: "easter", part2: "advent"},
]);
