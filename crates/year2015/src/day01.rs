use utils::prelude::*;

/// Counting brackets.
#[derive(Clone, Debug)]
pub struct Day01 {
    directions: Vec<i32>,
}

impl Day01 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            directions: input
                .chars()
                .map(|c| match c {
                    '(' => Ok(1),
                    ')' => Ok(-1),
                    _ => Err(InputError::new(input, c, "expected bracket")),
                })
                .collect::<Result<Vec<i32>, InputError>>()?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> i32 {
        self.directions.iter().sum()
    }

    #[must_use]
    pub fn part2(&self) -> usize {
        self.directions
            .iter()
            .enumerate()
            .scan(0, |floor, (i, x)| {
                *floor += x;
                Some((i + 1, *floor)) // Character positions are 1-indexed in the puzzle
            })
            .find(|&(_, floor)| floor == -1)
            .expect("floor -1 not reached")
            .0
    }
}

examples!(Day01 -> (i32, usize) [
    {input: "(())", part1: 0},
    {input: "()()", part1: 0},
    {input: "(((", part1: 3},
    {input: "(()(()(", part1: 3},
    {input: "))(((((", part1: 3},
    {input: "())", part1: -1},
    {input: "))(", part1: -1},
    {input: ")))", part1: -3},
    {input: ")())())", part1: -3},
    {input: ")", part2: 1},
    {input: "()())", part2: 5},
]);
