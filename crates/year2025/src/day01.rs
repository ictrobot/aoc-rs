use utils::prelude::*;

/// Counting how many times a dial passes zero.
#[derive(Clone, Debug)]
pub struct Day01 {
    turns: Vec<(Direction, i32)>,
}

parser::parsable_enum! {
    #[derive(Clone, Copy, Debug)]
    enum Direction {
        "L" => Left = -1,
        "R" => Right = 1,
    }
}

impl Day01 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            turns: Direction::PARSER
                .then(parser::number_range(1i32..=999))
                .parse_lines(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let mut zero_count = 0;
        let mut pos = 50;
        for &(dir, steps) in &self.turns {
            pos = (pos + (steps * dir as i32)).rem_euclid(100);
            zero_count += u32::from(pos == 0);
        }
        zero_count
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut zero_count = 0;
        let mut pos = 50;
        for &(dir, steps) in &self.turns {
            zero_count += match dir {
                Direction::Left => ((100 - pos + steps) / 100) as u32 - u32::from(pos == 0),
                Direction::Right => ((pos + steps) / 100) as u32,
            };
            pos = (pos + (steps * dir as i32)).rem_euclid(100);
        }
        zero_count
    }
}

examples!(Day01 -> (u32, u32) [
    {input: "L68\nL30\nR48\nL5\nR60\nL55\nL1\nL99\nR14\nL82", part1: 3, part2: 6},
]);
