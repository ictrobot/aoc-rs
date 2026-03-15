use crate::intcode::features::Day05Part2Features;
use crate::intcode::{Event, Interpreter};
use std::ops::Range;
use utils::prelude::*;

/// Finding the maximum output of an interpreter pipeline.
#[derive(Clone, Debug)]
pub struct Day07 {
    interpreter: Interpreter,
}

impl Day07 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            interpreter: Interpreter::parse(input, 1)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> i64 {
        let mut amplifier = self.interpreter.clone();
        Self::best_signal(0..5, |phases| {
            let mut signal = 0;
            for phase in phases {
                amplifier.clone_from(&self.interpreter);
                amplifier.push_input(phase);
                amplifier.push_input(signal);

                match amplifier.run::<Day05Part2Features>() {
                    Event::Halt => panic!("expected amplifier to output signal"),
                    Event::Input => panic!("expected amplifier to only consume two inputs"),
                    Event::Output(x) => signal = x,
                }
            }
            signal
        })
    }

    #[must_use]
    pub fn part2(&self) -> i64 {
        let mut amplifiers: [Interpreter; 5] = std::array::from_fn(|_| self.interpreter.clone());
        Self::best_signal(5..10, |phases| {
            for (amplifier, &phase) in amplifiers.iter_mut().zip(&phases) {
                amplifier.clone_from(&self.interpreter);
                amplifier.push_input(phase);
            }

            let mut signal = 0;
            loop {
                for amplifier in &mut amplifiers {
                    amplifier.push_input(signal);

                    match amplifier.run::<Day05Part2Features>() {
                        Event::Halt => return signal,
                        Event::Input => {
                            panic!("expected amplifier to only consume one input per cycle")
                        }
                        Event::Output(x) => signal = x,
                    }
                }
            }
        })
    }

    #[inline]
    fn best_signal(phases: Range<i64>, mut f: impl FnMut([i64; 5]) -> i64) -> i64 {
        let mut best = i64::MIN;
        for a in phases.start..phases.end {
            for b in phases.start..phases.end {
                if b == a {
                    continue;
                }
                for c in phases.start..phases.end {
                    if c == a || c == b {
                        continue;
                    }
                    for d in phases.start..phases.end {
                        if d == a || d == b || d == c {
                            continue;
                        }
                        for e in phases.start..phases.end {
                            if e == a || e == b || e == c || e == d {
                                continue;
                            }
                            best = best.max(f([a, b, c, d, e]));
                        }
                    }
                }
            }
        }
        best
    }
}

examples!(Day07 -> (i64, i64) [
    {
        input: "3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0",
        part1: 43210,
    },
    {
        input: "3,23,3,24,1002,24,10,24,1002,23,-1,23,101,5,23,23,1,24,23,23,4,23,99,0,0",
        part1: 54321,
    },
    {
        input: "3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0",
        part1: 65210,
    },
    {
        input: "3,26,1001,26,-4,26,3,27,1002,27,2,27,1,27,26,27,4,27,1001,28,-1,28,1005,28,6,99,0,0,5",
        part2: 139629729,
    },
    {
        input: "3,52,1001,52,-5,52,3,53,1,52,56,54,1007,54,5,55,1005,55,26,1001,54,-5,54,1105,1,12,1,53,54,53,1008,54,0,55,1001,55,1,55,2,53,55,53,4,53,1001,56,-1,56,1005,56,6,99,0,0,0,0,10",
        part2: 18216,
    },
]);
