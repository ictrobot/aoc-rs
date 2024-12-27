use utils::number::is_prime;
use utils::prelude::*;

/// Interpreting assembly to count composite numbers.
///
/// Uses a modified version of the interpreter from [day 18](crate::Day18).
#[derive(Clone, Debug)]
pub struct Day23 {
    instructions: Vec<Instruction>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Register {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Instruction {
    Set(Register, Register),
    SetN(Register, i32),
    Sub(Register, Register),
    SubN(Register, i32),
    Mul(Register, Register),
    MulN(Register, i32),
    JnzN(Register, i32),
    Jmp(i32), // Used for jnz $n where $n != 0
    Noop(),   // Used for jgz $n where $n == 0
}

impl Day23 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let register = parser::literal_map!(
            "a" => Register::A,
            "b" => Register::B,
            "c" => Register::C,
            "d" => Register::D,
            "e" => Register::E,
            "f" => Register::F,
            "g" => Register::G,
            "h" => Register::H,
        );

        let instruction = parser::parse_tree!(
            ("set ", r @ register, b' ') =>> {
                (r2 @ register) => Instruction::Set(r, r2),
                (v @ parser::i32()) => Instruction::SetN(r, v),
            },
            ("sub ", r @ register, b' ') =>> {
                (r2 @ register) => Instruction::Sub(r, r2),
                (v @ parser::i32()) => Instruction::SubN(r, v),
            },
            ("mul ", r @ register, b' ') =>> {
                (r2 @ register) => Instruction::Mul(r, r2),
                (v @ parser::i32()) => Instruction::MulN(r, v),
            },
            ("jnz ") =>> {
                (r @ register, b' ', v @parser::i32()) => Instruction::JnzN(r, v),
                (x @ parser::i32(), b' ', v @ parser::i32()) => {
                    if x != 0 {
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
    pub fn part1(&self) -> u32 {
        self.run(&mut [0; 8])
    }

    #[must_use]
    pub fn part2(&self) -> i64 {
        let mut reg = [1, 0, 0, 0, 0, 0, 0, 0];
        self.run(&mut reg);
        reg[Register::H as usize]
    }

    fn run(&self, reg: &mut [i64; 8]) -> u32 {
        let mut pc = 0;
        let mut mul_count = 0;
        while pc < self.instructions.len() {
            #[rustfmt::skip] // Rustfmt wants the pattern to be on a single long line
            match self.instructions[pc..] {
                // Recognize the naive prime check and replace it with a native implementation
                //   set $d 2
                //   set $e 2
                //   set $g $d
                //   mul $g $e
                //   sub $g $b
                //   jnz $g 2
                //   set $f 0
                //   sub $e -1
                //   set $g $e
                //   sub $g $b
                //   jnz $g -8
                //   sub $d -1
                //   set $g $d
                //   sub $g $b
                //   jnz $g -13
                [
                    Instruction::SetN(d, 2),
                    Instruction::SetN(e, 2),
                    Instruction::Set(g, d2),
                    Instruction::Mul(g2, e2),
                    Instruction::Sub(g3, b),
                    Instruction::JnzN(g4, 2),
                    Instruction::SetN(f, 0),
                    Instruction::SubN(e3, -1),
                    Instruction::Set(g5, e4),
                    Instruction::Sub(g6, b2),
                    Instruction::JnzN(g7, -8),
                    Instruction::SubN(d3, -1),
                    Instruction::Set(g8, d4),
                    Instruction::Sub(g9, b3),
                    Instruction::JnzN(g10, -13),
                    ..
                ] if b == b2 && b == b3 && reg[b as usize] >= 0
                    && d == d2 && d == d3 && d == d4
                    && e == e2 && e == e3 && e == e4
                    && g == g2 && g == g3 && g == g4 && g == g5&& g == g6 && g == g7 && g == g8 && g == g9 && g == g10
                => {
                    reg[d as usize] = reg[b as usize];
                    reg[e as usize] = reg[b as usize];
                    if !is_prime(reg[b as usize] as u64) {
                        reg[f as usize] = 0;
                    }
                    reg[g as usize] = 0;
                    pc += 15;
                    mul_count += (reg[b as usize] - 2).pow(2) as u32;
                    continue;
                }
                _ => {},
            };

            match self.instructions[pc] {
                Instruction::Set(r, r2) => reg[r as usize] = reg[r2 as usize],
                Instruction::SetN(r, v) => reg[r as usize] = v as i64,
                Instruction::Sub(r, r2) => reg[r as usize] -= reg[r2 as usize],
                Instruction::SubN(r, v) => reg[r as usize] -= v as i64,
                Instruction::Mul(r, r2) => {
                    reg[r as usize] *= reg[r2 as usize];
                    mul_count += 1;
                }
                Instruction::MulN(r, v) => {
                    reg[r as usize] *= v as i64;
                    mul_count += 1;
                }
                Instruction::JnzN(r, o) => {
                    if reg[r as usize] != 0 {
                        pc = pc.wrapping_add_signed(o as isize);
                        continue;
                    }
                }
                Instruction::Jmp(o) => {
                    pc = pc.wrapping_add_signed(o as isize);
                    continue;
                }
                Instruction::Noop() => {}
            }

            pc += 1;
        }
        mul_count
    }
}

examples!(Day23 -> (u32, i64) []);
