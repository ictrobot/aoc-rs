use utils::point::Point2D;
use utils::prelude::*;

/// Measuring regions using Manhattan distance.
#[derive(Clone, Debug)]
pub struct Day06 {
    locations: Vec<Point2D<usize>>,
    region_threshold: u32,
}

const WIDTH: usize = 320;

impl Day06 {
    pub fn new(input: &str, input_type: InputType) -> Result<Self, InputError> {
        let locations = parser::number_range(0..=999)
            .repeat_n(", ")
            .map(Point2D::from)
            .parse_lines(input)?;
        if locations.is_empty() {
            return Err(InputError::new(input, 0, "expected at least one location"));
        }
        if locations.len() >= u8::MAX as usize {
            return Err(InputError::new(input, 0, "too many locations"));
        }

        let min = locations.iter().map(|p| p.x.min(p.y)).min().unwrap();
        let max = locations.iter().map(|p| p.x.max(p.y)).max().unwrap();
        if max - min >= WIDTH {
            return Err(InputError::new(input, 0, "coordinate range too large"));
        }

        let min_point = Point2D::new(min, min);
        Ok(Self {
            locations: locations.into_iter().map(|p| p - min_point).collect(),
            region_threshold: match input_type {
                InputType::Example => 32,
                InputType::Real => 10000,
            },
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let mut distances = [0u16; WIDTH * WIDTH];
        let mut closest = [0u8; WIDTH * WIDTH];

        for (i, location) in self.locations.iter().enumerate() {
            // Distances are stored inverted so the array can start zeroed
            distances[location.y * WIDTH + location.x] = u16::MAX;
            closest[location.y * WIDTH + location.x] = 1 + i as u8;
        }

        // Right
        for y in 0..WIDTH {
            Self::chunk(
                &mut distances,
                &mut closest,
                (y * WIDTH)..(y * WIDTH) + (WIDTH - 1),
                1,
            );
        }
        // Down
        Self::chunk(&mut distances, &mut closest, 0..WIDTH * (WIDTH - 1), WIDTH);

        // Left
        for y in (0..WIDTH).rev() {
            Self::chunk(
                &mut distances,
                &mut closest,
                ((y * WIDTH) + 1..(y * WIDTH) + WIDTH).rev(),
                1usize.wrapping_neg(),
            );
        }
        // Up
        Self::chunk(
            &mut distances,
            &mut closest,
            (WIDTH..WIDTH * WIDTH).rev(),
            WIDTH.wrapping_neg(),
        );

        let mut finite = vec![true; self.locations.len() + 1];
        finite[0] = false;
        for i in 0..WIDTH {
            finite[closest[i] as usize] = false;
            finite[closest[WIDTH * (WIDTH - 1) + i] as usize] = false;
            finite[closest[WIDTH * i] as usize] = false;
            finite[closest[WIDTH * i + WIDTH - 1] as usize] = false;
        }

        let mut counts = vec![0; self.locations.len() + 1];
        for &c in closest.iter() {
            counts[c as usize] += 1;
        }

        counts
            .iter()
            .zip(finite.iter())
            .filter(|&(_, &i)| i)
            .map(|(&c, _)| c)
            .max()
            .unwrap()
    }

    #[inline]
    fn chunk(
        distances: &mut [u16],
        closest: &mut [u8],
        coords: impl Iterator<Item = usize>,
        offset: usize,
    ) {
        for from in coords {
            let to = from.wrapping_add(offset);
            let dist = distances[from].saturating_sub(1);
            if dist > distances[to] {
                distances[to] = dist;
                closest[to] = closest[from];
            } else if dist == distances[to] && closest[to] != closest[from] {
                closest[to] = 0;
            }
        }
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut x_distances = [0u32; WIDTH];
        let mut y_distances = [0u32; WIDTH];
        for location in &self.locations {
            for i in 0..WIDTH {
                x_distances[i] += location.x.abs_diff(i) as u32;
                y_distances[i] += location.y.abs_diff(i) as u32;
            }
        }

        let mut region = 0;
        for y_dist in y_distances {
            region += x_distances
                .iter()
                .map(|&x| x + y_dist)
                .filter(|&x| x < self.region_threshold)
                .count() as u32;
        }
        region
    }
}

examples!(Day06 -> (u32, u32) [
    {input: "1, 1\n1, 6\n8, 3\n3, 4\n5, 5\n8, 9", part1: 17, part2: 16},
]);
