use std::collections::VecDeque;
use utils::prelude::*;

/// Interpreting assembly, with message passing between instances.
#[derive(Clone, Debug)]
pub struct Day18 {
    instructions: Vec<Instruction>,
}

// Use an enum for registers to avoid bounds checks when accessing the register array, slightly
// improving performance
#[derive(Copy, Clone, Debug, PartialEq)]
enum Register {
    A,
    B,
    C,
    D,
    F,
    I,
    P,
}

// Use separate enum variants for register and constant operands to minimize branching in the
// interpreter loop, again slightly improving performance
#[derive(Copy, Clone, Debug, PartialEq)]
enum Instruction {
    Snd(Register),
    SndN(i32), // Used in second example
    Set(Register, Register),
    SetN(Register, i32),
    Add(Register, Register),
    AddN(Register, i32),
    Mul(Register, Register), // Used in first example
    MulN(Register, i32),
    Mod(Register, Register),
    ModN(Register, i32),
    Rcv(Register),
    Jgz(Register, Register),
    JgzN(Register, i32),
    Jmp(i32), // Used for jgz $n where $n > 0
    Noop(),   // Used for jgz $n where $n <= 0
}

#[derive(Debug)]
struct Program<'a> {
    instructions: &'a [Instruction],
    pc: usize,
    reg: [i64; 7],
}

impl Day18 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let register = parser::literal_map!(
            "a" => Register::A,
            "b" => Register::B,
            "c" => Register::C,
            "d" => Register::D,
            "f" => Register::F,
            "i" => Register::I,
            "p" => Register::P,
        );

        let instruction = parser::parse_tree!(
            ("snd ") =>> {
                (r @ register) => Instruction::Snd(r),
                (v @ parser::i32()) => Instruction::SndN(v),
            },
            ("set ", r @ register, b' ') =>> {
                (r2 @ register) => Instruction::Set(r, r2),
                (v @ parser::i32()) => Instruction::SetN(r, v),
            },
            ("add ", r @ register, b' ') =>> {
                (r2 @ register) => Instruction::Add(r, r2),
                (v @ parser::i32()) => Instruction::AddN(r, v),
            },
            ("mul ", r @ register, b' ') =>> {
                (r2 @ register) => Instruction::Mul(r, r2),
                (v @ parser::i32()) => Instruction::MulN(r, v),
            },
            ("mod ", r @ register, b' ') =>> {
                (r2 @ register) => Instruction::Mod(r, r2),
                (v @ parser::i32()) => Instruction::ModN(r, v),
            },
            ("rcv ", r @ register) => Instruction::Rcv(r),
            ("jgz ") =>> {
                (r @ register, b' ') =>> {
                    (r2 @ register) => Instruction::Jgz(r, r2),
                    (v @ parser::i32()) => Instruction::JgzN(r, v),
                },
                (x @ parser::i32(), b' ', v @ parser::i32()) => {
                    if x > 0 {
                        Instruction::Jmp(v)
                    } else {
                        Instruction::Noop()
                    }
                },
            },
        );

        Ok(Self {
            instructions: instruction.parse_lines(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> i64 {
        let mut program = Program::from_instructions(&self.instructions);
        let mut last = None;

        program.run(
            |v| last = Some(v),
            |v| {
                if v == 0 {
                    // If the register is zero, do nothing/set the register back to zero
                    Some(0)
                } else {
                    // If the register is not zero, this is the first recovered frequency, so return
                    // None to halt program execution
                    None
                }
            },
        );

        last.unwrap()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut program0 = Program::from_instructions(&self.instructions);
        let mut inbound0 = VecDeque::new();

        let mut program1 = Program::from_instructions(&self.instructions);
        let mut inbound1 = VecDeque::new();
        let mut sent = 0;
        program1.reg[Register::P as usize] = 1;

        // Run both programs until they both fail to make progress. Use | instead of || so each
        // iteration always attempts to run both
        while program0.run(
            |v| {
                // Send to program 1
                inbound1.push_back(v)
            },
            |_| {
                // Receive from program 1
                inbound0.pop_front()
            },
        ) | program1.run(
            |v| {
                // Send to program 0, keeping track of how many values are sent
                sent += 1;
                inbound0.push_back(v)
            },
            |_| {
                // Receive from program 0
                inbound1.pop_front()
            },
        ) {}

        sent
    }
}

impl<'a> Program<'a> {
    fn from_instructions(instructions: &'a [Instruction]) -> Self {
        Program {
            instructions,
            pc: 0,
            reg: [0; 7],
        }
    }

    fn run(&mut self, mut snd: impl FnMut(i64), mut rcv: impl FnMut(i64) -> Option<i64>) -> bool {
        let mut progress = false;

        while let Some(&instruction) = self.instructions.get(self.pc) {
            match instruction {
                Instruction::Snd(r) => snd(self.reg[r as usize]),
                Instruction::SndN(v) => snd(v as i64),
                Instruction::Set(r, r2) => self.reg[r as usize] = self.reg[r2 as usize],
                Instruction::SetN(r, v) => self.reg[r as usize] = v as i64,
                Instruction::Add(r, r2) => self.reg[r as usize] += self.reg[r2 as usize],
                Instruction::AddN(r, v) => self.reg[r as usize] += v as i64,
                Instruction::Mul(r, r2) => self.reg[r as usize] *= self.reg[r2 as usize],
                Instruction::MulN(r, v) => self.reg[r as usize] *= v as i64,
                Instruction::Mod(r, r2) => {
                    self.reg[r as usize] = self.reg[r as usize].rem_euclid(self.reg[r2 as usize])
                }
                Instruction::ModN(r, v) => {
                    self.reg[r as usize] = self.reg[r as usize].rem_euclid(v as i64)
                }
                Instruction::Rcv(r) => {
                    if let Some(value) = rcv(self.reg[r as usize]) {
                        self.reg[r as usize] = value;
                    } else {
                        break;
                    }
                }
                Instruction::Jgz(r, ro) => {
                    if self.reg[r as usize] > 0 {
                        self.pc = self.pc.wrapping_add_signed(self.reg[ro as usize] as isize) - 1;
                    }
                }
                Instruction::JgzN(r, v) => {
                    if self.reg[r as usize] > 0 {
                        self.pc = self.pc.wrapping_add_signed(v as isize) - 1;
                    }
                }
                Instruction::Jmp(v) => {
                    self.pc = self.pc.wrapping_add(v as usize) - 1;
                }
                Instruction::Noop() => {}
            }

            self.pc += 1;
            progress = true;
        }

        progress
    }
}

examples!(Day18 -> (i64, u32) [
    {
        input: "set a 1\n\
            add a 2\n\
            mul a a\n\
            mod a 5\n\
            snd a\n\
            set a 0\n\
            rcv a\n\
            jgz a -1\n\
            set a 1\n\
            jgz a -2",
        part1: 4,
    },
    {
        input: "snd 1\n\
            snd 2\n\
            snd p\n\
            rcv a\n\
            rcv b\n\
            rcv c\n\
            rcv d",
        part2: 3,
    },
]);
