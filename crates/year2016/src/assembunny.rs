//! Assembunny interpreter.
//!
//! See [`Day12`](crate::Day12), [`Day23`](crate::Day23) and [`Day25`](crate::Day25).

use std::ops::ControlFlow;
use utils::prelude::*;

#[derive(Clone, Debug)]
pub(crate) struct Interpreter<const TGL: bool, const OUT: bool> {
    instructions: Vec<Instruction>,
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
    Out(Register),
    Invalid2(Value, Value),
}

impl<const TGL: bool, const OUT: bool> Interpreter<TGL, OUT> {
    pub fn new(input: &str) -> Result<Self, InputError> {
        let register = parser::literal_map!(
            "a" => Register::A,
            "b" => Register::B,
            "c" => Register::C,
            "d" => Register::D,
        );
        let value = register
            .map(Value::Register)
            .or(parser::i32().map(Value::Number));

        Ok(Self {
            instructions: parser::parse_tree!(
                ("inc ", r @ register) => Instruction::Increment(r),
                ("dec ", r @ register) => Instruction::Decrement(r),
                ("cpy ", v @ value, " ", r @ register) => Instruction::Copy(v, r),
                ("jnz ", v @ value, " ", o @ value) => Instruction::JumpIfNotZero(v, o),
                ("tgl ", r @ register) =?> {
                    if TGL {
                        Ok(Instruction::Toggle(r))
                    } else {
                        Err("tgl instruction not supported")
                    }
                },
                ("out ", r @ register) =?> {
                    if OUT {
                        Ok(Instruction::Out(r))
                    } else {
                        Err("out instruction not supported")
                    }
                },
            )
            .parse_lines(input)?,
        })
    }
}

impl<const TGL: bool> Interpreter<TGL, false> {
    pub fn execute(&self, reg: [i32; 4]) -> i32 {
        execute(&self.instructions, reg, |_, _| unreachable!())
    }
}

impl<const TGL: bool> Interpreter<TGL, true> {
    pub fn execute(
        &self,
        reg: [i32; 4],
        out_fn: impl FnMut(i32, (usize, [i32; 4])) -> ControlFlow<()>,
    ) -> i32 {
        execute(&self.instructions, reg, out_fn)
    }
}

#[inline]
fn execute(
    instructions: &[Instruction],
    mut reg: [i32; 4],
    mut out_fn: impl FnMut(i32, (usize, [i32; 4])) -> ControlFlow<()>,
) -> i32 {
    let mut pc = 0;

    let mut instructions = instructions.to_vec();
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
            // Recognize the following pattern of instructions which can be simplified to division
            //  cpy $N $x
            //  jnz $y 2
            //  jnz 1 6
            //  dec $y
            //  dec $x
            //  jnz $x -4
            //  inc $z
            //  jnz 1 -7
            // This is the key optimisation for Day 25
            [
                Instruction::Copy(Value::Number(n), x),
                Instruction::JumpIfNotZero(Value::Register(y), Value::Number(2)),
                Instruction::JumpIfNotZero(Value::Number(1), Value::Number(6)),
                Instruction::Decrement(y2),
                Instruction::Decrement(x2),
                Instruction::JumpIfNotZero(Value::Register(x3), Value::Number(-4)),
                Instruction::Increment(z),
                Instruction::JumpIfNotZero(Value::Number(1), Value::Number(-7)),
                ..
            ] if x == x2 && x == x3 && y == y2 => {
                reg[z as usize] += reg[y as usize] / n;
                reg[x as usize] = n - (reg[y as usize] % n);
                reg[y as usize] = 0;
                pc += 8;
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
                    Instruction::Decrement(r) => Instruction::Increment(r),
                    Instruction::Toggle(r) => Instruction::Increment(r),
                    Instruction::Out(r) => Instruction::Increment(r),
                    Instruction::JumpIfNotZero(v, Value::Register(r)) => Instruction::Copy(v, r),
                    Instruction::JumpIfNotZero(v, o @ Value::Number(_)) => {
                        Instruction::Invalid2(v, o)
                    }
                    Instruction::Copy(v, r) => Instruction::JumpIfNotZero(v, Value::Register(r)),
                    Instruction::Invalid2(v1, v2) => {
                        // Effectively an invalid copy instruction
                        Instruction::JumpIfNotZero(v1, v2)
                    }
                };
            }
            Instruction::Out(r) => {
                if out_fn(reg[r as usize], (pc, reg)).is_break() {
                    break;
                }
            }
            Instruction::Invalid2(_, _) => {}
        }

        pc += 1;
    }

    reg[0]
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
