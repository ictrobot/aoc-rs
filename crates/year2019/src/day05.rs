use crate::intcode::features::{Day05Part1Features, Day05Part2Features};
use crate::intcode::{Event, Features, Interpreter};
use utils::prelude::*;

/// Interpreting assembly with IO, conditionals and immediate operands.
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

        let mut last_output = 0;
        loop {
            match interpreter.run::<F>() {
                Event::Halt if last_output != 0 => return last_output,
                Event::Halt => panic!("no solution found: no non-zero output"),
                Event::Input => panic!("no solution found: program requires more input"),
                Event::Output(_) if last_output != 0 => {
                    panic!("no solution found: output after non-zero output")
                }
                Event::Output(x) => last_output = x,
            }
        }
    }
}

examples!(Day05 -> (i64, i64) []);
