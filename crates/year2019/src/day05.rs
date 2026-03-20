use crate::intcode::features::{Day05Part1Features, Day05Part2Features};
use crate::intcode::{Features, Interpreter};
use utils::prelude::*;

/// Interpreting machine code with IO, conditionals and immediate operands.
#[derive(Clone, Debug)]
pub struct Day05 {
    interpreter: Interpreter,
}

impl Day05 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            interpreter: Interpreter::parse(input, 1)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> i64 {
        self.run::<Day05Part1Features>(1)
    }

    #[must_use]
    pub fn part2(&self) -> i64 {
        self.run::<Day05Part2Features>(5)
    }

    fn run<F: Features>(&self, input: i64) -> i64 {
        let mut interpreter = self.interpreter.clone();
        interpreter.push_input(input);

        loop {
            let output = interpreter.expect_output::<F>();

            if output != 0 {
                interpreter.expect_halt::<F>();
                return output;
            }
        }
    }
}

examples!(Day05 -> (i64, i64) []);
