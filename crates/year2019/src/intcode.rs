//! Intcode interpreter.
//!
//! See [`Day02`](crate::Day02).

use utils::prelude::*;

#[derive(Clone, Debug)]
pub(crate) struct Interpreter {
    pub mem: Vec<i64>,
    pub ip: usize,
}

const OPCODE_ADD: i64 = 1;
const OPCODE_MUL: i64 = 2;
const OPCODE_HALT: i64 = 99;

impl Interpreter {
    pub fn new(input: &str, min_elements: usize) -> Result<Self, InputError> {
        Ok(Self {
            mem: parser::i64()
                .repeat(b',', min_elements)
                .parse_complete(input)?,
            ip: 0,
        })
    }

    #[inline]
    pub fn run(&mut self) {
        loop {
            let instruction = self.mem.get(self.ip).copied().unwrap_or(0);

            match instruction {
                OPCODE_ADD => {
                    let result = self.read_operand(1) + self.read_operand(2);
                    *self.write_operand(3) = result;
                }
                OPCODE_MUL => {
                    let result = self.read_operand(1) * self.read_operand(2);
                    *self.write_operand(3) = result;
                }
                OPCODE_HALT => break,
                _ => panic!("invalid instruction {instruction} at address {}", self.ip),
            }

            self.ip += 4;
        }
    }

    #[inline]
    fn read_operand(&self, offset: usize) -> i64 {
        let operand = self.ip + offset;
        let address = self.mem.get(operand).copied().unwrap_or(0);

        usize::try_from(address)
            .ok()
            .and_then(|address| self.mem.get(address).copied())
            .unwrap_or_else(|| panic!("invalid memory address {address} at address {operand}"))
    }

    #[inline]
    fn write_operand(&mut self, offset: usize) -> &mut i64 {
        let operand = self.ip + offset;
        let address = self.mem.get(operand).copied().unwrap_or(0);

        usize::try_from(address)
            .ok()
            .and_then(|address| self.mem.get_mut(address))
            .unwrap_or_else(|| panic!("invalid memory address {address} at address {operand}"))
    }
}
