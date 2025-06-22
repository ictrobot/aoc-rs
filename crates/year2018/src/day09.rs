use std::collections::VecDeque;
use utils::prelude::*;

/// Simulating a marble game.
#[derive(Clone, Debug)]
pub struct Day09 {
    players: u32,
    marbles: u32,
}

impl Day09 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (players, marbles) = parser::number_range(1..=999)
            .with_suffix(" players; last marble is worth ")
            .then(parser::number_range(1..=99_999))
            .with_suffix(" points")
            .parse_complete(input)?;

        Ok(Self { players, marbles })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        Self::max_score(self.players, self.marbles)
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        Self::max_score(self.players, self.marbles * 100)
    }

    fn max_score(players: u32, marbles: u32) -> u64 {
        let mut circle = VecDeque::with_capacity(marbles as usize);
        circle.push_front(0u32);
        let mut scores = vec![0u64; players as usize];

        let batches = marbles / 23;
        for base in (0..23 * batches).step_by(23) {
            // Equivalent to the following operations, which naively add 23 marbles while keeping
            // the current marble at the back of dequeue:
            //  22x [push_front(pop_back), push_front(pop_back), push_back(i)]
            //   7x [push_back(pop_front)]
            //      [pop_back]
            // By eliminating redundant pushes and pops the total number of operations per batch is
            // decreased from 125 to 67.
            let front = circle.pop_back().unwrap();
            circle.push_front(front);

            for i in 1..=18 {
                let front = circle.pop_back().unwrap();
                circle.push_front(front);
                circle.push_front(base + i);
            }

            let f1 = circle.pop_back().unwrap();
            let f2 = circle.pop_back().unwrap();
            let f3 = circle.pop_back().unwrap();
            let f4 = circle.pop_back().unwrap();

            circle.push_back(base + 22);
            circle.push_back(f4);
            circle.push_back(base + 21);
            circle.push_back(f3);
            circle.push_back(base + 20);
            circle.push_back(f2);
            circle.push_back(base + 19);

            scores[((base + 23) % players) as usize] += (base as u64 + 23) + (f1 as u64);
        }

        scores.iter().copied().max().unwrap()
    }
}

examples!(Day09 -> (u64, u64) [
    {input: "10 players; last marble is worth 1618 points", part1: 8317},
    {input: "13 players; last marble is worth 7999 points", part1: 146373},
    {input: "17 players; last marble is worth 1104 points", part1: 2764},
    {input: "21 players; last marble is worth 6111 points", part1: 54718},
    {input: "30 players; last marble is worth 5807 points", part1: 37305},
]);
