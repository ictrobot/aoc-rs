use utils::prelude::*;

/// Interpreting assembly.
#[derive(Clone, Debug)]
pub struct Day23 {
    instructions: Vec<Instruction>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Register {
    A,
    B,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Instruction {
    Half(Register),
    Triple(Register),
    Increment(Register),
    Jump(i16),
    JumpIfEven(Register, i16),
    JumpIfOne(Register, i16),
}

impl Day23 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let register = parser::literal_map!(
            "a" => Register::A,
            "b" => Register::B,
        );

        Ok(Self {
            instructions: parser::parse_tree!(
                ("hlf ", r @ register) => Instruction::Half(r),
                ("tpl ", r @ register) => Instruction::Triple(r),
                ("inc ", r @ register) => Instruction::Increment(r),
                ("jmp ", v @ parser::i16()) => Instruction::Jump(v),
                ("jie ", r @ register, ", ", o @ parser::i16()) => Instruction::JumpIfEven(r, o),
                ("jio ", r @ register, ", ", o @ parser::i16()) => Instruction::JumpIfOne(r, o),
            )
            .parse_lines(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        self.execute(0, 0)
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        self.execute(1, 0)
    }

    fn execute(&self, mut a: u64, mut b: u64) -> u64 {
        let mut pc = 0;

        while let Some(&instruction) = pc
            .try_into()
            .ok()
            .and_then(|i: usize| self.instructions.get(i))
        {
            pc += 1;
            match instruction {
                Instruction::Half(Register::A) => a /= 2,
                Instruction::Half(Register::B) => b /= 2,
                Instruction::Triple(Register::A) => a *= 3,
                Instruction::Triple(Register::B) => b *= 3,
                Instruction::Increment(Register::A) => a += 1,
                Instruction::Increment(Register::B) => b += 1,
                Instruction::Jump(offset) => pc += offset - 1,
                Instruction::JumpIfEven(Register::A, offset) if a.is_multiple_of(2) => {
                    pc += offset - 1
                }
                Instruction::JumpIfEven(Register::B, offset) if b.is_multiple_of(2) => {
                    pc += offset - 1
                }
                Instruction::JumpIfOne(Register::A, offset) if a == 1 => pc += offset - 1,
                Instruction::JumpIfOne(Register::B, offset) if b == 1 => pc += offset - 1,
                Instruction::JumpIfEven(_, _) | Instruction::JumpIfOne(_, _) => {}
            }
        }

        b
    }
}

examples!(Day23 -> (u64, u64) []);
