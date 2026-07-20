use utils::geometry::Vec2;
use utils::grid;
use utils::prelude::*;

/// Counting obstacles on slopes through a repeating grid.
#[derive(Clone, Debug)]
pub struct Day03 {
    part1: u64,
    part2: u64,
}

const SLOPES: [Vec2<usize>; 5] = [
    Vec2::new(1, 1),
    Vec2::new(3, 1),
    Vec2::new(5, 1),
    Vec2::new(7, 1),
    Vec2::new(1, 2),
];

impl Day03 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut counts = [0; SLOPES.len()];
        grid::for_each_row(
            input,
            |b| matches!(b, b'.' | b'#'),
            || "expected '.' or '#'",
            |row, cols, trees| {
                for (delta, count) in SLOPES.into_iter().zip(counts.iter_mut()) {
                    if row.is_multiple_of(delta.y) {
                        let col = ((row / delta.y) * delta.x) % cols;
                        *count += u64::from(trees[col] == b'#');
                    }
                }
                Ok(())
            },
        )?;

        Ok(Self {
            part1: counts[1],
            part2: counts.iter().product(),
        })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        self.part2
    }
}

examples!(Day03 -> (u64, u64) [
    {file: "day03_example0.txt", part1: 7, part2: 336},
]);
