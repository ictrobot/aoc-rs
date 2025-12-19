//! Intcode interpreter.
//!
//! See [`Day02`](crate::Day02), [`Day05`](crate::Day05).

use std::collections::VecDeque;
use utils::prelude::*;

#[derive(Clone, Debug)]
pub(crate) struct Interpreter {
    pub mem: Vec<i64>,
    pub ip: usize,
    pub input: VecDeque<i64>,
}

const OPCODE_ADD: i64 = 1;
const OPCODE_MUL: i64 = 2;
const OPCODE_INPUT: i64 = 3;
const OPCODE_OUTPUT: i64 = 4;
const OPCODE_JMP_NON_ZERO: i64 = 5;
const OPCODE_JMP_ZERO: i64 = 6;
const OPCODE_LESS_THAN: i64 = 7;
const OPCODE_EQUALS: i64 = 8;
const OPCODE_HALT: i64 = 99;

const POSITION_MODE: u32 = 0;
const IMMEDIATE_MODE: u32 = 1;

impl Interpreter {
    pub fn new(mem: Vec<i64>) -> Self {
        Self {
            mem,
            ip: 0,
            input: VecDeque::new(),
        }
    }

    pub fn parse(input: &str, min_elements: usize) -> Result<Self, InputError> {
        Ok(Self::new(
            parser::i64()
                .repeat(b',', min_elements)
                .parse_complete(input)?,
        ))
    }

    #[inline]
    pub fn push_input(&mut self, value: i64) {
        self.input.push_back(value);
    }

    #[inline]
    pub fn run<F: Features>(&mut self) -> Event {
        loop {
            let instruction = self.mem.get(self.ip).copied().unwrap_or(0);

            if !(0..=99999).contains(&instruction) {
                // 2 digits for opcode, 3x 1 digit for opcode mode
                panic!("invalid instruction {instruction} at address {}", self.ip);
            }
            let opcode = instruction % 100;

            match opcode {
                OPCODE_ADD => {
                    let result = self.read_operand::<F>(1) + self.read_operand::<F>(2);
                    *self.write_operand::<F>(3) = result;
                    self.ip += 4;
                }
                OPCODE_MUL => {
                    let result = self.read_operand::<F>(1) * self.read_operand::<F>(2);
                    *self.write_operand::<F>(3) = result;
                    self.ip += 4;
                }
                OPCODE_INPUT if F::IO_OPCODES => {
                    let Some(value) = self.input.pop_front() else {
                        break Event::Input;
                    };
                    *self.write_operand::<F>(1) = value;
                    self.ip += 2;
                }
                OPCODE_OUTPUT if F::IO_OPCODES => {
                    let value = self.read_operand::<F>(1);
                    self.ip += 2;
                    break Event::Output(value);
                }
                OPCODE_JMP_NON_ZERO if F::CONDITIONAL_OPCODES => {
                    if self.read_operand::<F>(1) != 0 {
                        let value = self.read_operand::<F>(2);
                        self.ip = usize::try_from(value).unwrap_or_else(|_| {
                            panic!("invalid jump address {value} at address {}", self.ip + 2)
                        });
                    } else {
                        self.ip += 3;
                    }
                }
                OPCODE_JMP_ZERO if F::CONDITIONAL_OPCODES => {
                    if self.read_operand::<F>(1) == 0 {
                        let value = self.read_operand::<F>(2);
                        self.ip = usize::try_from(value).unwrap_or_else(|_| {
                            panic!("invalid jump address {value} at address {}", self.ip + 2)
                        });
                    } else {
                        self.ip += 3;
                    }
                }
                OPCODE_LESS_THAN if F::CONDITIONAL_OPCODES => {
                    let result = self.read_operand::<F>(1) < self.read_operand::<F>(2);
                    *self.write_operand::<F>(3) = i64::from(result);
                    self.ip += 4;
                }
                OPCODE_EQUALS if F::CONDITIONAL_OPCODES => {
                    let result = self.read_operand::<F>(1) == self.read_operand::<F>(2);
                    *self.write_operand::<F>(3) = i64::from(result);
                    self.ip += 4;
                }
                OPCODE_HALT => break Event::Halt,
                _ => panic!("invalid opcode {opcode} at address {}", self.ip),
            }
        }
    }

