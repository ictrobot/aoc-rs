use std::ops::ControlFlow;
use utils::disjoint_set::Dsu;
use utils::geometry::Vec3;
use utils::prelude::*;

/// Connecting the nearest 3D points to form a connected graph.
#[derive(Clone, Debug)]
pub struct Day08 {
    points: Vec<Vec3<u32>>,
    subdivisions: Vec<Vec<u32>>,
    part1_limit: usize,
}

const MAX_COORD: u32 = 99999;
const SUBDIVISIONS: usize = 6;
const TOTAL_SUBDIVISIONS: usize = SUBDIVISIONS * SUBDIVISIONS * SUBDIVISIONS;
const SUBDIVISION_WIDTH: usize = (MAX_COORD as usize + 1).div_ceil(SUBDIVISIONS);
const SUBDIVISION_WIDTH2: usize = SUBDIVISION_WIDTH * SUBDIVISION_WIDTH;

// [(min euclidian distance between subdivisions, slice of subdivision offsets)]
const DISTANCE_TIER_OFFSETS: &[(usize, &[Vec3<i8>])] = {
    const fn compute_offsets<const N: usize>(tier: u32) -> [Vec3<i8>; N] {
        let max_abs = (tier.isqrt() + 1) as i8;

        let mut result = [Vec3::ORIGIN; N];
        let mut i = 0usize;

        // Only generate offsets where the second subdivision has a greater index, so that each
        // subdivision pair is only checked once
        let mut dz = 0;
        while dz <= max_abs {
            let mut dy = if dz == 0 { 0 } else { -max_abs };
            while dy <= max_abs {
                let mut dx = if dz == 0 && dy == 0 { 1 } else { -max_abs };
                while dx <= max_abs {
                    let min_dist_x = dx.unsigned_abs().saturating_sub(1) as u32;
                    let min_dist_y = dy.unsigned_abs().saturating_sub(1) as u32;
                    let min_dist_z = dz.unsigned_abs().saturating_sub(1) as u32;
                    let min_dist2 =
                        min_dist_x * min_dist_x + min_dist_y * min_dist_y + min_dist_z * min_dist_z;

                    if tier == min_dist2 {
                        result[i] = Vec3::new(dx, dy, dz);
                        i += 1;
                    }
                    dx += 1;
                }
                dy += 1;
            }
            dz += 1;
        }

        assert!(i == N);
        result
    }

    &[
        // 3x |d|<=1
        (0, &compute_offsets::<13>(0)),
        // 1x |d|=2, 2x |d|<=1
        (SUBDIVISION_WIDTH2, &compute_offsets::<27>(1)),
        // 2x |d|=2, 1x |d|<=1
        (2 * SUBDIVISION_WIDTH2, &compute_offsets::<18>(2)),
        // 3x |d|=2
        (3 * SUBDIVISION_WIDTH2, &compute_offsets::<4>(3)),
        // 1x |d|=3, 2x |d|<=1
        (4 * SUBDIVISION_WIDTH2, &compute_offsets::<27>(4)),
        // 1x |d|=3, 1x |d|=2, 1x |d|<=1
        (5 * SUBDIVISION_WIDTH2, &compute_offsets::<36>(5)),
        // 1x |d|=3, 2x |d|=2
        (6 * SUBDIVISION_WIDTH2, &compute_offsets::<12>(6)),
        // Impossible
        (7 * SUBDIVISION_WIDTH2, &compute_offsets::<0>(7)),
        // 2x |d|=3, 1x |d|<=1
        (8 * SUBDIVISION_WIDTH2, &compute_offsets::<18>(8)),
        // 2x |d|=3, 1x |d|=2 OR 1x |d|=4, 2x |d|<=1
        (9 * SUBDIVISION_WIDTH2, &compute_offsets::<39>(9)),
        // 1x |d|=4, 1x |d|=2, 1x |d|<=1
        (10 * SUBDIVISION_WIDTH2, &compute_offsets::<36>(10)),
        // 1x |d|=4, 2x |d|=2
        (11 * SUBDIVISION_WIDTH2, &compute_offsets::<12>(11)),
        // 3x |d|=3
        (12 * SUBDIVISION_WIDTH2, &compute_offsets::<4>(12)),
    ]
};

// DISTANCE_TIER_OFFSETS is truncated and doesn't include every possible distance/subdivision offset
const MAX_DISTANCE_CHECKED: usize = (DISTANCE_TIER_OFFSETS.len() * SUBDIVISION_WIDTH2).isqrt();

