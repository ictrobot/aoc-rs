use utils::parser::ParseError;
use utils::prelude::*;

/// Summing nested node metadata.
#[derive(Clone, Debug)]
pub struct Day08 {
    part1: u32,
    part2: u32,
}

impl Day08 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut iter = parser::u32()
            .with_consumed()
            .with_suffix(b' '.or(parser::eof()))
            .parse_iterator(input);

        let (part1, part2) = Self::parse_node(&mut |name, min_bound, max_bound| {
            let min_bound = min_bound.unwrap_or(0);
            let max_bound = max_bound.unwrap_or(u32::MAX);
            match iter.next() {
                None => Err(InputError::new(
                    input,
                    input.len(),
                    ParseError::Expected(name),
                )),
                Some(Ok((n, pos))) if n < min_bound => Err(InputError::new(
                    input,
                    pos,
                    ParseError::NumberTooSmall(min_bound.into()),
                )),
                Some(Ok((n, pos))) if n > max_bound => Err(InputError::new(
                    input,
                    pos,
                    ParseError::NumberTooLarge(max_bound.into()),
                )),
                Some(Ok((n, _))) => Ok(n),
                Some(Err(e)) => Err(e),
            }
        })?;

        let remaining = iter.remaining();
        if !remaining.is_empty() {
            return Err(InputError::new(input, remaining, ParseError::ExpectedEof()));
        }

        Ok(Self { part1, part2 })
    }

    fn parse_node(
        next: &mut impl FnMut(&'static str, Option<u32>, Option<u32>) -> Result<u32, InputError>,
    ) -> Result<(u32, u32), InputError> {
        let children = next("child count", None, Some(16))?;
        let metadata = next("metadata count", Some(1), None)?;

        let mut part1 = 0;
        let mut child_values = [0; 16];
        for i in 0..children {
            let (p1, p2) = Self::parse_node(next)?;
            part1 += p1;
            child_values[i as usize] = p2;
        }

        let mut part2 = 0;
        for _ in 0..metadata {
            let e = next("metadata entry", None, None)?;
            part1 += e;
            part2 += child_values
                .get(e.wrapping_sub(1) as usize)
                .copied()
                .unwrap_or(0);
        }

        if children == 0 {
            part2 = part1;
        }

        Ok((part1, part2))
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

examples!(Day08 -> (u32, u32) [
    {input: "2 3 0 3 10 11 12 1 1 0 1 99 2 1 1 2", part1: 138, part2: 66},
]);
