use utils::prelude::*;

/// Matching palindromes in bracketed sequences.
#[derive(Clone, Debug)]
pub struct Day07 {
    part1: u32,
    part2: u32,
}

impl Day07 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        // As the input is so large (~175KiB) do everything (validation, part 1, part 2) in 1 pass
        let mut part1_total = 0;
        let mut part2_total = 0;

        for line in input.lines().map(|l| l.as_bytes()) {
            let mut part1_valid = true;
            let mut part1_match = false;

            let mut part2_pairs = [0u8; 26 * 26];
            let mut part2_match = false;

            let mut in_brackets = false;
            for i in 0..line.len() {
                // Validation
                if line[i] == b'[' {
                    if in_brackets {
                        return Err(InputError::new(
                            input,
                            &line[i..],
                            "unexpected nested bracket",
                        ));
                    }
                    in_brackets = true;
                    continue;
                }

                if line[i] == b']' {
                    if !in_brackets {
                        return Err(InputError::new(
                            input,
                            &line[i..],
                            "unexpected close bracket",
                        ));
                    }
                    in_brackets = false;
                    continue;
                }

                if !line[i].is_ascii_lowercase() {
                    return Err(InputError::new(input, &line[i..], "unexpected character"));
                }

                // Shared
                if i + 2 >= line.len() || !line[i + 1].is_ascii_lowercase() {
                    continue; // Can't match either part
                }
                let (a, b, c) = (line[i], line[i + 1], line[i + 2]);

                // Part 1
                if let Some(&d) = line.get(i + 3)
                    && a != b
                    && a == d
                    && b == c
                {
                    if in_brackets {
                        part1_valid = false;
                    } else {
                        part1_match = true;
                    }
                }

                // Part 2
                if a != b && a == c && !part2_match {
                    let index;
                    if in_brackets {
                        index = (b - b'a') as usize * 26 + (a - b'a') as usize; // Reversed
                        part2_pairs[index] |= 1;
                    } else {
                        index = (a - b'a') as usize * 26 + (b - b'a') as usize;
                        part2_pairs[index] |= 2;
                    }
                    part2_match = part2_pairs[index] == 3;
                }
            }

            if in_brackets {
                return Err(InputError::new(
                    input,
                    &line[line.len()..],
                    "expected close bracket",
                ));
            }

            if part1_valid && part1_match {
                part1_total += 1;
            }
            if part2_match {
                part2_total += 1;
            }
        }

        Ok(Self {
            part1: part1_total,
            part2: part2_total,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.part2
    }
}

examples!(Day07 -> (u32, u32) [
    {input: "abba[mnop]qrst", part1: 1},
    {input: "abcd[bddb]xyyx", part1: 0},
    {input: "aaaa[qwer]tyui", part1: 0},
    {input: "ioxxoj[asdfgh]zxcvbn", part1: 1},
    {input: "aba[bab]xyz", part2: 1},
    {input: "xyx[xyx]xyx", part2: 0},
    {input: "aaa[kek]eke", part2: 1},
    {input: "zazbz[bzb]cdb", part2: 1},
]);
