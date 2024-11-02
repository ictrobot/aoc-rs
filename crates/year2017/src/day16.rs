use std::array;
use std::fmt::{Display, Formatter, Write};
use std::ops::MulAssign;
use utils::prelude::*;

/// Composing permutations.
///
/// The key optimization is that all the index-based permutations (Spin and Exchange) can be
/// combined into one permutation, and all the value-based permutations (Partner) can be combined
/// into another.
#[derive(Clone, Debug)]
pub struct Day16 {
    dance: Dance,
}

#[derive(Clone, Debug)]
enum DanceMove {
    Spin(u8),
    Exchange(u8, u8),
    Partner(u8, u8),
}

#[derive(Copy, Clone, Debug)]
struct Dance {
    index_perm: [usize; 16],
    value_perm: [usize; 16],
}

impl Day16 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let program = parser::byte_range(b'a'..=b'p').map(|b| b - b'a');
        let position = parser::number_range(0..=15);

        Ok(Self {
            dance: parser::one_of((
                position.with_prefix(b's').map(DanceMove::Spin),
                position
                    .with_prefix(b'x')
                    .then(position.with_prefix(b'/'))
                    .map(|(a, b)| DanceMove::Exchange(a, b)),
                program
                    .with_prefix(b'p')
                    .then(program.with_prefix(b'/'))
                    .map(|(a, b)| DanceMove::Partner(a, b)),
            ))
            .with_suffix(b','.or(parser::eof()))
            .parse_iterator(input)
            .collect::<Result<Dance, InputError>>()?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> String {
        self.dance.to_string()
    }

    #[must_use]
    pub fn part2(&self) -> String {
        // Exponentiation by squaring, but for dance permutations
        let mut result = Dance::default();
        let mut base = self.dance;
        let mut exponent = 1_000_000_000;

        while exponent > 0 {
            if exponent % 2 == 1 {
                result *= base;
            }
            exponent >>= 1;
            base *= base;
        }

        result.to_string()
    }
}

impl Default for Dance {
    fn default() -> Self {
        Dance {
            index_perm: array::from_fn(|i| i),
            value_perm: array::from_fn(|i| i),
        }
    }
}

impl FromIterator<DanceMove> for Dance {
    #[inline]
    fn from_iter<T: IntoIterator<Item = DanceMove>>(iter: T) -> Self {
        let mut dance = Dance::default();
        let mut value_positions: [usize; 16] = array::from_fn(|i| i);
        let mut rotation = 0; // Rotate once at the end as it is expensive

        for dance_move in iter {
            match dance_move {
                DanceMove::Spin(r) => rotation += 16 - r as usize,
                DanceMove::Exchange(a, b) => {
                    let a_idx = (a as usize + rotation) % 16;
                    let b_idx = (b as usize + rotation) % 16;
                    dance.index_perm.swap(a_idx, b_idx);
                }
                DanceMove::Partner(a, b) => {
                    value_positions.swap(a as usize, b as usize);
                    let a_idx = value_positions[a as usize];
                    let b_idx = value_positions[b as usize];
                    dance.value_perm.swap(a_idx, b_idx);
                }
            }
        }
        dance.index_perm.rotate_left(rotation % 16);

        dance
    }
}

impl Display for Dance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for &i in &self.index_perm {
            f.write_char((self.value_perm[i] as u8 + b'a') as char)?;
        }
        Ok(())
    }
}

impl MulAssign for Dance {
    fn mul_assign(&mut self, rhs: Self) {
        self.index_perm = self.index_perm.map(|i| rhs.index_perm[i]);
        self.value_perm = self.value_perm.map(|i| rhs.value_perm[i]);
    }
}

examples!(Day16 -> (&'static str, &'static str) []);
