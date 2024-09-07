use std::cell::RefCell;
use std::collections::HashSet;
use std::iter;
use utils::number::{chinese_remainder, is_prime};
use utils::prelude::*;

/// Finding when discs align.
///
/// This puzzle is a system of linear simultaneous congruences which can be solved using
/// <https://en.wikipedia.org/wiki/Chinese_remainder_theorem>.
#[derive(Clone, Debug)]
pub struct Day15 {
    discs: Vec<Disc>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct Disc {
    size: u32,
    position: u32,
}

impl Day15 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let seen_positions: RefCell<HashSet<u32>> = Default::default();
        Ok(Self {
            discs: parser::u32()
                .with_prefix(" has ".with_prefix(parser::u32()).with_prefix("Disc #"))
                .then(parser::u32().with_prefix(" positions; at time=0, it is at position "))
                .with_suffix(".")
                .map_res(|(size, position)| {
                    if position >= size {
                        Err("current position should be less than number of positions")
                    } else if !is_prime(size) {
                        Err("number of positions should be prime")
                    } else if !seen_positions.borrow_mut().insert(size) {
                        Err("number of positions should be unique")
                    } else {
                        Ok(Disc { size, position })
                    }
                })
                .parse_lines(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> i64 {
        Self::earliest_alignment(self.discs.iter())
    }

    #[must_use]
    pub fn part2(&self) -> i64 {
        Self::earliest_alignment(self.discs.iter().chain(iter::once(&Disc {
            size: 11,
            position: 0,
        })))
    }

    #[inline]
    fn earliest_alignment<'a>(discs: impl Iterator<Item = &'a Disc> + Clone) -> i64 {
        let residues = discs
            .clone()
            .enumerate()
            .map(|(i, disc)| -(disc.position as i64) - (i as i64 + 1));
        let moduli = discs.map(|disc| disc.size as i64);

        chinese_remainder(residues, moduli).expect("sizes are all prime")
    }
}

examples!(Day15 -> (i64, i64) [
    {
        input: "Disc #1 has 5 positions; at time=0, it is at position 4.\n\
            Disc #2 has 2 positions; at time=0, it is at position 1.",
        part1: 5
    },
]);