impl Day08 {
    pub fn new(input: &str, input_type: InputType) -> Result<Self, InputError> {
        let points = parser::number_range(0..=MAX_COORD)
            .repeat_n(b',')
            .map(Vec3::from)
            .parse_lines(input)?;

        if points.len() > u32::MAX as usize {
            return Err(InputError::new(input, 0, "too many points"));
        }

        let mut subdivisions = vec![Vec::new(); TOTAL_SUBDIVISIONS];
        for (i, p) in points.iter().enumerate() {
            let sub_coords = p.map(|x| x as usize / SUBDIVISION_WIDTH);
            subdivisions[Self::subdivision_index(sub_coords)].push(i as u32);
        }

        Ok(Self {
            points,
            subdivisions,
            part1_limit: match input_type {
                InputType::Example => 10,
                InputType::Real => 1000,
            },
        })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        let mut dsu = Dsu::new(self.points.len());
        let mut remaining_edges = self.part1_limit;
        self.for_each_sorted_edge(
            |_, i, j| {
                let _ = dsu.union(i, j);
                remaining_edges -= 1;
                if remaining_edges == 0 {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            },
            self.part1_limit,
        );

        let mut component_sizes: Vec<_> = dsu.roots().map(|x| dsu.root_size(x) as u64).collect();
        component_sizes.select_nth_unstable_by(2, |a, b| b.cmp(a));
        component_sizes.iter().take(3).product()
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        let mut dsu = Dsu::new(self.points.len());
        let mut remaining_merges = self.points.len() - 1;

        self.for_each_sorted_edge(
            |_, i, j| {
                if dsu.union(i, j) {
                    remaining_merges -= 1;
                    if remaining_merges == 0 {
                        return ControlFlow::Break(
                            self.points[i].x as u64 * self.points[j].x as u64,
                        );
                    }
                }
                ControlFlow::Continue(())
            },
            usize::MAX,
        )
    }

    #[inline]
    fn for_each_sorted_edge<T>(
        &self,
        mut f: impl FnMut(i64, usize, usize) -> ControlFlow<T>,
        mut limit: usize,
    ) -> T {
        let mut edges = Vec::new();
        let mut next_edges = Vec::new();

        for &(min_dist2, offsets) in DISTANCE_TIER_OFFSETS.iter() {
            let max_dist2 = min_dist2 + SUBDIVISION_WIDTH2;

            // Move edges now below max_dist2 from next_edges to edges
            next_edges.retain(|&(dist2, i, j)| {
                if dist2 < max_dist2 as i64 {
                    edges.push((dist2, i, j));
                    false
                } else {
                    true
                }
            });

            // Push the pair to edges if its distance is within the current tier
            let mut process_pair = |i: u32, j: u32| {
                let dist2 = self.points[i as usize]
                    .cast::<i64>()
                    .squared_euclidean_distance_to(self.points[j as usize].cast());

                if dist2 < max_dist2 as i64 {
                    edges.push((dist2, i, j));
                } else if edges.len() < limit {
                    // Only store pairs for the next tier if the current tier hasn't already reached
                    // the provided limit
                    next_edges.push((dist2, i, j));
                }
            };

            // Check all point pairs within subdivision pairs at the current tier
            for sub1 in 0..self.subdivisions.len() {
                if min_dist2 == 0 {
                    // Try pairs within the current subdivision
                    for (n, &i) in self.subdivisions[sub1].iter().enumerate() {
                        for &j in self.subdivisions[sub1].iter().skip(n + 1) {
                            process_pair(i, j);
                        }
                    }
                }

                let sub1_coords = Self::subdivision_coords(sub1);
                for offset in offsets.iter() {
                    let sub2_coords = sub1_coords.wrapping_add_signed(offset.cast());
                    if sub2_coords.x >= SUBDIVISIONS
                        || sub2_coords.y >= SUBDIVISIONS
                        || sub2_coords.z >= SUBDIVISIONS
                    {
                        continue;
                    }

                    let sub2 = Self::subdivision_index(sub2_coords);

                    for &i in self.subdivisions[sub1].iter() {
                        for &j in self.subdivisions[sub2].iter() {
                            process_pair(i, j);
                        }
                    }
                }
            }

            if edges.len() > limit {
                edges.select_nth_unstable_by(limit - 1, |(a, _, _), (b, _, _)| a.cmp(b));
                edges.truncate(limit);
            }

            edges.sort_unstable_by(|(a, _, _), (b, _, _)| a.cmp(b));
            limit -= edges.len();

            for &(d, i, j) in edges.iter() {
                if let ControlFlow::Break(result) = f(d, i as usize, j as usize) {
                    return result;
                }
            }

            assert!(
                limit > 0,
                "f should return break at or before the provided limit"
            );
            edges.clear();
        }

        panic!("no solution found after checking all pairs within {MAX_DISTANCE_CHECKED} distance");
    }

    #[inline]
    fn subdivision_index(c: Vec3<usize>) -> usize {
        c.x + c.y * SUBDIVISIONS + c.z * SUBDIVISIONS * SUBDIVISIONS
    }

    #[inline]
    fn subdivision_coords(i: usize) -> Vec3<usize> {
        let x = i % SUBDIVISIONS;
        let y = (i / SUBDIVISIONS) % SUBDIVISIONS;
        let z = i / SUBDIVISIONS / SUBDIVISIONS;
        Vec3::new(x, y, z)
    }
}

examples!(Day08 -> (u64, u64) [
    {file: "day08_example0.txt", part1: 40, part2: 25272},
]);
