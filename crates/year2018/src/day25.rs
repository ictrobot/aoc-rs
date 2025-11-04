use utils::geometry::Vec4;
use utils::prelude::*;

/// Clustering points within Manhattan distance.
#[derive(Clone, Debug)]
pub struct Day25 {
    points: Vec<Vec4<i32>>,
}

impl Day25 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut points = parser::i32()
            .repeat_n(b',')
            .map(Vec4::from)
            .parse_lines(input)?;

        points.sort_unstable_by_key(|p| p.x);

        Ok(Self { points })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let mut remaining = self.points.clone();
        let mut to_visit = Vec::with_capacity(remaining.len());
        let mut constellations = 0;
        while let Some(start) = remaining.pop() {
            to_visit.push(start);
            while let Some(point) = to_visit.pop() {
                // Use binary search to only check points in range on the x-axis.
                // This requires keeping the remaining points sorted (instead of using swap_remove)
                let start = remaining.partition_point(|p| p.x < point.x - 3);
                let end = remaining.partition_point(|p| p.x <= point.x + 3);

                to_visit.extend(
                    remaining.extract_if(start..end, |p| p.manhattan_distance_to(point) <= 3),
                );
            }
            constellations += 1;
        }
        constellations
    }

    #[must_use]
    pub fn part2(&self) -> &'static str {
        "🎄"
    }
}

examples!(Day25 -> (u32, &'static str) [
    {file: "day25_example0.txt", part1: 4},
    {file: "day25_example1.txt", part1: 3},
    {file: "day25_example2.txt", part1: 8},
]);
