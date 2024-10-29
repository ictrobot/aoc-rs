use std::collections::HashMap;
use utils::prelude::*;

/// Evaluating conditional add instructions.
#[derive(Clone, Debug)]
pub struct Day08 {
    part1: i32,
    part2: i32,
}

impl Day08 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let parsed = parser::take_while1(u8::is_ascii_lowercase)
            .with_suffix(" ")
            .then(
                parser::one_of((
                    parser::i32().with_prefix("inc "),
                    parser::i32().with_prefix("dec ").map(|x| -x),
                ))
                .with_suffix(" if "),
            )
            .then(parser::take_while1(u8::is_ascii_lowercase).with_suffix(" "))
            .then(
                parser::one_of((
                    "==".map(|_| i32::eq as fn(&i32, &i32) -> bool),
                    "!=".map(|_| i32::ne as fn(&i32, &i32) -> bool),
                    "<=".map(|_| i32::le as fn(&i32, &i32) -> bool),
                    ">=".map(|_| i32::ge as fn(&i32, &i32) -> bool),
                    "<".map(|_| i32::lt as fn(&i32, &i32) -> bool),
                    ">".map(|_| i32::gt as fn(&i32, &i32) -> bool),
                ))
                .with_suffix(" "),
            )
            .then(parser::i32())
            .parse_lines(input)?;

        let mut registers = HashMap::new();
        let mut max = 0;
        for (reg, value, cond_reg, comparison, cond_value) in parsed {
            if comparison(registers.entry(cond_reg).or_insert(0), &cond_value) {
                let entry = registers.entry(reg).or_insert(0);
                *entry += value;
                max = max.max(*entry);
            }
        }

        Ok(Self {
            part1: registers.into_values().max().unwrap_or(0),
            part2: max,
        })
    }

    #[must_use]
    pub fn part1(&self) -> i32 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> i32 {
        self.part2
    }
}

examples!(Day08 -> (i32, i32) [
    {
        input: "b inc 5 if a > 1\n\
            a inc 1 if b < 5\n\
            c dec -10 if a >= 1\n\
            c inc -20 if c == 10",
        part1: 1,
        part2: 10
    },
]);
