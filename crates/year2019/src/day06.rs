use utils::parser::{ParseError, Parser};
use utils::prelude::*;

/// Counting depths and distances in a tree.
#[derive(Clone, Debug)]
pub struct Day06 {
    parents: Vec<u16>,
    depths: Vec<u16>,
    you: u16,
    san: u16,
}

const RADIX: u16 = 36;
const LABEL_SPACE: usize = (36 * 36 * 36) + (36 * 36) + (36);
const LABEL_BYTE_LUT: [Option<u16>; 256] = {
    let mut result = [None; 256];

    let mut i = 0;
    while i < 10 {
        result[(b'0' as usize) + i] = Some(i as u16);
        i += 1;
    }

    let mut i = 0;
    while i < 26 {
        result[(b'A' as usize) + i] = Some(i as u16 + 10);
        i += 1;
    }

    result
};

const fn encode(label: [u8; 3]) -> u16 {
    const fn encode_byte(b: u8) -> u16 {
        LABEL_BYTE_LUT[b as usize].unwrap()
    }
    encode_byte(label[0]) * RADIX * RADIX + encode_byte(label[1]) * RADIX + encode_byte(label[2])
}
const COM: u16 = encode(*b"COM");
const YOU: u16 = encode(*b"YOU");
const SAN: u16 = encode(*b"SAN");

impl Day06 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let label_byte = parser::byte_lut(
            &LABEL_BYTE_LUT,
            ParseError::Custom("expected uppercase letter or digit"),
        );
        let label = parser::parse_tree!((a @ label_byte) =>> {
            (b @ label_byte) =>> {
                (c @ label_byte) => a * RADIX * RADIX + b * RADIX + c,
                (parser::noop()) => (RADIX * RADIX * RADIX) + a * RADIX + b,
            },
            (parser::noop()) => (RADIX * RADIX * RADIX) + (RADIX * RADIX) + a,
        });
        let orbit = label
            .then(label.with_prefix(")"))
            .with_consumed()
            .with_eol();

        let mut label_to_dense = [u16::MAX; LABEL_SPACE];
        let mut parents = Vec::with_capacity(2048);
        for item in orbit.parse_iterator(input) {
            let ((lhs, rhs), line): ((u16, u16), &[u8]) = item?;
            if rhs == COM {
                return Err(InputError::new(
                    input,
                    line,
                    "expected COM to have no parent",
                ));
            }
            if lhs == rhs {
                return Err(InputError::new(
                    input,
                    line,
                    "expected object to not orbit itself",
                ));
            }

            let mut intern = |label| {
                let dense_index = label_to_dense[usize::from(label)];
                if dense_index != u16::MAX {
                    dense_index
                } else {
                    let next = parents.len();
                    parents.push(u16::MAX);
                    label_to_dense[usize::from(label)] = next as u16;
                    next as u16
                }
            };
            let lhs = intern(lhs);
            let rhs = intern(rhs);

            if parents[usize::from(rhs)] != u16::MAX {
                return Err(InputError::new(
                    input,
                    line,
                    "expected each object to have one parent",
                ));
            }
            parents[usize::from(rhs)] = lhs;
        }

        let com = label_to_dense[usize::from(COM)];
        if com == u16::MAX {
            return Err(InputError::new(input, 0, "expected COM object"));
        }
        let mut depths = vec![u16::MAX; parents.len()];
        depths[usize::from(com)] = 0;

        let mut stack = Vec::with_capacity(256);
        for start in 0..parents.len() {
            if depths[start] != u16::MAX {
                continue;
            }

            let mut current = start;
            while depths[current] == u16::MAX {
                stack.push(current as u16);

                if stack.len() > parents.len() {
                    return Err(InputError::new(input, 0, "expected acyclic orbit graph"));
                }

                let next = parents[current];
                if next == u16::MAX {
                    return Err(InputError::new(
                        input,
                        0,
                        "expected all objects to connect to COM",
                    ));
                }
                current = usize::from(next);
            }

            let mut depth = depths[current];
            while let Some(node) = stack.pop() {
                depth += 1;
                depths[usize::from(node)] = depth;
            }
        }

        Ok(Self {
            parents,
            depths,
            you: label_to_dense[usize::from(YOU)],
            san: label_to_dense[usize::from(SAN)],
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.depths.iter().map(|&depth| u32::from(depth)).sum()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        if self.you == u16::MAX || self.san == u16::MAX {
            panic!("expected YOU and SAN objects");
        }

        let mut a = usize::from(self.parents[self.you as usize]);
        let mut b = usize::from(self.parents[self.san as usize]);
        let mut distance = 0;
        while self.depths[a] > self.depths[b] {
            a = usize::from(self.parents[a]);
            distance += 1;
        }
        while self.depths[b] > self.depths[a] {
            b = usize::from(self.parents[b]);
            distance += 1;
        }
        while a != b {
            a = usize::from(self.parents[a]);
            b = usize::from(self.parents[b]);
            distance += 2;
        }

        distance
    }
}

examples!(Day06 -> (u32, u32) [
    {
        input: "COM)B\nB)C\nC)D\nD)E\nE)F\nB)G\nG)H\nD)I\nE)J\nJ)K\nK)L",
        part1: 42,
    },
    {
        input: "COM)B\nB)C\nC)D\nD)E\nE)F\nB)G\nG)H\nD)I\nE)J\nJ)K\nK)L\nK)YOU\nI)SAN",
        part2: 4,
    },
]);
