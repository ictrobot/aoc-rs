use std::collections::HashSet;
use utils::point::Point2D;
use utils::prelude::*;

/// Counting unique points.
#[derive(Clone, Debug)]
pub struct Day03 {
    directions: Vec<Point2D<i32>>,
}

impl Day03 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InvalidInputError> {
        Ok(Self {
            directions: input
                .chars()
                .map(|c| match c {
                    '^' => Ok(Point2D::UP),
                    '>' => Ok(Point2D::RIGHT),
                    'v' => Ok(Point2D::DOWN),
                    '<' => Ok(Point2D::LEFT),
                    _ => Err(InvalidInputError::UnexpectedChar(c)),
                })
                .collect::<Result<Vec<Point2D<i32>>, InvalidInputError>>()?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        let mut pos = Point2D::default();

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

    fn count_positions(&self, mut f: impl FnMut(Point2D<i32>) -> Point2D<i32>) -> usize {
        let mut set = HashSet::with_capacity(self.directions.len());
        set.insert(Point2D::default());

        for &dir in &self.directions {
            set.insert(f(dir));
        }

        set.len()
    }
}

examples!(Day03<usize, usize> => [
    ">" part1=2,
    "^>" part2=3,
    "^>v<" part1=4 part2=3,
    "^v^v^v^v^v" part1=2 part2=11,
]);
