use utils::prelude::*;

/// Minimizing subset size & product.
#[derive(Clone, Debug)]
pub struct Day24 {
    presents: Vec<u32>,
    remaining_totals: Vec<u32>,
    sum: u32,
}

impl Day24 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut presents = parser::u32().parse_lines(input)?;
        presents.sort_unstable_by(|a, b| b.cmp(a));

        // Pre-calculate the total of the remaining presents for each position, to avoid repeatedly
        // summing the remainder of the array in min_quantum_entanglement
        let mut remaining_totals = presents
            .iter()
            .rev()
            .scan(0, |acc, x| {
                *acc += x;
                Some(*acc)
            })
            .collect::<Vec<_>>();
        remaining_totals.reverse();

        let sum: u32 = remaining_totals.first().copied().unwrap_or(0);
        if !sum.is_multiple_of(12) {
            return Err(InputError::new(input, 0, "Total must be multiple of 12"));
        }

        Ok(Self {
            presents,
            remaining_totals,
            sum,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        let mut min = (u32::MAX, u64::MAX);
        self.min_quantum_entanglement(self.sum / 3, 0, 0, 1, &mut min);
        min.1
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        let mut min = (u32::MAX, u64::MAX);
        self.min_quantum_entanglement(self.sum / 4, 0, 0, 1, &mut min);
        min.1
    }

    fn min_quantum_entanglement(
        &self,
        remaining: u32,
        start_index: usize,
        count: u32,
        product: u64,
        min_found: &mut (u32, u64),
    ) {
        if remaining == 0 {
            *min_found = (*min_found).min((count, product));
            return;
        }

        // Stop searching early if the group would contain more packages than the current minimum
        if count >= min_found.0 {
            return;
        }

        // Stop searching early if the group can't be competed with the remaining presents
        // Equivalent to `self.presents[start_index..].iter().sum::<u32>() < remaining`
        if self.remaining_totals.get(start_index).copied().unwrap_or(0) < remaining {
            return;
        }

        for (i, &size) in self.presents[start_index..].iter().enumerate() {
            if size <= remaining {
                self.min_quantum_entanglement(
                    remaining - size,
                    start_index + i + 1,
                    count + 1,
                    product * u64::from(size),
                    min_found,
                )
            }
        }
    }
}

examples!(Day24 -> (u64, u64) [
    {input: "1\n2\n3\n4\n5\n7\n8\n9\n10\n11", part1: 99, part2: 44},
]);
