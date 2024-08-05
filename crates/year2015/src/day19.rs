use std::collections::HashSet;
use utils::prelude::*;

/// Molecule string replacements.
///
/// Part 2 assumes there is only one possible number of steps, and that the replacements are always
/// the same length or longer.
#[derive(Clone, Debug)]
pub struct Day19<'a> {
    replacements: Vec<(&'a [u8], &'a [u8])>,
    molecule: &'a [u8],
}

impl<'a> Day19<'a> {
    pub fn new(input: &'a str, _: InputType) -> Result<Self, InputError> {
        let Some((replacements, molecule)) = input.rsplit_once("\n\n") else {
            return Err(InputError::new(
                input,
                0,
                "expected replacements then a blank line then the molecule",
            ));
        };

        Ok(Self {
            replacements: parser::take_while1(u8::is_ascii_alphabetic)
                .then(parser::take_while1(u8::is_ascii_alphabetic).with_prefix(" => "))
                .parse_lines(replacements)?,
            molecule: molecule.trim_ascii_end().as_bytes(),
        })
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        let mut set = HashSet::new();
        for &(from, to) in &self.replacements {
            let new_length = self.molecule.len() - from.len() + to.len();
            for i in 0..self.molecule.len() {
                if self.molecule[i..].starts_with(from) {
                    let mut molecule = Vec::with_capacity(new_length);
                    molecule.extend_from_slice(&self.molecule[..i]);
                    molecule.extend_from_slice(to);
                    molecule.extend_from_slice(&self.molecule[i + from.len()..]);
                    set.insert(molecule);
                }
            }
        }
        set.len()
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        let mut molecule = self.molecule.to_vec();
        let mut steps = 0;
        while molecule.iter().any(|&x| x != b'e') {
            for &(from, to) in &self.replacements {
                let mut i = 0;
                while i < molecule.len() {
                    if molecule[i..].starts_with(to) {
                        // Replace to with from, presuming from.len() <= to.len()
                        molecule[i..i + from.len()].copy_from_slice(from);
                        molecule.drain(i + from.len()..i + to.len());

                        steps += 1;
                    } else {
                        i += 1;
                    }
                }
            }
        }
        steps
    }
}

examples!(Day19<'_> -> (usize, u64) [
    {input: "H => HO\nH => OH\nO => HH\n\nHOH", part1: 4},
    {input: "e => H\ne => O\nH => HO\nH => OH\nO => HH\n\nHOH", part2: 3},
    {input: "e => H\ne => O\nH => HO\nH => OH\nO => HH\n\nHOHOHO", part2: 6},
]);
