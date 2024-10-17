//! Assembunny interpreter.
//!
//! See [`Day12`](crate::Day12) and [`Day23`](crate::Day23).

use std::marker::PhantomData;
use utils::prelude::*;

pub(crate) trait InterpreterConfig {
    const SUPPORTS_TOGGLE: bool = false;
}

#[derive(Clone, Debug)]
pub(crate) struct Interpreter<C: InterpreterConfig> {
    instructions: Vec<Instruction>,
    phantom: PhantomData<C>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Register {
    A,
    B,
    C,
    D,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Value {
    Register(Register),
    Number(i32),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Instruction {
    Copy(Value, Register),
    Increment(Register),
    Decrement(Register),
    JumpIfNotZero(Value, Value),
    Toggle(Register),
    Invalid2(Value, Value),
}

impl<C: InterpreterConfig> Interpreter<C> {
    pub fn new(input: &str) -> Result<Self, InputError> {
        let register = parser::one_of((
            b'a'.map(|_| Register::A),
            b'b'.map(|_| Register::B),
            b'c'.map(|_| Register::C),
            b'd'.map(|_| Register::D),
        ));
        let value = register
            .map(Value::Register)
            .or(parser::i32().map(Value::Number));

        Ok(Self {
            instructions: parser::one_of((
                register.with_prefix("inc ").map(Instruction::Increment),
                register.with_prefix("dec ").map(Instruction::Decrement),
                value
                    .with_prefix("cpy ")
                    .then(register.with_prefix(" "))
                    .map(|(v, r)| Instruction::Copy(v, r)),
                value
                    .with_prefix("jnz ")
                    .then(value.with_prefix(" "))
                    .map(|(v, o)| Instruction::JumpIfNotZero(v, o)),
                register.with_prefix("tgl ").map_res(|r| {
                    if C::SUPPORTS_TOGGLE {
                        Ok(Instruction::Toggle(r))
                    } else {
                        Err("toggle instruction not supported")
                    }
                }),
            ))
            .parse_lines(input)?,
            phantom: PhantomData,
        })
    }

    pub fn execute(&self, mut reg: [i32; 4]) -> i32 {
        let mut pc = 0;

        let mut instructions = self.instructions.clone();
        while pc < instructions.len() {
            #[rustfmt::skip] // Rustfmt wants each pattern to be on a single really long line
            match instructions[pc..] {
                // Recognize the following pattern of instructions which can be simplified to addition
                //  inc $x
                //  dec $y
                //  jnz $y -2
                // This is the key optimization for Day 12
                [
                    Instruction::Increment(x),
                    Instruction::Decrement(y),
                    Instruction::JumpIfNotZero(Value::Register(y2), Value::Number(-2)),
                    ..
                ] if y == y2 => {
                    reg[x as usize] += reg[y as usize];
                    reg[y as usize] = 0;
                    pc += 3;
                    continue;
                }
                // Recognize the following pattern of instructions which can be simplified to multiplication
                //  cpy $w $x
                //  inc $y
                //  dec $x
                //  jnz $x -2
                //  dec $z
                //  jnz $z -5
                // This is the key optimisation for Day 23
                [
                    Instruction::Copy(Value::Register(w), x),
                    Instruction::Increment(y),
                    Instruction::Decrement(x2),
                    Instruction::JumpIfNotZero(Value::Register(x3), Value::Number(-2)),
                    Instruction::Decrement(z),
                    Instruction::JumpIfNotZero(Value::Register(z2), Value::Number(-5)),
                    ..
                ] if x == x2 && x == x3 && z == z2 => {
                    reg[y as usize] = reg[w as usize] * reg[z as usize];
                    reg[x as usize] = 0;
                    reg[z as usize] = 0;
                    pc += 6;
                    continue;
                }
                _ => {}
            };

            match instructions[pc] {
                Instruction::Copy(v, dst) => reg[dst as usize] = v.get(&reg),
                Instruction::Increment(dst) => reg[dst as usize] += 1,
                Instruction::Decrement(dst) => reg[dst as usize] -= 1,
                Instruction::JumpIfNotZero(v, offset) => {
                    if v.get(&reg) != 0 {
                        let offset = offset.get(&reg);
                        let Some(new_pc) = pc.checked_add_signed(offset as isize) else {
                            break;
                        };
                        pc = new_pc;
                        continue;
                    }
                }
                Instruction::Toggle(r) => 'toggle: {
                    let Some(index) = pc.checked_add_signed(reg[r as usize] as isize) else {
                        break 'toggle;
                    };
                    if index >= instructions.len() {
                        break 'toggle;
                    }

                    instructions[index] = match instructions[index] {
                        Instruction::Increment(r) => Instruction::Decrement(r),
                        Instruction::Decrement(r) | Instruction::Toggle(r) => {
                            Instruction::Increment(r)
                        }
                        Instruction::JumpIfNotZero(v, Value::Register(r)) => {
                            Instruction::Copy(v, r)
                        }
                        Instruction::JumpIfNotZero(v, o @ Value::Number(_)) => {
                            Instruction::Invalid2(v, o)
                        }
                        Instruction::Copy(v, r) => {
                            Instruction::JumpIfNotZero(v, Value::Register(r))
                        }
                        Instruction::Invalid2(v1, v2) => {
                            // Effectively an invalid copy instruction
                            Instruction::JumpIfNotZero(v1, v2)
                        }
                    };
                }
                Instruction::Invalid2(_, _) => {}
            }

            pc += 1;
        }

        reg[0]
    }
}

impl Value {
    #[inline]
    fn get(self, registers: &[i32; 4]) -> i32 {
        match self {
            Value::Register(r) => registers[r as usize],
            Value::Number(n) => n,
        }
    }
}
