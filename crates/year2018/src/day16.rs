use utils::prelude::*;

/// Inferring opcode operations from examples.
///
/// See also [day 19](crate::Day19) and [day 21](crate::Day21).
#[derive(Clone, Debug)]
pub struct Day16 {
    samples: Vec<(u32, u16)>,
    instructions: Vec<(u32, [u32; 3])>,
}

utils::enumerable_enum! {
    #[repr(u32)]
    #[derive(Copy, Clone, Debug)]
    enum Operation {
        Addr,
        Addi,
        Mulr,
        Muli,
        Banr,
        Bani,
        Borr,
        Bori,
        Setr,
        Seti,
        Gtir,
        Gtri,
        Gtrr,
        Eqir,
        Eqri,
        Eqrr,
    }
}

impl Day16 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let registers = parser::digit().map(u32::from).repeat_n::<4, _>(", ");
        let instruction = parser::number_range(0u32..=15)
            .with_suffix(b' ')
            .then(parser::number_range(0u32..=3).repeat_n::<3, _>(b' '));
        let sample = registers
            .with_prefix("Before: [")
            .with_suffix("]".then(parser::eol()))
            .then(instruction.with_suffix(parser::eol()))
            .then(
                registers
                    .with_prefix("After:  [")
                    .with_suffix("]".then(parser::eol())),
            )
            .map(|(before, (opcode, [a, b, c]), after)| {
                (
                    opcode,
                    // Convert the before and after registers and the operands into a mask of
                    // possible operations as used in both parts.
                    Operation::iter().fold(0, |acc, op| {
                        let possible = Self::execute(op, a, b, &before) == after[c as usize];
                        acc | (possible as u16) << op as u32
                    }),
                )
            });

        let (samples, instructions) = sample
            .repeat(parser::eol(), 1)
            .with_eol()
            .with_eol()
            .with_eol()
            .then(instruction.repeat(parser::eol(), 1))
            .parse_complete(input)?;

        Ok(Self {
            samples,
            instructions,
        })
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        self.samples
            .iter()
            .filter(|&(_, mask)| mask.count_ones() >= 3)
            .count()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut op_masks = [u16::MAX; 16];
        for &(opcode, mask) in &self.samples {
            op_masks[opcode as usize] &= mask;
        }

        let mut ops = [Operation::Addi; 16];
        for _ in 0..16 {
            let opcode = op_masks
                .iter()
                .position(|&mask| mask.count_ones() == 1)
                .expect("no solution found: all remaining opcodes could be multiple operations");

            let mask = op_masks[opcode];
            ops[opcode] = Operation::from_discriminant(mask.trailing_zeros());

            op_masks.iter_mut().for_each(|m| *m &= !mask);
        }

        let mut registers = [0; 4];
        for &(opcode, [a, b, c]) in &self.instructions {
            registers[c as usize] = Self::execute(ops[opcode as usize], a, b, &registers);
        }
        registers[0]
    }

    #[inline]
    fn execute(op: Operation, a: u32, b: u32, registers: &[u32; 4]) -> u32 {
        match op {
            Operation::Addr => registers[a as usize] + registers[b as usize],
            Operation::Addi => registers[a as usize] + b,
            Operation::Mulr => registers[a as usize] * registers[b as usize],
            Operation::Muli => registers[a as usize] * b,
            Operation::Banr => registers[a as usize] & registers[b as usize],
            Operation::Bani => registers[a as usize] & b,
            Operation::Borr => registers[a as usize] | registers[b as usize],
            Operation::Bori => registers[a as usize] | b,
            Operation::Setr => registers[a as usize],
            Operation::Seti => a,
            Operation::Gtir => u32::from(a > registers[b as usize]),
            Operation::Gtri => u32::from(registers[a as usize] > b),
            Operation::Gtrr => u32::from(registers[a as usize] > registers[b as usize]),
            Operation::Eqir => u32::from(a == registers[b as usize]),
            Operation::Eqri => u32::from(registers[a as usize] == b),
            Operation::Eqrr => u32::from(registers[a as usize] == registers[b as usize]),
        }
    }
}

examples!(Day16 -> (usize, u32) []);
