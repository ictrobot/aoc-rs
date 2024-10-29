use std::array;
use utils::md5;
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
            .with_suffix(b','.optional())
            .parse_all(self.input)
            .expect("input invalid for part 1");

        let list = Self::knot_hash(lengths.iter().copied(), 1);

        list[0] as u32 * list[1] as u32
    }

    #[must_use]
    pub fn part2(&self) -> String {
        let lengths = self.input.bytes().chain([17, 31, 73, 47, 23]);

        let sparse = Self::knot_hash(lengths, 64);

        let dense: [u8; 16] = array::from_fn(|i| {
            sparse[16 * i..16 * (i + 1)]
                .iter()
                .fold(0, |acc, x| acc ^ x)
        });

        let dense_hex = md5::to_hex(array::from_fn(|i| {
            u32::from_be_bytes(dense[4 * i..4 * (i + 1)].try_into().unwrap())
        }));

        String::from_utf8(dense_hex.to_vec()).unwrap()
    }

    fn knot_hash(lengths: impl Iterator<Item = u8> + Clone, rounds: u32) -> [u8; 256] {
        let mut list = array::from_fn(|i| i as u8);
        let mut position = 0;
        let mut skip = 0;

        for _ in 0..rounds {
            for length in lengths.clone() {
                list[0..length as usize].reverse();
                list.rotate_left((length as usize + skip) % 256);
                position = (position + length as usize + skip) % 256;
                skip += 1;
            }
        }

        list.rotate_right(position);
        list
    }
}

examples!(Day10<'_> -> (u32, &'static str) [
    {input: "", part2: "a2582a3a0e66e6e86e3812dcb672a272"},
    {input: "AoC 2017", part2: "33efeb34ea91902bb2f59c9920caa6cd"},
    {input: "1,2,3", part2: "3efbe78a8d82f29979031a4aa0b16a9d"},
    {input: "1,2,4", part2: "63960835bcdc130f0b66d7ff4f6a5a8e"},
]);
