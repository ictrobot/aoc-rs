use utils::prelude::*;

/// Counting how many times a dial passes zero.
#[derive(Clone, Debug)]
pub struct Day01 {
    part1: u32,
    part2: u32,
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
        let mut pos = 50i32;
        let (mut part1, mut part2) = (0, 0);
        for line in Direction::PARSER
            .then(parser::number_range(1i32..=999))
            .with_eol()
            .parse_iterator(input)
        {
            let (dir, steps) = line?;

            part2 += match dir {
                Direction::Left => ((100 - pos + steps) / 100) as u32 - u32::from(pos == 0),
                Direction::Right => ((pos + steps) / 100) as u32,
            };

            pos = (pos + (steps * dir as i32)).rem_euclid(100);
            part1 += u32::from(pos == 0);
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

examples!(Day01 -> (u32, u32) [
    {input: "L68\nL30\nR48\nL5\nR60\nL55\nL1\nL99\nR14\nL82", part1: 3, part2: 6},
]);