    #[inline(always)]
    fn read_operand<F: Features>(&self, offset: usize) -> i64 {
        let operand_address = self.ip + offset;
        let operand = self.mem.get(operand_address).copied().unwrap_or(0);

        match self.operand_mode(offset) {
            POSITION_MODE => usize::try_from(operand)
                .ok()
                .and_then(|address| self.mem.get(address).copied())
                .unwrap_or_else(|| {
                    panic!("invalid memory address {operand} at address {operand_address}")
                }),
            IMMEDIATE_MODE if F::IMMEDIATE_OPERANDS => operand,
            mode => panic!("invalid operand mode {mode} at address {operand_address}"),
        }
    }

    #[inline(always)]
    fn write_operand<F: Features>(&mut self, offset: usize) -> &mut i64 {
        let operand_address = self.ip + offset;
        let operand = self.mem.get(operand_address).copied().unwrap_or(0);

        match self.operand_mode(offset) {
            POSITION_MODE => usize::try_from(operand)
                .ok()
                .and_then(|address| self.mem.get_mut(address))
                .unwrap_or_else(|| {
                    panic!("invalid memory address {operand} at address {operand_address}")
                }),
            mode => panic!("invalid write operand mode {mode} at address {operand_address}"),
        }
    }

    #[inline(always)]
    fn operand_mode(&self, offset: usize) -> u32 {
        (self.mem[self.ip] as u32 / 10u32.pow(1 + offset as u32)) % 10
    }
}

#[derive(Debug, Clone, Copy)]
#[must_use]
pub enum Event {
    Halt,
    Input,
    Output(i64),
}

pub(crate) trait Features {
    /// Introduced in Day 5 Part 1
    const IO_OPCODES: bool = false;
    /// Introduced in Day 5 Part 2
    const CONDITIONAL_OPCODES: bool = false;
    /// Introduced in Day 5 Part 1
    const IMMEDIATE_OPERANDS: bool = false;
}

pub mod features {
    use crate::intcode::Features;

    pub struct Day02Features;
    impl Features for Day02Features {}

    pub struct Day05Part1Features;
    impl Features for Day05Part1Features {
        const IO_OPCODES: bool = true;
        const IMMEDIATE_OPERANDS: bool = true;
    }

    pub struct Day05Part2Features;
    impl Features for Day05Part2Features {
        const IO_OPCODES: bool = true;
        const CONDITIONAL_OPCODES: bool = true;
        const IMMEDIATE_OPERANDS: bool = true;
    }
}

#[cfg(test)]
mod tests {
    use super::features::*;
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    enum TestEvent {
        Input(i64),
        Output(i64),
    }

    fn interpreter_test<F: Features>(
        initial_memory: Vec<i64>,
        final_memory: Vec<i64>,
        events: Vec<TestEvent>,
    ) {
        let mut interpreter = Interpreter::new(initial_memory);
        let mut events = events.into_iter();
        loop {
            match (interpreter.run::<F>(), events.next()) {
                (Event::Halt, None) => break,
                (Event::Input, Some(TestEvent::Input(value))) => interpreter.push_input(value),
                (Event::Output(value), Some(TestEvent::Output(expected))) => {
                    assert_eq!(value, expected)
                }
                (event, expected) => {
                    panic!("unexpected event: expected {expected:?}, got {event:?}")
                }
            }
        }

        if !final_memory.is_empty() {
            assert_eq!(interpreter.mem, final_memory);
        }
        assert_eq!(interpreter.input.len(), 0);
    }

