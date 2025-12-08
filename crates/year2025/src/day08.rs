use std::collections::BinaryHeap;
use utils::disjoint_set::Dsu;
use utils::geometry::Vec3;
use utils::prelude::*;

/// Connecting the nearest 3D points to form a connected graph.
#[derive(Clone, Debug)]
pub struct Day08 {
    boxes: Vec<Vec3<u32>>,
    part1_limit: usize,
}

impl Day08 {
    pub fn new(input: &str, input_type: InputType) -> Result<Self, InputError> {
        let boxes = parser::u32()
            .repeat_n(b',')
            .map(Vec3::from)
            .parse_lines(input)?;

        if boxes.len() > u32::MAX as usize {
            return Err(InputError::new(input, 0, "too many boxes"));
        }

        Ok(Self {
            boxes,
            part1_limit: match input_type {
                InputType::Example => 10,
                InputType::Real => 1000,
            },
        })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        let mut heap = BinaryHeap::with_capacity(self.part1_limit);
        for i in 0..self.boxes.len() {
            for j in (i + 1)..self.boxes.len() {
                let dist_squared = self.boxes[i]
                    .cast::<i64>()
                    .squared_euclidean_distance_to(self.boxes[j].cast());
                if heap.len() < self.part1_limit {
                    heap.push((dist_squared, i, j));
                } else if dist_squared < heap.peek().unwrap().0 {
                    heap.pop();
                    heap.push((dist_squared, i, j));
                }
            }
        }
        let edges = heap.into_sorted_vec();

        let mut dsu = Dsu::new(self.boxes.len());
        for &(_, i, j) in edges.iter() {
            let _ = dsu.union(i, j);
        }

        let mut component_sizes: Vec<_> = dsu.roots().map(|x| dsu.root_size(x) as u64).collect();
        component_sizes.select_nth_unstable_by(2, |a, b| b.cmp(a));
        component_sizes.iter().take(3).product()
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        let mut edges = Vec::with_capacity(self.boxes.len() * (self.boxes.len() - 1) / 2);
        for i in 0..self.boxes.len() {
            for j in (i + 1)..self.boxes.len() {
                edges.push((
                    self.boxes[i]
                        .cast::<i64>()
                        .squared_euclidean_distance_to(self.boxes[j].cast()),
                    // Store indices as u32s to reduce memory usage and speed up sorting
                    i as u32,
                    j as u32,
                ));
            }
        }

        let mut dsu = Dsu::new(self.boxes.len());
        let mut remaining_merges = self.boxes.len() - 1;

        // Process edges in chunks to avoid sorting the entire vec
        let mut edges = edges.as_mut_slice();
        while !edges.is_empty() {
            let chunk_size = edges.len().min(8192);

            edges.select_nth_unstable_by(chunk_size - 1, |(a, _, _), (b, _, _)| a.cmp(b));
            edges[..chunk_size].sort_unstable_by(|(a, _, _), (b, _, _)| a.cmp(b));

            for &(_, i, j) in edges[..chunk_size].iter() {
                if dsu.union(i as usize, j as usize) {
                    remaining_merges -= 1;
                    if remaining_merges == 0 {
                        return self.boxes[i as usize].x as u64 * self.boxes[j as usize].x as u64;
                    }
                }
            }

            edges = &mut edges[chunk_size..];
        }

        panic!("no solution found");
    }
}

examples!(Day08 -> (u64, u64) [
    {file: "day08_example0.txt", part1: 40, part2: 25272},
]);
