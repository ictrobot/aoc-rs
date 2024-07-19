use std::collections::HashMap;
use utils::prelude::*;

/// Logic gates.
///
/// Use indexes instead of wire names for performance.
/// To avoid adding a special type for the gates with constant inputs, pack u16 constants into the
/// highest usize values. This should be fine on any platform with at least 32-bit pointers.
#[derive(Clone, Debug)]
pub struct Day07 {
    wires: Vec<Signal>,
    a_idx: usize,
    b_idx: usize,
}

#[derive(Copy, Clone, Debug)]
enum Signal {
    Wire(usize),
    And(usize, usize),
    Or(usize, usize),
    Not(usize),
    LShift(usize, u8),
    RShift(usize, u8),
}

impl Day07 {
    const U16_CONST_MASK: usize = usize::MAX & !(u16::MAX as usize);

    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut indexes = HashMap::new();
        for (i, l) in input.lines().enumerate() {
            let Some((_, name)) = l.rsplit_once(" -> ") else {
                return Err(InputError::new(input, l, "line missing \" -> \""));
            };
            indexes.insert(name.as_bytes(), i);
        }
        if indexes.len() >= Self::U16_CONST_MASK {
            return Err(InputError::new(input, 0, "too many wires"));
        }

        let parse_wire = parser::take_while(u8::is_ascii_lowercase)
            .map_res(|v| indexes.get(v).copied().ok_or("wire not found"))
            .or(parser::u16().map(|v| Self::U16_CONST_MASK | v as usize));

        let wires = parse_wire
            .with_prefix("NOT ")
            .map(Signal::Not)
            .or(parse_wire
                .with_suffix(" AND ")
                .then(parse_wire)
                .map(|(a, b)| Signal::And(a, b)))
            .or(parse_wire
                .with_suffix(" OR ")
                .then(parse_wire)
                .map(|(a, b)| Signal::Or(a, b)))
            .or(parse_wire
                .with_suffix(" RSHIFT ")
                .then(parser::u8())
                .map(|(a, b)| Signal::RShift(a, b)))
            .or(parse_wire
                .with_suffix(" LSHIFT ")
                .then(parser::u8())
                .map(|(a, b)| Signal::LShift(a, b)))
            .or(parse_wire.map(Signal::Wire))
            .with_suffix(" -> ")
            .with_suffix(parser::take_while(u8::is_ascii_lowercase))
            .parse_lines(input)?;

        let Some(&a_idx) = indexes.get(&b"a"[..]) else {
            return Err(InputError::new(input, 0, "missing 'a' wire"));
        };
        let Some(&b_idx) = indexes.get(&b"b"[..]) else {
            return Err(InputError::new(input, 0, "missing 'b' wire"));
        };

        Ok(Self {
            wires,
            a_idx,
            b_idx,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u16 {
        self.calculate(self.a_idx, &mut vec![None; self.wires.len()])
    }

    #[must_use]
    pub fn part2(&self) -> u16 {
        let mut cache = vec![None; self.wires.len()];
        cache[self.b_idx] = Some(self.part1());
        self.calculate(self.a_idx, &mut cache)
    }

    fn calculate(&self, idx: usize, cache: &mut [Option<u16>]) -> u16 {
        if idx & Self::U16_CONST_MASK == Self::U16_CONST_MASK {
            idx as u16
        } else if let Some(v) = cache[idx] {
            v
        } else {
            let v = match self.wires[idx] {
                Signal::Wire(l) => self.calculate(l, cache),
                Signal::And(l, r) => self.calculate(l, cache) & self.calculate(r, cache),
                Signal::Or(l, r) => self.calculate(l, cache) | self.calculate(r, cache),
                Signal::Not(l) => !self.calculate(l, cache),
                Signal::LShift(l, by) => self.calculate(l, cache) << by,
                Signal::RShift(l, by) => self.calculate(l, cache) >> by,
            };
            cache[idx] = Some(v);
            v
        }
    }
}

examples!(Day07 -> (u16, u16) []);
