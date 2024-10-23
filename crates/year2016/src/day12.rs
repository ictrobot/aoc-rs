use crate::assembunny::Interpreter;
use utils::prelude::*;

/// Interpreting assembly, again.
///
/// The key optimization is that
/// ```text
/// inc $r1
/// dec $r2
/// jnz $r2 -2
/// ```
/// can be replaced with `$r1 += $r2` followed by `$r2 = 0`. This reduces the number of simulated
/// cycles ~5,000 times for part 1 and ~100,000 times for part 2, to around ~200 cycles each.
#[derive(Clone, Debug)]
pub struct Day12 {
    interpreter: Interpreter<false, false>,
}

impl Day12 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            interpreter: Interpreter::new(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> i32 {
        self.interpreter.execute([0; 4])
    }

    #[must_use]
    pub fn part2(&self) -> i32 {
        self.interpreter.execute([0, 0, 1, 0])
    }
}

examples!(Day12 -> (i32, i32) [
    {
        input: "cpy 41 a\n\
            inc a\n\
            inc a\n\
            dec a\n\
            jnz a 2\n\
            dec a",
        part1: 42
    },
]);
