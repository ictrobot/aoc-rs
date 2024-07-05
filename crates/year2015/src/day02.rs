use utils::prelude::*;

/// Box maths.
#[derive(Clone, Debug)]
pub struct Day02 {
    boxes: Vec<Box>,
}

#[derive(Clone, Debug)]
struct Box {
    l: u32,
    w: u32,
    h: u32,
}

impl Day02 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InvalidInputError> {
        Ok(Self {
            boxes: input
                .lines()
                .map(|line| {
                    let (l, rest) = line
                        .split_once('x')
                        .ok_or_else(|| InvalidInputError::UnexpectedString(line.to_string()))?;
                    let (w, h) = rest
                        .split_once('x')
                        .ok_or_else(|| InvalidInputError::UnexpectedString(line.to_string()))?;
                    Ok(Box {
                        l: l.parse()?,
                        w: w.parse()?,
                        h: h.parse()?,
                    })
                })
                .collect::<Result<Vec<Box>, InvalidInputError>>()?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.boxes
            .iter()
            .map(|&Box { l, w, h }| {
                (2 * l * w) + (2 * w * h) + (2 * h * l) + (l * w).min(w * h).min(h * l)
            })
            .sum()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.boxes
            .iter()
            .map(|&Box { l, w, h }| {
                let min = l.min(w).min(h);
                let max = l.max(w).max(h);
                let mid = l + w + h - min - max;
                (2 * min) + (2 * mid) + (l * w * h)
            })
            .sum()
    }
}

examples!(Day02 -> (u32, u32) [
    {input: "2x3x4", part1: 58, part2: 34},
    {input: "1x1x10", part1: 43, part2: 14},
]);
