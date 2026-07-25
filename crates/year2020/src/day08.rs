use utils::prelude::*;

/// Interpreting assembly and fixing an infinite loop.
#[derive(Clone, Debug)]
pub struct Day08 {
    part1: i32,
    part2: i32,
}

#[derive(Copy, Clone, Debug)]
enum Instruction {
    Acc(i32),
    Jmp(i32),
    Nop(i32),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum State {
    Unvisited,
    Loops,
    Terminates,
}

impl Day08 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        if input.is_empty() {
            return Err(InputError::new(input, 0, "expected instruction"));
        }

        let instructions = parser::parse_tree!(
            ("acc ", v @ parser::i32()) => Instruction::Acc(v),
            ("jmp ", v @ parser::i32()) => Instruction::Jmp(v),
            ("nop ", v @ parser::i32()) => Instruction::Nop(v),
        )
        .parse_lines(input)?;

        let len = instructions.len();
        let mut outcomes = vec![State::Unvisited; len + 1];
        let mut alternatives = Vec::with_capacity(len);
        outcomes[len] = State::Terminates;

        let (mut pc, mut part1) = (0, 0);
        while outcomes.get(pc) == Some(&State::Unvisited) {
            outcomes[pc] = State::Loops;

            match instructions[pc] {
                Instruction::Acc(value) => {
                    part1 += value;
                    pc += 1;
                }
                Instruction::Jmp(offset) => {
                    alternatives.push((pc + 1, part1));
                    pc = pc.wrapping_add_signed(offset as isize);
                }
                Instruction::Nop(offset) => {
                    alternatives.push((pc.wrapping_add_signed(offset as isize), part1));
                    pc += 1;
                }
            };
        }

        if pc >= len {
            return Err(InputError::new(input, 0, "expected program to loop"));
        }

        let mut path = Vec::with_capacity(len);
        let mut suffix_sums = vec![0; len + 1];
        for &(alternate_pc, alternative_acc) in alternatives.iter().rev() {
            if alternate_pc > len {
                continue;
            }

            if outcomes[alternate_pc] == State::Unvisited {
                path.clear();
                pc = alternate_pc;
                while pc < len && outcomes[pc] == State::Unvisited {
                    outcomes[pc] = State::Loops;
                    path.push(pc);
                    pc = match instructions[pc] {
                        Instruction::Acc(_) | Instruction::Nop(_) => pc + 1,
                        Instruction::Jmp(offset) => {
                            pc.checked_add_signed(offset as isize).unwrap_or(usize::MAX)
                        }
                    };
                }

                if pc <= len && outcomes[pc] == State::Terminates {
                    let mut acc = suffix_sums[pc];
                    for &pc in path.iter().rev() {
                        if let Instruction::Acc(value) = instructions[pc] {
                            acc += value;
                        }
                        suffix_sums[pc] = acc;
                        outcomes[pc] = State::Terminates;
                    }
                }
            }

            if outcomes[alternate_pc] == State::Terminates {
                return Ok(Self {
                    part1,
                    part2: alternative_acc + suffix_sums[alternate_pc],
                });
            }
        }

        Err(InputError::new(
            input,
            0,
            "expected one corrupted instruction",
        ))
    }

    #[must_use]
    pub fn part1(&self) -> i32 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> i32 {
        self.part2
    }
}

examples!(Day08 -> (i32, i32) [
    {file: "day08_example0.txt", part1: 5, part2: 8},
]);
