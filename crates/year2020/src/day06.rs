use utils::prelude::*;

/// Summing the sizes of set unions and intersections.
#[derive(Clone, Debug)]
pub struct Day06 {
    part1: u32,
    part2: u32,
}

const ALL_ANSWERS: u32 = (1 << 26) - 1;
const ANSWER_BITS: [Option<u32>; 256] = {
    let mut bits = [None; 256];
    let mut i = 0u8;
    while i < 26 {
        bits[(b'a' + i) as usize] = Some(1 << i);
        i += 1;
    }
    bits
};

impl Day06 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let bytes = input.as_bytes();
        let (mut part1, mut part2) = (0, 0);
        let (mut any, mut every, mut person) = (0u32, ALL_ANSWERS, 0u32);

        for (index, &b) in bytes.iter().enumerate() {
            if let Some(answer) = ANSWER_BITS[b as usize] {
                person |= answer;
            } else if b == b'\n' {
                if person != 0 {
                    any |= person;
                    every &= person;
                    person = 0;
                } else {
                    if any == 0 {
                        return Err(InputError::new(input, index, "expected lowercase letter"));
                    }
                    part1 += any.count_ones();
                    part2 += every.count_ones();
                    (any, every) = (0, ALL_ANSWERS);
                }
            } else if !(b == b'\r' && bytes.get(index + 1) == Some(&b'\n')) {
                return Err(InputError::new(input, index, "expected lowercase letter"));
            }
        }

        if person != 0 {
            any |= person;
            every &= person;
        }

        if any == 0 {
            return Err(InputError::new(
                input,
                input.len(),
                "expected lowercase letter",
            ));
        }
        part1 += any.count_ones();
        part2 += every.count_ones();

        Ok(Self { part1, part2 })
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

examples!(Day06 -> (u32, u32) [
    {input: "abcx\nabcy\nabcz", part1: 6},
    {input: "abc\n\na\nb\nc\n\nab\nac\n\na\na\na\na\n\nb", part1: 11, part2: 6},
]);
