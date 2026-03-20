use utils::prelude::*;

use crate::intcode::Interpreter;
use crate::intcode::features::Day09Features;

/// Interpreting assembly with base-relative addressing.
#[derive(Clone, Debug)]
pub struct Day09 {
    interpreter: Interpreter,
}

impl Day09 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            interpreter: Interpreter::parse(input, 1)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> i64 {
        self.run(1)
    }

    #[must_use]
    pub fn part2(&self) -> i64 {
        self.run(2)
    }

    fn run(&self, input: i64) -> i64 {
        let mut interpreter = self.interpreter.clone();
        interpreter.push_input(input);

        let output = interpreter.expect_output::<Day09Features>();
        interpreter.expect_halt::<Day09Features>();
        output
    }
}

examples!(Day09 -> (i64, i64) []);
