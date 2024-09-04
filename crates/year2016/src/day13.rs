use std::collections::{HashSet, VecDeque};
use utils::point::Point2D;
use utils::prelude::*;

/// Finding the shortest path.
#[derive(Clone, Debug)]
pub struct Day13 {
    part1: u32,
    part2: u32,
}

impl Day13 {
    pub fn new(input: &str, input_type: InputType) -> Result<Self, InputError> {
        let favorite_number = parser::u32().parse_complete(input)?;
        let target: Point2D<u32> = if input_type == InputType::Real {
            Point2D::new(31, 39)
        } else {
            Point2D::new(7, 4)
        };

        // Use a hashset to store visited nodes to avoid having a fixed grid size, as theoretically
        // the shortest route to the target may first go a long way down/right.
        let mut visited = HashSet::new();
        visited.insert(Point2D::new(1, 1));
        let mut queue = VecDeque::new();
        queue.push_back((Point2D::new(1, 1), 0));

        let (mut part1, mut part2) = (0, 0);
        while let Some((p, steps)) = queue.pop_front() {
            if p == target {
                part1 = steps;
            }

            if steps <= 50 {
                part2 += 1;
            } else if part1 != 0 {
                break;
            }

            for next @ Point2D { x, y } in [
                Point2D::new(p.x.saturating_sub(1), p.y),
                Point2D::new(p.x.saturating_add(1), p.y),
                Point2D::new(p.x, p.y.saturating_sub(1)),
                Point2D::new(p.x, p.y.saturating_add(1)),
            ] {
                let n = (x * x) + (3 * x) + (2 * x * y) + y + (y * y) + favorite_number;
                if n.count_ones() % 2 == 0 && !visited.contains(&next) {
                    visited.insert(next);
                    queue.push_back((next, steps + 1));
                }
            }
        }

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

examples!(Day13 -> (u32, u32) [
    {input: "10", part1: 11},
]);
