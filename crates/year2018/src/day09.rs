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
            .then(parser::number_range(23..=99_999))
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
        let batches = marbles / 23;

        // Each batch does 23x pop_back, 7x push_back and 37x push_front, meaning the buffer only
        // grows towards the front and the head pointer progresses far faster than the tail pointer.
        // The score only depends on values near the tail and within each batch is computed before
        // the tail is overwritten.
        let len = ((batches - 1) as usize * 16).next_multiple_of(37) + 22;
        let mut circle = vec![0u32; len];

        // Start with the first batch completed to ensure there are enough entries to pop
        circle[len - 22..].copy_from_slice(&[
            18, 4, 17, 8, 16, 0, 15, 7, 14, 3, 13, 6, 12, 1, 11, 22, 5, 21, 10, 20, 2, 19,
        ]);
        let (mut head, mut tail) = (len - 22, len - 1);
        let mut scores = vec![0u64; players as usize];
        scores[(23 % players) as usize] += 32;

        for base in (23..23 * batches).step_by(23) {
            // Equivalent to the following operations, which naively add 23 marbles while keeping
            // the current marble at the back of dequeue:
            //  22x [push_front(pop_back), push_front(pop_back), push_back(i)]
            //   7x [push_back(pop_front)]
            //      [pop_back]

            scores[((base + 23) % players) as usize] +=
                (base + 23) as u64 + circle[tail - 19] as u64;

            if head > 0 {
                let push_front = [
                    base + 18,
                    circle[tail - 18],
                    base + 17,
                    circle[tail - 17],
                    base + 16,
                    circle[tail - 16],
                    base + 15,
                    circle[tail - 15],
                    base + 14,
                    circle[tail - 14],
                    base + 13,
                    circle[tail - 13],
                    base + 12,
                    circle[tail - 12],
                    base + 11,
                    circle[tail - 11],
                    base + 10,
                    circle[tail - 10],
                    base + 9,
                    circle[tail - 9],
                    base + 8,
                    circle[tail - 8],
                    base + 7,
                    circle[tail - 7],
                    base + 6,
                    circle[tail - 6],
                    base + 5,
                    circle[tail - 5],
                    base + 4,
                    circle[tail - 4],
                    base + 3,
                    circle[tail - 3],
                    base + 2,
                    circle[tail - 2],
                    base + 1,
                    circle[tail - 1],
                    circle[tail],
                ];
                let push_back = [
                    base + 22,
                    circle[tail - 22],
                    base + 21,
                    circle[tail - 21],
                    base + 20,
                    circle[tail - 20],
                    base + 19,
                ];

                circle[head - 37..head].copy_from_slice(&push_front);
                circle[tail - 22..tail - 15].copy_from_slice(&push_back);

                head -= 37;
            }

            tail -= 16;
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
