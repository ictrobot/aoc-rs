use utils::prelude::*;

/// Finding the winners of counting-out games.
#[derive(Clone, Debug)]
pub struct Day19 {
    elves: u32,
}

impl Day19 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            elves: parser::u32().parse_complete(input)?,
        })
    }

    /// See <https://en.wikipedia.org/wiki/Josephus_problem#k_=_2>.
    #[must_use]
    pub fn part1(&self) -> u32 {
        2 * (self.elves - (1 << self.elves.ilog2())) + 1
    }

    /// See <https://www.reddit.com/r/adventofcode/comments/5j4lp1/2016_day_19_solutions/>.
    ///
    /// If the number of elves is a power of 3, the final elf wins. Otherwise, calculate the
    /// largest power of 3 smaller or equal to the number of elves (`pow3`). Less than `2 * pow3`
    /// the winner is `elves - pow3`, and after that it `2 * (elves - pow3) + pow3`.
    #[must_use]
    pub fn part2(&self) -> u32 {
        let pow3 = 3u32.pow(self.elves.ilog(3));
        if pow3 == self.elves {
            pow3
        } else {
            let remainder = self.elves - pow3;
            if remainder <= pow3 {
                remainder
            } else {
                2 * remainder - pow3
            }
        }
    }
}

examples!(Day19 -> (u32, u32) [
    {input: "5", part1: 3, part2: 2},
]);
