use std::collections::HashMap;
use utils::bit::BitIterator;
use utils::prelude::*;

/// Finding the shortest and longest path.
///
/// Traverse each route from the first location, adding on the edge from the last location back to
/// the first location to find the shortest/longest loop. The shortest/longest route starting and
/// ending at different locations is then the shortest/longest loop minus the longest/shortest edge.
#[derive(Clone, Debug)]
pub struct Day09 {
    part1: u32,
    part2: u32,
}

type Visited = u32;

impl Day09 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let location = parser::take_while1(u8::is_ascii_alphabetic).error_msg("expected location");
        let parsed = location
            .then(location.with_prefix(" to ").with_suffix(" = "))
            .then(parser::u32())
            .parse_lines(input)?;

        let mut indexes = HashMap::new();
        parsed.iter().for_each(|&(start, end, _)| {
            let len = indexes.len();
            indexes.entry(start).or_insert(len);
            let len = indexes.len();
            indexes.entry(end).or_insert(len);
        });

        if indexes.len() > Visited::BITS as usize {
            return Err(InputError::new(input, 0, "too many locations"));
        }

        let mut matrix = vec![0; indexes.len() * indexes.len()];
        parsed.iter().for_each(|&(start, end, dist)| {
            let start = indexes[start];
            let end = indexes[end];
            matrix[indexes.len() * start + end] = dist;
            matrix[indexes.len() * end + start] = dist;
        });

        let (part1, part2) = Visitor::visit_all(matrix, indexes.len() as u32);
        Ok(Self { part1, part2 })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.part2
    }
}

struct Visitor {
    matrix: Vec<u32>,
    locations: u32,
    min: u32,
    max: u32,
}

impl Visitor {
    #[must_use]
    fn visit_all(matrix: Vec<u32>, locations: u32) -> (u32, u32) {
        let mut v = Self {
            matrix,
            locations,
            min: u32::MAX,
            max: 0,
        };
        v.visit(
            0,
            !(Visited::MAX >> (Visited::BITS - locations)) | 1,
            0,
            u32::MAX,
            0,
        );
        (v.min, v.max)
    }

    fn visit(&mut self, prev: u32, visited: Visited, distance: u32, min_edge: u32, max_edge: u32) {
        if visited == Visited::MAX {
            let loop_edge = self.matrix[prev as usize];
            self.min = self.min.min(distance + loop_edge - max_edge.max(loop_edge));
            self.max = self.max.max(distance + loop_edge - min_edge.min(loop_edge));
            return;
        }

        for (next, next_bit) in BitIterator::zeroes(visited) {
            let edge = self.matrix[(prev * self.locations + next) as usize];
            self.visit(
                next,
                visited | next_bit,
                distance + edge,
                min_edge.min(edge),
                max_edge.max(edge),
            );
        }
    }
}

examples!(Day09 -> (u32, u32) [
    {
        input: "London to Dublin = 464\n\
            London to Belfast = 518\n\
            Dublin to Belfast = 141",
        part1: 605,
        part2: 982,
    },
]);
