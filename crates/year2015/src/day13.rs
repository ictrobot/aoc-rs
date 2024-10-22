use std::collections::HashMap;
use utils::graph::explore_hamiltonian_paths;
use utils::prelude::*;

/// Seating plan.
///
/// Very similar to [Day 9](crate::Day09), including part 2, which only requires subtracting the
/// minimum edge from each permutation's total.
#[derive(Clone, Debug)]
pub struct Day13 {
    part1: i32,
    part2: i32,
}

type Seated = u32;

impl Day13 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let parsed = parser::take_while1(u8::is_ascii_alphabetic)
            .with_suffix(" would ")
            .then(
                parser::one_of((
                    parser::u16().with_prefix("gain ").map(i32::from),
                    parser::u16().with_prefix("lose ").map(|x| -i32::from(x)),
                ))
                .with_suffix(" happiness units by sitting next to "),
            )
            .then(parser::take_while1(u8::is_ascii_alphabetic).with_suffix(b'.'))
            .parse_lines(input)?;

        let mut indexes = HashMap::new();
        parsed.iter().for_each(|&(person, ..)| {
            let len = indexes.len();
            indexes.entry(person).or_insert(len);
        });

        if indexes.len() > Seated::BITS as usize {
            return Err(InputError::new(input, 0, "too many people"));
        }

        let people = indexes.len();
        let mut matrix = vec![0; people * people];
        parsed.iter().for_each(|&(person1, change, person2)| {
            matrix[indexes[person1] * people + indexes[person2]] = change;
        });

        let (mut part1, mut part2) = (i32::MIN, i32::MIN);
        explore_hamiltonian_paths(
            people as u32,
            0,
            (0, i32::MAX),
            |a, b| {
                matrix[a as usize * people + b as usize] + matrix[b as usize * people + a as usize]
            },
            |(total, min_edge), edge| (total + edge, min_edge.min(edge)),
            |(total, min_edge), loop_edge| {
                part1 = part1.max(total + loop_edge);
                part2 = part2.max(total + loop_edge - min_edge.min(loop_edge));
            },
        );

        Ok(Self { part1, part2 })
    }

    #[must_use]
    pub fn part1(&self) -> i32 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> i32 {
        self.part2
    }
}

examples!(Day13 -> (i32, i32) [
    {file: "day13_example0.txt", part1: 330},
]);