    #[test]
    fn day02_part1_examples() {
        interpreter_test::<Day02Features>(
            vec![1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50],
            vec![3500, 9, 10, 70, 2, 3, 11, 0, 99, 30, 40, 50],
            vec![],
        );

        interpreter_test::<Day02Features>(vec![1, 0, 0, 0, 99], vec![2, 0, 0, 0, 99], vec![]);

        interpreter_test::<Day02Features>(vec![2, 3, 0, 3, 99], vec![2, 3, 0, 6, 99], vec![]);

        interpreter_test::<Day02Features>(
            vec![2, 4, 4, 5, 99, 0],
            vec![2, 4, 4, 5, 99, 9801],
            vec![],
        );

        interpreter_test::<Day02Features>(
            vec![1, 1, 1, 4, 99, 5, 6, 0, 99],
            vec![30, 1, 1, 4, 2, 5, 6, 0, 99],
            vec![],
        );
    }

    #[test]
    fn day05_part1_examples() {
        interpreter_test::<Day05Part1Features>(
            vec![3, 0, 4, 0, 99],
            vec![42, 0, 4, 0, 99],
            vec![TestEvent::Input(42), TestEvent::Output(42)],
        );

        interpreter_test::<Day05Part1Features>(
            vec![1002, 4, 3, 4, 33],
            vec![1002, 4, 3, 4, 99],
            vec![],
        );

        interpreter_test::<Day05Part1Features>(
            vec![1101, 100, -1, 4, 0],
            vec![1101, 100, -1, 4, 99],
            vec![],
        );
    }

    #[test]
    fn day05_part2_examples() {
        for input in -10..=20 {
            let equal_to = i64::from(input == 8);
            let less_than = i64::from(input < 8);

            interpreter_test::<Day05Part2Features>(
                vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8],
                vec![3, 9, 8, 9, 10, 9, 4, 9, 99, equal_to, 8],
                vec![TestEvent::Input(input), TestEvent::Output(equal_to)],
            );
            interpreter_test::<Day05Part2Features>(
                vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8],
                vec![3, 9, 7, 9, 10, 9, 4, 9, 99, less_than, 8],
                vec![TestEvent::Input(input), TestEvent::Output(less_than)],
            );

            interpreter_test::<Day05Part2Features>(
                vec![3, 3, 1108, -1, 8, 3, 4, 3, 99],
                vec![3, 3, 1108, equal_to, 8, 3, 4, 3, 99],
                vec![TestEvent::Input(input), TestEvent::Output(equal_to)],
            );
            interpreter_test::<Day05Part2Features>(
                vec![3, 3, 1107, -1, 8, 3, 4, 3, 99],
                vec![3, 3, 1107, less_than, 8, 3, 4, 3, 99],
                vec![TestEvent::Input(input), TestEvent::Output(less_than)],
            );
        }

        for input in -10..=20 {
            let output = i64::from(input != 0);

            interpreter_test::<Day05Part2Features>(
                vec![3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9],
                vec![
                    3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, input, output, 1, 9,
                ],
                vec![TestEvent::Input(input), TestEvent::Output(output)],
            );

            interpreter_test::<Day05Part2Features>(
                vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1],
                vec![3, 3, 1105, input, 9, 1101, 0, 0, 12, 4, 12, 99, output],
                vec![TestEvent::Input(input), TestEvent::Output(output)],
            );
        }

        for input in -10..=20 {
            interpreter_test::<Day05Part2Features>(
                vec![
                    3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0,
                    36, 98, 0, 0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46,
                    1101, 1000, 1, 20, 4, 20, 1105, 1, 46, 98, 99,
                ],
                vec![],
                vec![
                    TestEvent::Input(input),
                    TestEvent::Output(if input < 8 {
                        999
                    } else if input == 8 {
                        1000
                    } else {
                        1001
                    }),
                ],
            )
        }
    }
}
