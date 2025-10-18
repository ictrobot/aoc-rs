//! "Elfcode" interpreter.
//!
//! See [day 19](crate::Day19) and [day 21](crate::Day21).
use utils::prelude::*;

#[derive(Clone, Debug)]
pub(crate) struct Interpreter {
    instruction_pointer: Register,
    instructions: Vec<Instruction>,
}

// Avoids bounds checks when indexing the register array
utils::enumerable_enum! {
    #[repr(u32)]
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub(crate) enum Register {
        A,
        B,
        C,
        D,
        E,
        F,
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum Instruction {
    Addr(Register, Register, Register),
    Addi(Register, u32, Register),
    Mulr(Register, Register, Register),
    Muli(Register, u32, Register),
    Banr(Register, Register, Register),
    Bani(Register, u32, Register),
    Borr(Register, Register, Register),
    Bori(Register, u32, Register),
    Setr(Register, Register),
    Seti(u32, Register),
    Gtir(u32, Register, Register),
    Gtri(Register, u32, Register),
    Gtrr(Register, Register, Register),
    Eqir(u32, Register, Register),
    Eqri(Register, u32, Register),
    Eqrr(Register, Register, Register),
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum HookControlFlow {
    Execute,
    Next,
    Halt,
}

impl Interpreter {
    pub fn new(input: &str) -> Result<Self, InputError> {
        let register =
            parser::byte_range(b'0'..=b'5').map(|b| Register::from_discriminant((b - b'0') as u32));
        let rrr_instructions = parser::literal_map!(
            "addr " => Instruction::Addr as fn(_, _, _) -> _,
            "mulr " => Instruction::Mulr,
            "banr " => Instruction::Banr,
            "borr " => Instruction::Borr,
            "gtrr " => Instruction::Gtrr,
            "eqrr " => Instruction::Eqrr,
        );
        let rir_instructions = parser::literal_map!(
            "addi " => Instruction::Addi as fn(_, _, _) -> _,
            "muli " => Instruction::Muli,
            "bani " => Instruction::Bani,
            "bori " => Instruction::Bori,
            "gtri " => Instruction::Gtri,
            "eqri " => Instruction::Eqri,
        );
        let instruction = parser::parse_tree!(
            (i @ rrr_instructions, a @ register, b' ', b @ register, b' ', c @ register) => i(a, b, c),
            (i @ rir_instructions, a @ register, b' ', b @ parser::u32(), b' ', c @ register) => i(a, b, c),
            ("setr ", a @ register, b' ', parser::u32(), b' ', c @ register) => Instruction::Setr(a, c),
            ("seti ", a @ parser::u32(), b' ', parser::u32(), b' ', c @ register) => Instruction::Seti(a, c),
            ("gtir ", a @ parser::u32(), b' ', b @ register, b' ', c @ register) => Instruction::Gtir(a, b, c),
            ("eqir ", a @ parser::u32(), b' ', b @ register, b' ', c @ register) => Instruction::Eqir(a, b, c),
        );

        let (instruction_pointer, instructions) = register
            .with_prefix("#ip ")
            .with_suffix(parser::eol())
            .then(instruction.repeat(parser::eol(), 1))
            .parse_complete(input)?;

        Ok(Self {
            instruction_pointer,
            instructions,
        })
    }

    #[inline]
    pub fn run(
        &self,
        reg: &mut [u32; 6],
        mut hook: impl FnMut(&[Instruction], Register, &mut [u32; 6]) -> HookControlFlow,
    ) {
        while let addr = reg[self.instruction_pointer] as usize
            && addr < self.instructions.len()
        {
            match hook(&self.instructions, self.instruction_pointer, reg) {
                HookControlFlow::Execute => {}
                HookControlFlow::Next => continue,
                HookControlFlow::Halt => return,
            }

            match self.instructions[addr] {
                Instruction::Addr(a, b, c) => reg[c] = reg[a].wrapping_add(reg[b]),
                Instruction::Addi(a, b, c) => reg[c] = reg[a].wrapping_add(b),
                Instruction::Mulr(a, b, c) => reg[c] = reg[a].wrapping_mul(reg[b]),
                Instruction::Muli(a, b, c) => reg[c] = reg[a].wrapping_mul(b),
                Instruction::Banr(a, b, c) => reg[c] = reg[a] & reg[b],
                Instruction::Bani(a, b, c) => reg[c] = reg[a] & b,
                Instruction::Borr(a, b, c) => reg[c] = reg[a] | reg[b],
                Instruction::Bori(a, b, c) => reg[c] = reg[a] | b,
                Instruction::Setr(a, c) => reg[c] = reg[a],
                Instruction::Seti(a, c) => reg[c] = a,
                Instruction::Gtir(a, b, c) => reg[c] = u32::from(a > reg[b]),
                Instruction::Gtri(a, b, c) => reg[c] = u32::from(reg[a] > b),
                Instruction::Gtrr(a, b, c) => reg[c] = u32::from(reg[a] > reg[b]),
                Instruction::Eqir(a, b, c) => reg[c] = u32::from(a == reg[b]),
                Instruction::Eqri(a, b, c) => reg[c] = u32::from(reg[a] == b),
                Instruction::Eqrr(a, b, c) => reg[c] = u32::from(reg[a] == reg[b]),
            }

            reg[self.instruction_pointer] += 1;
        }
    }

    #[inline]
    #[must_use]
    pub fn instruction_pointer(&self) -> Register {
        self.instruction_pointer
    }

    #[inline]
    #[must_use]
    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }
}

impl Instruction {
    #[inline]
    pub fn registers(self) -> impl Iterator<Item = Register> {
        match self {
            Instruction::Addr(r1, r2, r3)
            | Instruction::Mulr(r1, r2, r3)
            | Instruction::Banr(r1, r2, r3)
            | Instruction::Borr(r1, r2, r3)
            | Instruction::Gtrr(r1, r2, r3)
            | Instruction::Eqrr(r1, r2, r3) => [r1, r2, r3].into_iter().take(3),
            Instruction::Addi(r1, _, r2)
            | Instruction::Muli(r1, _, r2)
            | Instruction::Bani(r1, _, r2)
            | Instruction::Bori(r1, _, r2)
            | Instruction::Setr(r1, r2)
            | Instruction::Gtir(_, r1, r2)
            | Instruction::Gtri(r1, _, r2)
            | Instruction::Eqir(_, r1, r2)
            | Instruction::Eqri(r1, _, r2) => [r1, r2, Register::A].into_iter().take(2),
            Instruction::Seti(_, r1) => [r1, Register::A, Register::A].into_iter().take(1),
        }
    }
}
