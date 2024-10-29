use utils::parser::{ParseError, ParseResult};
use utils::prelude::*;

/// Parsing a nested structure.
#[derive(Clone, Debug)]
pub struct Day09 {
    part1: u32,
    part2: u32,
}

impl Day09 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (part1, part2) = Self::parse.parse_complete(input)?;
        Ok(Self { part1, part2 })
    }

    // Parses either a (nested) group or a single piece of garbage.
    fn parse(mut input: &[u8]) -> ParseResult<(u32, u32)> {
        let mut group_depth = 0;
        let mut in_garbage = false;

        let mut group_score = 0;
        let mut garbage_count = 0;

        loop {
            input = if in_garbage {
                match input {
                    [b'!', _, rest @ ..] => rest,
                    [b'>', rest @ ..] => {
                        in_garbage = false;
                        if group_depth == 0 {
                            return Ok(((group_score, garbage_count), rest));
                        }
                        rest
                    }
                    [_, rest @ ..] => {
                        garbage_count += 1;
                        rest
                    }
                    [] => return Err((ParseError::ExpectedByte(b'>'), input)),
                }
            } else {
                match input {
                    [b'{', rest @ ..] => {
                        group_depth += 1;
                        group_score += group_depth;
                        rest
                    }
                    [b'}', rest @ ..] if group_depth > 0 => {
                        group_depth -= 1;
                        if group_depth == 0 {
                            return Ok(((group_score, garbage_count), rest));
                        }
                        rest
                    }
                    [b'<', rest @ ..] => {
                        in_garbage = true;
                        rest
                    }
                    [b',', rest @ ..] if group_depth > 0 => rest,
                    _ if group_depth > 0 => {
                        return Err((ParseError::Custom("expected '{', '}', ',' or '<'"), input))
                    }
                    _ => return Err((ParseError::Custom("expected '{' or '<'"), input)),
                }
            }
        }
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.part2
    }
}

examples!(Day09 -> (u32, u32) [
    {input: "{}", part1: 1},
    {input: "{{{}}}", part1: 6},
    {input: "{{},{}}", part1: 5},
    {input: "{{{},{},{{}}}}", part1: 16},
    {input: "{<a>,<a>,<a>,<a>}", part1: 1},
    {input: "{{<ab>},{<ab>},{<ab>},{<ab>}}", part1: 9},
    {input: "{{<!!>},{<!!>},{<!!>},{<!!>}}", part1: 9},
    {input: "{{<a!>},{<a!>},{<a!>},{<ab>}}", part1: 3},
    {input: "<>", part2: 0},
    {input: "<random characters>", part2: 17},
    {input: "<<<<>", part2: 3},
    {input: "<{!>}>", part2: 2},
    {input: "<!!>", part2: 0},
    {input: "<!!!>>", part2: 0},
    {input: "<{o\"i!a,<{i<a>", part2: 10},
]);
