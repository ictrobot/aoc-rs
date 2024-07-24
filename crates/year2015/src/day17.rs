use utils::prelude::*;

/// Counting subset sums.
#[derive(Clone, Debug)]
pub struct Day17 {
    part1: u16,
    part2: u16,
}

impl Day17 {
    pub fn new(input: &str, input_type: InputType) -> Result<Self, InputError> {
        let parsed = parser::u8().parse_lines(input)?;
        let (part1, part2) = match input_type {
            InputType::Example => Self::calculate::<25>(&parsed),
            InputType::Real => Self::calculate::<150>(&parsed),
        };
        Ok(Self { part1, part2 })
    }

    fn calculate<const N: usize>(sizes: &[u8]) -> (u16, u16) {
        // matrix[number of containers - 1][total capacity - 1] = combinations
        let mut matrix = vec![[0; N]; sizes.len()];

        for (i, size) in sizes.iter().copied().enumerate() {
            // Reverse order is required to avoid double counting container
            for containers in (1..=i).rev() {
                for total in 0..N - size as usize {
                    matrix[containers][total + size as usize] += matrix[containers - 1][total];
                }
            }
            matrix[0][size as usize - 1] += 1;
        }

        (
            matrix.iter().map(|x| x[N - 1]).sum(),
            matrix
                .iter()
                .map(|x| x[N - 1])
                .find(|&x| x > 0)
                .unwrap_or(0),
        )
    }

    #[must_use]
    pub fn part1(&self) -> u16 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u16 {
        self.part2
    }
}

examples!(Day17 -> (u16, u16) [
    {input: "20\n15\n10\n5\n5", part1: 4, part2: 3},
]);
