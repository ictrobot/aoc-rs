use utils::prelude::*;

use crate::intcode::features::Day09Features;
use crate::intcode::{Event, Interpreter};

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

        let mut output = None;
        loop {
            match interpreter.run::<Day09Features>() {
                Event::Halt => {
                    return output.expect("no solution found: program produced no output");
                }
                Event::Input => panic!("no solution found: program required more input"),
                Event::Output(_) if output.is_some() => {
                    panic!("no solution found: program output multiple values")
                }
                Event::Output(x) => output = Some(x),
            }
        }
    }
}

examples!(Day09 -> (i64, i64) []);
