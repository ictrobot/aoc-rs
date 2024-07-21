use utils::prelude::*;

/// Password rules.
#[derive(Clone, Debug)]
pub struct Day11 {
    part1: [u8; 8],
    part2: [u8; 8],
}

impl Day11 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        if input.len() != 8 || input.bytes().any(|x| !x.is_ascii_lowercase()) {
            return Err(InputError::new(input, 0, "expected 8 lowercase letters"));
        }

        let input: [u8; 8] = input.as_bytes().try_into().unwrap();
        let part1 = Self::next_password(input);
        let part2 = Self::next_password(part1);
        Ok(Self { part1, part2 })
    }

    #[must_use]
    pub fn part1(&self) -> &str {
        std::str::from_utf8(&self.part1).unwrap()
    }

    #[must_use]
    pub fn part2(&self) -> &str {
        std::str::from_utf8(&self.part2).unwrap()
    }

    fn next_password(mut pass: [u8; 8]) -> [u8; 8] {
        loop {
            if pass[0] != pass[1]
                && pass[1] != pass[2]
                && pass[2] != pass[3]
                && pass[1] + 1 != pass[2]
                && pass[2] + 1 != pass[3]
            {
                // If in the first 4 characters, there are no pairs, no increasing runs, and the
                // third and forth characters can't start an increasing run, skip ahead.
                if pass[3] <= b'x' && &pass[4..] < &[pass[3], pass[3] + 1, pass[3] + 2, pass[3] + 2]
                {
                    // The final 4 characters must be "aabcc", "bbcdd", ... to contain 2 pairs and
                    // an increasing sequence.
                    pass[4] = pass[3];
                    pass[5] = pass[3] + 1;
                    pass[6] = pass[3] + 2;
                    pass[7] = pass[3] + 2;
                } else {
                    // There are no other possible matches for the current first 4 characters, so
                    // increment the first characters and reset the last 4. Setting the last 4
                    // characters to the above pattern immediately after incrementing isn't safe as
                    // the first 4 characters may now contain a pair or increasing run.
                    Self::increment(&mut pass, 3);
                    pass[4] = b'a';
                    pass[5] = b'a';
                    pass[6] = b'a';
                    pass[7] = b'a';
                }
            } else {
                Self::increment(&mut pass, 7);
            }

            if Self::valid(&pass) {
                return pass;
            }
        }
    }

    fn increment(pass: &mut [u8; 8], from: usize) {
        for i in (0..=from).rev() {
            if pass[i] == b'z' {
                pass[i] = b'a';
                // Continue to next position
            } else if pass[i] == b'h' || pass[i] == b'k' || pass[i] == b'n' {
                pass[i] += 2; // Skip confusing letters ('i', 'l', 'o')
                break;
            } else {
                pass[i] += 1;
                break;
            }
        }
    }

    fn valid(pass: &[u8; 8]) -> bool {
        Self::has_two_pairs(pass)
            && Self::has_three_run(pass)
            && Self::has_no_confusing_letters(pass)
    }

    fn has_two_pairs(x: &[u8; 8]) -> bool {
        for i in 0..7 {
            if x[i] == x[i + 1] {
                for j in i + 2..7 {
                    if x[j] == x[j + 1] {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn has_three_run(x: &[u8; 8]) -> bool {
        x.windows(3).any(|x| x[0] + 1 == x[1] && x[0] + 2 == x[2])
    }

    fn has_no_confusing_letters(x: &[u8; 8]) -> bool {
        x.iter().all(|&x| x != b'i' && x != b'o' && x != b'l')
    }
}

examples!(Day11 -> (&'static str, &'static str) [
    {input: "abcdefgh", part1: "abcdffaa"},
    {input: "ghijklmn", part1: "ghjaabcc"},
]);
