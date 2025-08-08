use utils::prelude::*;

/// Decoding repetition code.
#[derive(Clone, Debug)]
pub struct Day06 {
    frequencies: Vec<[u32; 26]>,
}

impl Day06 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        if let Some(b) = input
            .bytes()
            .find(|b| !matches!(b, b'a'..=b'z' | b'\r' | b'\n'))
        {
            return Err(InputError::new(
                input,
                b as char,
                "expected lowercase letter or newline",
            ));
        }

        let mut frequencies = Vec::new();
        for l in input.lines() {
            if frequencies.is_empty() {
                frequencies = vec![[0; 26]; l.len()];
            } else if l.len() != frequencies.len() {
                return Err(InputError::new(input, 0, "expected line length to match"));
            }

            for (i, c) in l.bytes().enumerate() {
                frequencies[i][(c - b'a') as usize] += 1;
            }
        }

        if frequencies.is_empty() {
            return Err(InputError::new(input, 0, "expected at least one line"));
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
