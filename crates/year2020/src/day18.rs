use utils::parser::{LeafResult, ParseError};
use utils::prelude::*;

/// Evaluating expressions under different precedence rules.
#[derive(Clone, Debug)]
pub struct Day18 {
    part1: u64,
    part2: u64,
}

impl Day18 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (part1, part2) = Self::expression
            .repeat_fold(
                parser::eol(),
                1,
                Some((0u64, 0u64)),
                |totals, (part1, part2)| {
                    let (total1, total2) = totals?;
                    Some((total1.checked_add(part1)?, total2.checked_add(part2)?))
                },
            )
            .map_res(|totals| totals.ok_or("total too large"))
            .parse_complete(input)?;
        Ok(Self { part1, part2 })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        self.part2
    }

    fn expression(mut input: &[u8]) -> LeafResult<'_, (u64, u64)> {
        let mut multiplication = false;
        let mut part1 = 0u64;
        let (mut current_sum, mut part2) = (0u64, 1u64);

        loop {
            let operand_input = input;
            let ((operand1, operand2), remaining) = match input {
                [digit @ b'0'..=b'9', rest @ ..] => {
                    let operand = u64::from(digit - b'0');
                    ((operand, operand), rest)
                }
                [b'(', rest @ ..] => match Self::expression(rest)? {
                    (operands, [b')', rest @ ..]) => (operands, rest),
                    (_, remaining) => return Err((ParseError::ExpectedByte(b')'), remaining)),
                },
                _ => return Err((ParseError::Expected("a digit or '('"), input)),
            };

            part1 = if multiplication {
                part1.checked_mul(operand1)
            } else {
                part1.checked_add(operand1)
            }
            .ok_or((ParseError::Custom("result too large"), operand_input))?;

            current_sum = current_sum
                .checked_add(operand2)
                .ok_or((ParseError::Custom("result too large"), operand_input))?;

            match remaining {
                [b' ', b'+', b' ', rest @ ..] => {
                    multiplication = false;
                    input = rest;
                }
                [b' ', b'*', b' ', rest @ ..] => {
                    multiplication = true;
                    part2 = part2
                        .checked_mul(current_sum)
                        .ok_or((ParseError::Custom("result too large"), remaining))?;
                    current_sum = 0;
                    input = rest;
                }
                [b')' | b'\n', ..] | [b'\r', b'\n', ..] | [] => {
                    let product = part2
                        .checked_mul(current_sum)
                        .ok_or((ParseError::Custom("result too large"), remaining))?;
                    return Ok(((part1, product), remaining));
                }
                _ => {
                    return Err((
                        ParseError::Custom("expected ' + ', ' * ', ')' or end of line"),
                        remaining,
                    ));
                }
            }
        }
    }
}

examples!(Day18 -> (u64, u64) [
    {input: "1 + 2 * 3 + 4 * 5 + 6", part1: 71, part2: 231},
    {input: "1 + (2 * 3) + (4 * (5 + 6))", part1: 51, part2: 51},
    {input: "2 * 3 + (4 * 5)", part1: 26, part2: 46},
    {input: "5 + (8 * 3 + 9 + 3 * 4 * 3)", part1: 437, part2: 1445},
    {input: "5 * 9 * (7 * 3 * 3 + 9 * 3 + (8 + 6 * 4))", part1: 12240, part2: 669060},
    {input: "((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2", part1: 13632, part2: 23340},
]);
