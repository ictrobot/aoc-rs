use utils::prelude::*;

/// Summing values from a recursive formula.
#[derive(Clone, Debug)]
pub struct Day01 {
    modules: Vec<u32>,
}

impl Day01 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            modules: parser::u32().parse_lines(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.modules
            .iter()
            .map(|mass| (mass / 3).saturating_sub(2))
            .sum()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.modules
            .iter()
            .map(|&(mut mass)| {
                let mut fuel = 0;
                while mass > 0 {
                    mass = (mass / 3).saturating_sub(2);
                    fuel += mass;
                }
                fuel
            })
            .sum()
    }
}

examples!(Day01 -> (u32, u32) [
    {input: "12", part1: 2, part2: 2},
    {input: "14", part1: 2, part2: 2},
    {input: "1969", part1: 654, part2: 966},
    {input: "100756", part1: 33583, part2: 50346},
]);
