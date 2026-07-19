use crate::intcode::Interpreter;
use crate::intcode::features::Day09Features;
use utils::prelude::*;

/// Interpreting machine code to evaluate a boolean script.
#[derive(Clone, Debug)]
pub struct Day21 {
    interpreter: Interpreter,
}

// De Morgan's laws allow writing `J = (!A || !B || !C) && D` as `J = !(A && B && C) && D`, which
// can be written in 5 boolean instructions without using T.
const WALK_SCRIPT: &[u8] = b"OR A J\n\
AND B J\n\
AND C J\n\
NOT J J\n\
AND D J\n\
WALK\n";

// Jumping avoids holes at A, B and C before landing at D. After landing, H is another 4 tiles
// ahead, so if there is ground at H, a jump to D can be followed by an immediate second jump to H.
// Jump now if there is ground at D, and a hole at A or B, or a hole at C as well as ground at H.
// Otherwise walk once so C becomes the new B and E becomes the new landing tile D. This gives
// `J = (!A || !B || (!C && H)) && D`, which can be rewritten as `J = !(A && B && (C || !H)) && D`,
// using 6 boolean instructions without using T.
const RUN_SCRIPT: &[u8] = b"NOT H J\n\
OR C J\n\
AND B J\n\
AND A J\n\
NOT J J\n\
AND D J\n\
RUN\n";

impl Day21 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            interpreter: Interpreter::parse(input, 1)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> i64 {
        self.run(WALK_SCRIPT)
    }

    #[must_use]
    pub fn part2(&self) -> i64 {
        self.run(RUN_SCRIPT)
    }

    fn run(&self, script: &[u8]) -> i64 {
        let mut interpreter = self.interpreter.clone();
        interpreter.push_bytes(script);

        loop {
            if let x @ 128.. = interpreter.expect_output::<Day09Features>() {
                return x;
            }
        }
    }
}

examples!(Day21 -> (i64, i64) []);
