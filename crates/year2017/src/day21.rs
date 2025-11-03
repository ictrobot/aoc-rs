use std::array;
use utils::parser::{LeafResult, ParseError};
use utils::prelude::*;

/// Iteratively applying pixel transformations.
///
/// The key optimization is that every three iterations a 3x3 square splits into nine independent
/// 3x3 squares. Therefore, we can store counts for each of the 512 unique 3x3 squares and simulate
/// three iterations at a time for each square to find the counts for the following three iterations
#[derive(Clone, Debug)]
pub struct Day21 {
    part1: u32,
    part2: u32,
}

#[derive(Clone, Debug)]
enum Rule {
    TwoToThree(usize, usize),
    ThreeToFour(usize, usize),
}

const START_3X3: usize = 0b010_001_111;

impl Day21 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let parse_iterator = parser::one_of((
            Self::parse_square::<3>
                .with_suffix(" => ")
                .then(Self::parse_square::<4>)
                .map(|(a, b)| Rule::ThreeToFour(a, b)),
            Self::parse_square::<2>
                .with_suffix(" => ")
                .then(Self::parse_square::<3>)
                .map(|(a, b)| Rule::TwoToThree(a, b)),
        ))
        .with_eol()
        .parse_iterator(input);

        let mut rules_2x2_to_3x3 = [0usize; 16];
        let mut rules_3x3_to_4x4 = [0usize; 512];
        for item in parse_iterator {
            match item? {
                Rule::TwoToThree(two, three) => {
                    for transformed in Self::transformations_2x2(two) {
                        rules_2x2_to_3x3[transformed] = three;
                    }
                }
                Rule::ThreeToFour(three, four) => {
                    for transformed in Self::transformations_3x3(three) {
                        rules_3x3_to_4x4[transformed] = four;
                    }
                }
            }
        }

        let (mut three_counts, mut three_counts_next) = ([0; 512], [0; 512]);
        three_counts[START_3X3] = 1;

        let mut on_pixels = [0; 21];
        for i in (0..=18).step_by(3) {
            for (three, &count) in three_counts.iter().enumerate().filter(|&(_, &c)| c > 0) {
                on_pixels[i] += three.count_ones() * count;

                let four = rules_3x3_to_4x4[three];
                on_pixels[i + 1] += four.count_ones() * count;

                let six = Self::combine_3x3_into_6x6(
                    Self::split_4x4_into_2x2(four).map(|two| rules_2x2_to_3x3[two]),
                );
                on_pixels[i + 2] += six.count_ones() * count;

                for two in Self::split_6x6_into_2x2(six) {
                    let next = rules_2x2_to_3x3[two];
                    three_counts_next[next] += count;
                }
            }

            three_counts = three_counts_next;
            three_counts_next.fill(0);
        }

        Ok(Self {
            part1: on_pixels[5],
            part2: on_pixels[18],
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

    #[inline]
    fn parse_square<const N: usize>(mut input: &[u8]) -> LeafResult<'_, usize> {
        let mut result = 0;
        for row in 0..N {
            for col in 0..N {
                match input.first() {
                    Some(b'.') => {}
                    Some(b'#') => result |= 1 << (col + N * row),
                    _ => return Err((ParseError::Custom("expected '.' or '#'"), input)),
                }
                input = &input[1..];
            }

            if row < N - 1 {
                if input.first().copied() != Some(b'/') {
                    return Err((ParseError::Custom("expected '/'"), input));
                }
                input = &input[1..];
            }
        }

        Ok((result, input))
    }

    #[inline]
    #[expect(clippy::identity_op)]
    fn transformations_2x2(square: usize) -> [usize; 4] {
        let b: [_; 4] = array::from_fn(|i| (square >> i) & 1);

        [
            // | Original      | 90° rotation  | 180° rotation | 270° rotation |
            // | 01            | 20            | 32            | 13            |
            // | 23            | 31            | 10            | 02            |
            square,
            (b[2] << 0) | (b[0] << 1) | (b[3] << 2) | (b[1] << 3),
            (b[3] << 0) | (b[2] << 1) | (b[1] << 2) | (b[0] << 3),
            (b[1] << 0) | (b[3] << 1) | (b[0] << 2) | (b[2] << 3),
        ]
    }

    #[inline]
    #[rustfmt::skip]
    #[expect(clippy::identity_op)]
    fn transformations_3x3(square: usize) -> [usize; 8] {
        let b: [_; 9] = array::from_fn(|i| (square >> i) & 1);

        [
            // | Original      | 90° rotation  | 180° rotation | 270° rotation |
            // | 012           | 630           | 876           | 258           |
            // | 345           | 741           | 543           | 147           |
            // | 678           | 852           | 210           | 036           |
            square,
            (b[6] << 0) | (b[3] << 1) | (b[0] << 2) | (b[7] << 3) | (b[4] << 4) | (b[1] << 5) | (b[8] << 6) | (b[5] << 7) | (b[2] << 8),
            (b[8] << 0) | (b[7] << 1) | (b[6] << 2) | (b[5] << 3) | (b[4] << 4) | (b[3] << 5) | (b[2] << 6) | (b[1] << 7) | (b[0] << 8),
            (b[2] << 0) | (b[5] << 1) | (b[8] << 2) | (b[1] << 3) | (b[4] << 4) | (b[7] << 5) | (b[0] << 6) | (b[3] << 7) | (b[6] << 8),
            // | Flipped       | 90° rotation  | 180° rotation | 270° rotation |
            // | 678           | 036           | 210           | 852           |
            // | 345           | 147           | 543           | 741           |
            // | 012           | 258           | 876           | 630           |
            (b[6] << 0) | (b[7] << 1) | (b[8] << 2) | (b[3] << 3) | (b[4] << 4) | (b[5] << 5) | (b[0] << 6) | (b[1] << 7) | (b[2] << 8),
            (b[0] << 0) | (b[3] << 1) | (b[6] << 2) | (b[1] << 3) | (b[4] << 4) | (b[7] << 5) | (b[2] << 6) | (b[5] << 7) | (b[8] << 8),
            (b[2] << 0) | (b[1] << 1) | (b[0] << 2) | (b[5] << 3) | (b[4] << 4) | (b[3] << 5) | (b[8] << 6) | (b[7] << 7) | (b[6] << 8),
            (b[8] << 0) | (b[5] << 1) | (b[2] << 2) | (b[7] << 3) | (b[4] << 4) | (b[1] << 5) | (b[6] << 6) | (b[3] << 7) | (b[0] << 8),
        ]
    }

    #[inline]
    fn split_4x4_into_2x2(square: usize) -> [usize; 4] {
        // [ 0]   1  [ 2]   3     0  0  1  1
        //   4    5    6    7     0  0  1  1
        // [ 8]   9  [10]  11     2  2  3  3
        //  12   13   14   15     2  2  3  3
        [0, 2, 8, 10].map(|i| ((square >> i) & 0b11) | (((square >> (i + 4)) & 0b11) << 2))
    }

    #[inline]
    fn combine_3x3_into_6x6(squares: [usize; 4]) -> usize {
        // [ 0]   1    2  [ 3]   4    5     [0] 0-2  [1] 0-2
        //   6    7    8    9   10   11     [0] 3-5  [1] 3-5
        //  12   13   14   15   16   17     [0] 6-8  [1] 6-8
        // [18]  19   20  [21]  22   23     [2] 0-2  [3] 0-2
        //  24   25   26   27   28   29     [2] 3-5  [3] 3-5
        //  30   31   32   33   34   35     [2] 6-8  [3] 6-8
        let mut result = 0;
        for (&square, offset) in squares.iter().zip([0, 3, 18, 21]) {
            result |= ((square & 0b111) << offset)
                | (((square >> 3) & 0b111) << (offset + 6))
                | (((square >> 6) & 0b111) << (offset + 12));
        }
        result
    }

    #[inline]
    fn split_6x6_into_2x2(square: usize) -> [usize; 9] {
        // [ 0]   1  [ 2]   3  [ 4]   5      0  0  1  1  2  2
        //   6    7    8    9   10   11      0  0  1  1  2  2
        // [12]  13  [14]  15  [16]  17      3  3  4  4  5  5
        //  18   19   20   21   22   23      3  3  4  4  5  5
        // [24]  25  [26]  27  [28]  29      6  6  7  7  8  8
        //  30   31   32   33   34   35      6  6  7  7  8  8
        [0, 2, 4, 12, 14, 16, 24, 26, 28]
            .map(|i| ((square >> i) & 0b11) | (((square >> (i + 6)) & 0b11) << 2))
    }
}

examples!(Day21 -> (u32, u32) []);
