use utils::parser::{ParseError, ParseResult};
use utils::prelude::*;

/// JSON document numbers.
#[derive(Clone, Debug)]
pub struct Day12 {
    part1: i32,
    part2: i32,
}

impl Day12 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (part1, part2) = match Self::parse(input.as_bytes()) {
            Ok(((part1, part2), &[])) => Ok((part1, part2)),
            Ok((_, remaining)) => Err(InputError::new(input, remaining, "expected end of input")),
            Err((err, position)) => Err(InputError::new(input, position, err)),
        }?;
        Ok(Self { part1, part2 })
    }

    #[must_use]
    pub fn part1(&self) -> i32 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> i32 {
        self.part2
    }

    fn parse(input: &[u8]) -> ParseResult<(i32, i32)> {
        match input {
            [b'{', ..] => Self::parse_object(&input[1..]),
            [b'[', ..] => Self::parse_array(&input[1..]),
            _ => Err((ParseError::Expected("'{' or '['"), input)),
        }
    }

    fn parse_object(mut input: &[u8]) -> ParseResult<(i32, i32)> {
        let (mut part1, mut part2) = (0, 0);
        let mut red = false;
        loop {
            match input {
                [b'1'..=b'9', ..] | [b'-', b'1'..=b'9', ..] => {
                    let (num, remaining) = parser::i32().parse(input)?;
                    input = remaining;
                    part1 += num;
                    part2 += num;
                }
                [b'"', b'r', b'e', b'd', b'"', ..] => {
                    input = &input[5..];
                    red = true;
                }
                [b'{', ..] => {
                    let (result, remaining) = Self::parse_object(&input[1..])?;
                    input = remaining;
                    part1 += result.0;
                    part2 += result.1;
                }
                [b'[', ..] => {
                    let (result, remaining) = Self::parse_array(&input[1..])?;
                    input = remaining;
                    part1 += result.0;
                    part2 += result.1;
                }
                [b'}', ..] => return Ok(((part1, if red { 0 } else { part2 }), &input[1..])),
                [] => return Err((ParseError::ExpectedByte(b']'), input)),
                _ => input = &input[1..],
            }
        }
    }

    fn parse_array(mut input: &[u8]) -> ParseResult<(i32, i32)> {
        let (mut part1, mut part2) = (0, 0);
        loop {
            match input {
                [b'1'..=b'9', ..] | [b'-', b'1'..=b'9', ..] => {
                    let (num, remaining) = parser::i32().parse(input)?;
                    input = remaining;
                    part1 += num;
                    part2 += num;
                }
                [b'{', ..] => {
                    let (result, remaining) = Self::parse_object(&input[1..])?;
                    input = remaining;
                    part1 += result.0;
                    part2 += result.1;
                }
                [b'[', ..] => {
                    let (result, remaining) = Self::parse_array(&input[1..])?;
                    input = remaining;
                    part1 += result.0;
                    part2 += result.1;
                }
                [b']', ..] => return Ok(((part1, part2), &input[1..])),
                [] => return Err((ParseError::ExpectedByte(b']'), input)),
                _ => input = &input[1..],
            }
        }
    }
}

examples!(Day12 -> (i32, i32) [
    {input: r#"[1,2,3]"#, part1: 6, part2: 6},
    {input: r#"{"a":2,"b":4}"#, part1: 6},
    {input: r#"[[[3]]]"#, part1: 3},
    {input: r#"{"a":{"b":4},"c":-1}"#, part1: 3},
    {input: r#"{"a":[-1,1]}"#, part1: 0},
    {input: r#"[-1,{"a":1}]"#, part1: 0},
    {input: r#"[]"#, part1: 0},
    {input: r#"{}"#, part1: 0},
    {input: r#"[1,{"c":"red","b":2},3]"#, part2: 4},
    {input: r#"{"d":"red","e":[1,2,3,4],"f":5}"#, part2: 0},
    {input: r#"[1,"red",5]"#, part2: 6},
]);
