use std::collections::HashMap;
use utils::graph::explore_hamiltonian_paths;
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

        let locations = indexes.len();
        if locations > Visited::BITS as usize {
            return Err(InputError::new(input, 0, "too many locations"));
        }

        let mut matrix = vec![0; indexes.len() * indexes.len()];
        parsed.iter().for_each(|&(start, end, dist)| {
            let start = indexes[start];
            let end = indexes[end];
            matrix[indexes.len() * start + end] = dist;
            matrix[indexes.len() * end + start] = dist;
        });

        let (mut part1, mut part2) = (u32::MAX, 0);
        explore_hamiltonian_paths(
            indexes.len() as u32,
            0,
            (0, u32::MAX, 0),
            |a, b| matrix[a as usize * locations + b as usize],
            |(total, min_edge, max_edge), edge| {
                (total + edge, min_edge.min(edge), max_edge.max(edge))
            },
            |(total, min_edge, max_edge), loop_edge| {
                part1 = part1.min(total + loop_edge - max_edge.max(loop_edge));
                part2 = part2.max(total + loop_edge - min_edge.min(loop_edge))
            },
        );

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

examples!(Day09 -> (u32, u32) [
    {
        input: "London to Dublin = 464\n\
            London to Belfast = 518\n\
            Dublin to Belfast = 141",
        part1: 605,
        part2: 982,
    },
]);
