use std::collections::HashSet;
use utils::geometry::Vec2;
use utils::prelude::*;

/// Counting unique points.
#[derive(Clone, Debug)]
pub struct Day03 {
    directions: Vec<Vec2<i32>>,
}

impl Day03 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            directions: input
                .chars()
                .map(|c| match c {
                    '^' => Ok(Vec2::UP),
                    '>' => Ok(Vec2::RIGHT),
                    'v' => Ok(Vec2::DOWN),
                    '<' => Ok(Vec2::LEFT),
                    _ => Err(InputError::new(input, c, "expected one of ^>v<")),
                })
                .collect::<Result<Vec<Vec2<i32>>, InputError>>()?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        let mut pos = Vec2::default();

        self.count_positions(|dir| {
            pos += dir;
            pos
        })
    }

    #[must_use]
    pub fn part2(&self) -> usize {
        let (mut pos1, mut pos2) = Default::default();

        self.count_positions(|dir| {
            (pos1, pos2) = (pos2, pos1);
            pos1 += dir;
            pos1
        })
    }

    fn count_positions(&self, mut f: impl FnMut(Vec2<i32>) -> Vec2<i32>) -> usize {
        let mut set = HashSet::with_capacity(self.directions.len());
        set.insert(Vec2::default());

        for &dir in &self.directions {
            set.insert(f(dir));
        }

        set.len()
    }
}

examples!(Day03 -> (usize, usize) [
    {input: ">", part1: 2},
    {input: "^>", part2: 3},
    {input: "^>v<", part1: 4, part2: 3},
    {input: "^v^v^v^v^v", part1: 2, part2: 11},
]);
