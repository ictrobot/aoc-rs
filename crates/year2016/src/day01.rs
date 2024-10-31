use std::collections::HashSet;
use utils::point::Point2D;
use utils::prelude::*;

/// Calculating Manhattan distance.
#[derive(Clone, Debug)]
pub struct Day01 {
    instructions: Vec<(Turn, u16)>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Turn {
    L,
    R,
}

impl Day01 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            instructions: parser::literal_map!("L" => Turn::L, "R" => Turn::R)
                .then(parser::u16())
                .with_suffix(", ".optional())
                .parse_all(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> i32 {
        let mut pos = Point2D::ORIGIN;
        let mut dir = Point2D::UP;

        for &(turn, steps) in &self.instructions {
            dir = match turn {
                Turn::L => dir.turn_left(),
                Turn::R => dir.turn_right(),
            };
            pos += dir * i32::from(steps);
        }

        pos.manhattan_distance()
    }

    #[must_use]
    pub fn part2(&self) -> i32 {
        let mut pos: Point2D<i32> = Point2D::ORIGIN;
        let mut dir = Point2D::UP;
        let mut visited = HashSet::new();

        for &(turn, steps) in &self.instructions {
            dir = match turn {
                Turn::L => dir.turn_left(),
                Turn::R => dir.turn_right(),
            };
            for _ in 0..steps {
                pos += dir;
                if !visited.insert(pos) {
                    return pos.manhattan_distance();
                }
            }
        }

        panic!("no location visited twice");
    }
}

examples!(Day01 -> (i32, i32) [
    {input: "R2, L3", part1: 5},
    {input: "R2, R2, R2", part1: 2},
    {input: "R5, L5, R5, R3", part1: 12},
    {input: "R8, R4, R4, R8", part2: 4},
]);
