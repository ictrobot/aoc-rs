use crate::assembunny::Interpreter;
use std::collections::HashSet;
use std::ops::ControlFlow;
use utils::prelude::*;

/// Finding patterns in interpreted assembunny assembly output.
///
/// ```text
///  cpy $N $r3
///  jnz $r2 2
///  jnz 1 6
///  dec $r2
///  dec $r3
///  jnz $r3 -4
///  inc $r1
///  jnz 1 -7
/// ```
/// can be replaced with `$r1 += $r2 / $N`, followed by `$r3 = $N - ($r2 % $N)` and `$r2 = 0`.
/// This reduces the number of simulated cycles for ~300 times when using the previously implemented
/// addition optimization, and ~400 times compared to a naive implementation.
///
/// See also [2016 day 12](crate::Day12) and [2016 day 23](crate::Day23).
#[derive(Clone, Debug)]
pub struct Day25 {
    interpreter: Interpreter<false, true>,
}

impl Day25 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            interpreter: Interpreter::new(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> i32 {
        let mut seen = HashSet::new();
        for i in 1..=i32::MAX {
            let mut last = 1;
            let mut found_solution = false;
            seen.clear();

            self.interpreter.execute([i, 0, 0, 0], |v, state| {
                if (last == 1 && v != 0) || (last == 0 && v != 1) {
                    return ControlFlow::Break(());
                }
                last = v;

                if seen.insert((v, state)) {
                    ControlFlow::Continue(())
                } else {
                    // Looped to previously seen state that produces the correct pattern
                    found_solution = true;
                    ControlFlow::Break(())
                }
            });

            if found_solution {
                return i;
            }
        }

        panic!("no solution found")
    }

    #[must_use]
    pub fn part2(&self) -> &'static str {
        "🎄"
    }
}

examples!(Day25 -> (i32, &'static str) []);
