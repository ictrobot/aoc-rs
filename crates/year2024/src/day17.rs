use std::ops::ControlFlow;
use utils::prelude::*;

/// Interpreting 3-bit assembly.
///
/// Part 2 assumes the input is structured such that the Nth digit in the output depends only on
/// bits N*3 onwards. This enables working backwards from the right digit, checking 8 possible
/// 3-bit patterns for each digit.
#[derive(Clone, Debug)]
pub struct Day17 {
    a_start: u64,
    program: Vec<u8>,
}

impl Day17 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (a_start, b_start, c_start, program) = parser::u64()
            .with_prefix("Register A: ")
            .then(parser::u64().with_prefix(parser::eol().then("Register B: ")))
            .then(parser::u64().with_prefix(parser::eol().then("Register C: ")))
            .then(
                parser::number_range(0..=7)
                    .repeat(b',', 2)
                    .with_prefix(parser::eol().then(parser::eol()).then("Program: ")),
            )
            .parse_complete(input)?;

        if b_start != 0 || c_start != 0 {
            return Err(InputError::new(input, 0, "expected B and C to start at 0"));
        }

        Ok(Self { a_start, program })
    }

    #[must_use]
    pub fn part1(&self) -> String {
        let mut output = String::new();

        self.run(self.a_start, |x| {
            if !output.is_empty() {
                output.push(',');
            }
            output.push((b'0' + x) as char);
            ControlFlow::Continue(())
        });

        output
    }

    #[inline]
    fn run(&self, mut a: u64, mut out_fn: impl FnMut(u8) -> ControlFlow<()>) {
        let mut b = 0;
        let mut c = 0;
        let mut pc = 0;

        while pc + 1 < self.program.len() {
            let opcode = self.program[pc];
            let operand = self.program[pc + 1] as u64;
            let combo_operand = || match operand {
                0..=3 => operand,
                4 => a,
                5 => b,
                6 => c,
                7 => panic!("combo operand 7 is reserved"),
                _ => unreachable!(),
            };

            match opcode {
                0 => a /= 1 << combo_operand(), // adv
                1 => b ^= operand,              // bxl
                2 => b = combo_operand() % 8,   // bst
                3 => {
                    // jnz
                    if a != 0 {
                        pc = operand as usize;
                        continue;
                    }
                }
                4 => b ^= c, // bxc
                5 => {
                    // out
                    if out_fn((combo_operand() % 8) as u8) == ControlFlow::Break(()) {
                        return;
                    }
                }
                6 => b = a / (1 << combo_operand()), // bdv
                7 => c = a / (1 << combo_operand()), // cdv
                _ => unreachable!(),
            }

            pc += 2;
        }
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        self.search(0, self.program.len() - 1)
            .expect("no solution found")
    }

    fn search(&self, base: u64, pos: usize) -> Option<u64> {
        if pos == 0 {
            for x in 0..8 {
                let a = base | x;
                if self.check_all_matches(a) {
                    return Some(a);
                }
            }
            return None;
        }

        for x in 0..8 {
            let a = base | (x << (pos * 3));
            if self.check_pos_matches(a, pos)
                && let Some(result) = self.search(a, pos - 1)
            {
                return Some(result);
            }
        }
        None
    }

    fn check_pos_matches(&self, a: u64, pos: usize) -> bool {
        let mut output = None;
        self.run(a / (1 << (pos * 3)), |out| {
            output = Some(out);
            ControlFlow::Break(())
        });
        output == Some(self.program[pos])
    }

    fn check_all_matches(&self, a: u64) -> bool {
        let mut count = 0;
        self.run(a, |out| {
            if count >= self.program.len() || self.program[count] != out {
                ControlFlow::Break(())
            } else {
                count += 1;
                ControlFlow::Continue(())
            }
        });
        count == self.program.len()
    }
}

examples!(Day17 -> (&'static str, u64) [
    {
        input: "Register A: 729\n\
            Register B: 0\n\
            Register C: 0\n\
            \n\
            Program: 0,1,5,4,3,0",
        part1: "4,6,3,5,6,3,5,2,1,0"
    },
    {
        input: "Register A: 2024\n\
            Register B: 0\n\
            Register C: 0\n\
            \n\
            Program: 0,3,5,4,3,0",
        part2: 117440,
    },
]);
