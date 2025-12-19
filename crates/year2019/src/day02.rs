use crate::intcode::Interpreter;
use crate::intcode::features::Day02Features;
use utils::prelude::*;

/// Interpreting assembly with add and multiply instructions.
///
/// Part 2 assumes the output grows monotonically with both inputs and that the first input
/// is more significant than the second input.
#[derive(Clone, Debug)]
pub struct Day02 {
    interpreter: Interpreter,
}

const PART2_TARGET: i64 = 19_690_720;

impl Day02 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            interpreter: Interpreter::parse(input, 3)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> i64 {
        self.run(12, 2)
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut noun_lo = 0u32;
        let mut noun_hi = 99u32;
        while noun_lo + 1 < noun_hi {
            let mid = noun_lo + (noun_hi - noun_lo) / 2;
            if self.run(mid as i64, 0) <= PART2_TARGET {
                noun_lo = mid;
            } else {
                noun_hi = mid;
            }
        }

        let mut verb_lo = 0u32;
        let mut verb_hi = 99u32;
        while verb_lo + 1 < verb_hi {
            let mid = verb_lo + (verb_hi - verb_lo) / 2;
            if self.run(noun_lo as i64, mid as i64) <= PART2_TARGET {
                verb_lo = mid;
            } else {
                verb_hi = mid;
            }
        }

        assert_eq!(
            self.run(noun_lo as i64, verb_lo as i64),
            PART2_TARGET,
            "no solution found"
        );

        noun_lo * 100 + verb_lo
    }

    fn run(&self, noun: i64, verb: i64) -> i64 {
        let mut interpreter = self.interpreter.clone();
        interpreter.mem[1] = noun;
        interpreter.mem[2] = verb;

        let _ = interpreter.run::<Day02Features>();

        interpreter.mem[0]
    }
}

examples!(Day02 -> (i64, u32) []);
