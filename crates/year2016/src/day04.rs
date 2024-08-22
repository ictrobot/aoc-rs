use utils::prelude::*;

/// Decrypting room names.
#[derive(Clone, Debug)]
pub struct Day04<'a> {
    input: Vec<(&'a [u8], u32, &'a [u8])>,
}

impl<'a> Day04<'a> {
    pub fn new(input: &'a str, _: InputType) -> Result<Self, InputError> {
        let mut rooms = parser::take_while1(|&x| matches!(x, b'a'..=b'z' | b'-'))
            .then(parser::u32())
            .then(
                parser::take_while1(u8::is_ascii_lowercase)
                    .with_prefix(b'[')
                    .with_suffix(b']'),
            )
            .parse_lines(input)?;

        rooms.retain(|&(name, _, checksum)| {
            let mut counts = [0; 26];
            for &c in name {
                if c.is_ascii_lowercase() {
                    counts[(c - b'a') as usize] += 1;
                }
            }

            for &c in checksum {
                // Find the index/letter with the highest count. max_by_key(...) returns the last
                // max element so use .rev() to get first instead, to break ties alphabetically.
                let (letter, _) = counts
                    .iter()
                    .enumerate()
                    .rev()
                    .max_by_key(|&(_, &c)| c)
                    .unwrap();

                if c != b'a' + letter as u8 {
                    return false;
                }
                counts[letter] = 0;
            }

            true
        });

        Ok(Self { input: rooms })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.input.iter().map(|&r| r.1).sum()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        const NAME: [u8; 25] = *b"northpole-object-storage-";

        self.input
            .iter()
            .find(|&&(name, sector_id, _)| {
                name.len() == NAME.len()
                    && name.iter().enumerate().all(|(i, &c)| {
                        if c == b'-' {
                            NAME[i] == b'-'
                        } else {
                            ((c - b'a' + (sector_id % 26) as u8) % 26 + b'a') == NAME[i]
                        }
                    })
            })
            .unwrap()
            .1
    }
}

examples!(Day04<'_> -> (u32, u32) [
    {
        input: "aaaaa-bbb-z-y-x-123[abxyz]\n\
            a-b-c-d-e-f-g-h-987[abcde]\n\
            not-a-real-room-404[oarel]\n\
            totally-real-room-200[decoy]",
        part1: 1514,
    },
]);
