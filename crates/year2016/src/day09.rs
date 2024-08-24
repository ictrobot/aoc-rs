use utils::parser::ParseError;
use utils::prelude::*;

/// Calculating decompressed length.
#[derive(Clone, Debug)]
pub struct Day09 {
    part1: u64,
    part2: u64,
}

impl Day09 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            part1: Self::decompressed_length(input.as_bytes(), false).map_with_input(input)?,
            part2: Self::decompressed_length(input.as_bytes(), true).map_with_input(input)?,
        })
    }

    fn decompressed_length(mut input: &[u8], recursive: bool) -> Result<u64, (ParseError, &[u8])> {
        let mut len = 0;

        while !input.is_empty() {
            if input[0] == b'(' {
                let (characters, repeats);
                (characters, input) = parser::u32().parse(&input[1..])?;
                (_, input) = b'x'.parse(input)?;
                (repeats, input) = parser::u32().parse(input)?;
                (_, input) = b')'.parse(input)?;

                if input.len() < characters as usize {
                    return Err((
                        ParseError::Custom("insufficient characters after marker"),
                        input,
                    ));
                }

                let repeated_len = if recursive {
                    Self::decompressed_length(&input[..characters as usize], true)?
                } else {
                    characters as u64
                };

                len += repeated_len * repeats as u64;
                input = &input[characters as usize..];
            } else {
                len += 1;
                input = &input[1..];
            }
        }

        Ok(len)
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        self.part2
    }
}

examples!(Day09 -> (u64, u64) [
    {input: "ADVENT", part1: 6},
    {input: "A(1x5)BC", part1: 7},
    {input: "(3x3)XYZ", part1: 9, part2: 9},
    {input: "A(2x2)BCD(2x2)EFG", part1: 11},
    {input: "(6x1)(1x3)A", part1: 6},
    {input: "X(8x2)(3x3)ABCY", part1: 18, part2: 20},
    {input: "(27x12)(20x12)(13x14)(7x10)(1x12)A", part2: 241920},
    {input: "(25x3)(3x3)ABC(2x3)XY(5x2)PQRSTX(18x9)(3x2)TWO(5x7)SEVEN", part2: 445},
]);
