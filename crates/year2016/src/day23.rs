use crate::assembunny::{Interpreter, InterpreterConfig};
use utils::prelude::*;

/// Interpreting assembunny assembly, again.
///
/// The key optimization is that
/// ```text
///  cpy $r1 $r2
///  inc $r3
///  dec $r2
///  jnz $r2 -2
///  dec $r4
///  jnz $r4 -5
/// ```
/// can be replaced with `$r3 = $r1 * r3` followed by `$r2 = 0` and `$r4 = 0`. This reduces the
/// number of simulated cycles for part 2 ~2,000,000 times when using the previously implemented
/// addition optimization, and ~6,000,000 times compared to a naive implementation.
///
/// See also [2016 day 12](crate::Day12).
#[derive(Clone, Debug)]
pub struct Day23 {
    interpreter: Interpreter<Self>,
}

impl Day23 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            interpreter: Interpreter::new(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> i32 {
        self.interpreter.execute([7, 0, 0, 0])
    }

    #[must_use]
    pub fn part2(&self) -> i32 {
        self.interpreter.execute([12, 0, 0, 0])
    }
}

impl InterpreterConfig for Day23 {
    const SUPPORTS_TOGGLE: bool = true;
}

examples!(Day23 -> (i32, i32) [
    {
        input: "cpy 2 a\n\
            tgl a\n\
            tgl a\n\
            tgl a\n\
            cpy 1 a\n\
            dec a\n\
            dec a",
        part1: 3,
    },
]);
