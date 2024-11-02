use crate::knot_hash::{knot_hash_hex, knot_rounds};
use utils::prelude::*;

/// Implementing a custom hash function.
#[derive(Clone, Debug)]
pub struct Day10<'a> {
    input: &'a str,
}

impl<'a> Day10<'a> {
    pub fn new(input: &'a str, _: InputType) -> Result<Self, InputError> {
        // Parts 1 and 2 expect different input
        Ok(Self { input })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let lengths = parser::u8()
            .with_suffix(b','.or(parser::eof()))
            .parse_all(self.input)
            .expect("input invalid for part 1");

        let list = knot_rounds(lengths.iter().copied(), 1);

        list[0] as u32 * list[1] as u32
    }

    #[must_use]
    pub fn part2(&self) -> String {
        let hex = knot_hash_hex(self.input.bytes());

        String::from_utf8(hex.to_vec()).unwrap()
    }
}

examples!(Day10<'_> -> (u32, &'static str) [
    {input: "", part2: "a2582a3a0e66e6e86e3812dcb672a272"},
    {input: "AoC 2017", part2: "33efeb34ea91902bb2f59c9920caa6cd"},
    {input: "1,2,3", part2: "3efbe78a8d82f29979031a4aa0b16a9d"},
    {input: "1,2,4", part2: "63960835bcdc130f0b66d7ff4f6a5a8e"},
]);
