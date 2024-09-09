use std::collections::HashMap;
use utils::bit::BitIterator;
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

        let (part1, part2) = Visitor::visit_all(matrix, people as u32);
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

struct Visitor {
    matrix: Vec<i32>,
    people: u32,
    part1: i32,
    part2: i32,
}

impl Visitor {
    #[must_use]
    fn visit_all(matrix: Vec<i32>, people: u32) -> (i32, i32) {
        let mut v = Self {
            matrix,
            people,
            part1: i32::MIN,
            part2: i32::MIN,
        };
        v.visit(
            0,
            !(Seated::MAX >> (Seated::BITS - people)) | 1,
            0,
            i32::MAX,
        );
        (v.part1, v.part2)
    }

    fn visit(&mut self, prev: u32, seated: Seated, total: i32, min_edge: i32) {
        if seated == Seated::MAX {
            let loop_edge = self.matrix[prev as usize] + self.matrix[(prev * self.people) as usize];
            self.part1 = self.part1.max(total + loop_edge);
            self.part2 = self.part2.max(total + loop_edge - min_edge.min(loop_edge));
            return;
        }

        for (next, next_bit) in BitIterator::zeroes(seated) {
            let edge = self.matrix[(prev * self.people + next) as usize]
                + self.matrix[(next * self.people + prev) as usize];
            self.visit(next, seated | next_bit, total + edge, min_edge.min(edge));
        }
    }
}

examples!(Day13 -> (i32, i32) [
    {file: "day13_example0.txt", part1: 330},
]);
