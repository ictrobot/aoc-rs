use std::cmp::Reverse;
use std::collections::BinaryHeap;
use utils::array::ArrayVec;
use utils::bit::BitIterator;
use utils::grid;
use utils::hash::FastMap;
use utils::prelude::*;

/// Finding the shortest path to collect every key.
///
/// The key optimization is to precompute the distance between all entrances and keys, along with
/// the keys required to travel each path. This allows the final search to only track each robot's
/// position and which keys remain.
#[derive(Clone, Debug)]
pub struct Day18 {
    grid: Vec<u8>,
    cols: usize,
    starts: Vec<usize>,
    key_positions: [Option<usize>; MAX_KEYS],
    all_keys: u32,
}

#[derive(Clone, Copy, Debug, Default)]
struct Edge {
    distance: u32,
    needed_mask: u32,
    target_node_bit: u32,
    target_key_bit: u32,
}

const MAX_KEYS: usize = 26;
const MAX_ROBOTS: usize = 4;
const MAX_NODES: usize = MAX_KEYS + MAX_ROBOTS;

impl Day18 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut starts = Vec::new();
        let mut key_positions = [None; MAX_KEYS];
        let mut all_keys = 0;
        let mut all_doors = 0;
        let (_, cols, grid) = grid::parse(
            input,
            1,
            b'#',
            |b| b,
            |b| matches!(b, b'#' | b'.'),
            |i, b| match b {
                b'@' => {
                    starts.push(i);
                    Ok(b'@')
                }
                b'a'..=b'z' if key_positions[(b - b'a') as usize].is_some() => Err("duplicate key"),
                b'a'..=b'z' => {
                    key_positions[(b - b'a') as usize] = Some(i);
                    all_keys |= 1 << (b - b'a');
                    Ok(b)
                }
                b'A'..=b'Z' => {
                    all_doors |= 1 << (b - b'A');
                    Ok(b)
                }
                _ => Err("expected '#', '.', '@' or letter"),
            },
        )?;
        if grid.len() > usize::from(u16::MAX) {
            return Err(InputError::new(input, 0, "grid too large"));
        }
        if starts.len() != 1 && starts.len() != 4 {
            return Err(InputError::new(input, 0, "expected 1 or 4 entrances"));
        }
        if all_doors & !all_keys != 0 {
            return Err(InputError::new(input, 0, "door without matching key"));
        }

        Ok(Self {
            grid,
            cols,
            starts,
            key_positions,
            all_keys,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        assert_eq!(self.starts.len(), 1, "part 1 requires a single entrance");
        self.minimum_steps(&self.grid, &self.starts)
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        if self.starts.len() == 4 {
            return self.minimum_steps(&self.grid, &self.starts);
        }

        let mut grid = self.grid.clone();
        let middle = self.starts[0];
        let cols = self.cols as isize;
        for dy in [-cols, 0, cols] {
            for dx in [-1, 0, 1] {
                let i = middle.wrapping_add_signed(dy + dx);
                if grid[i] != (if dy == 0 && dx == 0 { b'@' } else { b'.' }) {
                    panic!("expected empty spaces around the entrance");
                }
                grid[i] = b'#';
            }
        }

        let starts = [
            middle.wrapping_add_signed(-cols - 1),
            middle.wrapping_add_signed(-cols + 1),
            middle.wrapping_add_signed(cols - 1),
            middle.wrapping_add_signed(cols + 1),
        ];
        for i in starts {
            grid[i] = b'@';
        }

        self.minimum_steps(&grid, &starts)
    }

    fn minimum_steps(&self, grid: &[u8], starts: &[usize]) -> u32 {
        let graph = self.build_key_graph(grid, starts);
        let positions = (1u32 << starts.len()) - 1;
        let mut cache = FastMap::with_capacity(8192);
        let result = Self::search(&graph, positions, self.all_keys, &mut cache);
        if result == u32::MAX {
            panic!("no solution found");
        }
        result
    }

    fn build_key_graph(&self, grid: &[u8], starts: &[usize]) -> Vec<ArrayVec<Edge, MAX_KEYS>> {
        let cols = self.cols;
        let key_count = self.all_keys.count_ones() as usize;
        let mut key_to_node = [u8::MAX; MAX_KEYS];
        let mut key_nodes = Vec::with_capacity(key_count);
        let mut node_positions = Vec::with_capacity(starts.len() + key_count);
        let mut node_key_bits = Vec::with_capacity(starts.len() + key_count);
        node_positions.extend_from_slice(starts);
        node_key_bits.resize(starts.len(), 0);

        for (key, position) in self.key_positions.iter().enumerate() {
            if let Some(position) = *position {
                let node = node_positions.len() as u8;
                key_to_node[key] = node;
                key_nodes.push((key, node));
                node_positions.push(position);
                node_key_bits.push(1 << key);
            }
        }

        // Precompute a grid graph with corridors as weighted edges, so each later Dijkstra
        // search pushes the next grid node instead of every neighboring tile.
        let mut grid_node_ids = vec![u32::MAX; grid.len()];
        let mut grid_node_positions = Vec::new();
        for index in 0..grid.len() {
            if grid[index] == b'#' {
                continue;
            }

            let mut degree = 0;
            for next in [index - 1, index + 1, index - cols, index + cols] {
                degree += u32::from(grid[next] != b'#');
            }

            if grid[index] != b'.' || degree != 2 {
                grid_node_ids[index] = grid_node_positions.len() as u32;
                grid_node_positions.push(index);
            }
        }

        let mut grid_edges = vec![ArrayVec::<(u32, u16), 4>::new(); grid_node_positions.len()];
        for (from, &index) in grid_node_positions.iter().enumerate() {
            for mut next in [index - 1, index + 1, index - cols, index + cols] {
                if grid[next] == b'#' {
                    continue;
                }

                let mut previous = index;
                let mut edge_dist = 1u16;
                while grid_node_ids[next] == u32::MAX {
                    let mut advance = next;
                    for candidate in [next - 1, next + 1, next - cols, next + cols] {
                        if candidate != previous && grid[candidate] != b'#' {
                            advance = candidate;
                            break;
                        }
                    }
                    previous = next;
                    next = advance;
                    edge_dist += 1;
                }

                grid_edges[from]
                    .push((grid_node_ids[next], edge_dist))
                    .expect("expected at most four edges per maze node");
            }
        }

        // Run Dijkstra from each start/key to record distances and doors to each next key.
        let node_count = node_positions.len();
        let mut dist_matrix = [[u16::MAX; MAX_NODES]; MAX_NODES];
        let mut required_keys = [[0u32; MAX_NODES]; MAX_NODES];
        let mut heap = BinaryHeap::with_capacity(grid_node_positions.len());
        let mut best_dist = vec![u16::MAX; grid_node_positions.len()];
        for (from, &start_index) in node_positions.iter().enumerate() {
            best_dist.fill(u16::MAX);
            heap.push(Reverse((0u16, grid_node_ids[start_index] as usize, 0u32)));

            while let Some(Reverse((distance, grid_node, mut required_doors))) = heap.pop() {
                if distance >= best_dist[grid_node] {
                    continue;
                }
                best_dist[grid_node] = distance;

                let tile = grid[grid_node_positions[grid_node]];
                if tile.is_ascii_uppercase() {
                    required_doors |= 1 << (tile - b'A');
                }

                if tile.is_ascii_lowercase() && distance != 0 {
                    let to = key_to_node[(tile - b'a') as usize] as usize;
                    dist_matrix[from][to] = distance;
                    dist_matrix[to][from] = distance;
                    required_keys[from][to] = required_doors;
                    required_keys[to][from] = required_doors;
                    continue;
                }

                for &(next, edge_dist) in &grid_edges[grid_node] {
                    let next = next as usize;
                    let next_distance = distance.saturating_add(edge_dist);
                    if next_distance < best_dist[next] {
                        heap.push(Reverse((next_distance, next, required_doors)));
                    }
                }
            }
        }

        for (i, row) in dist_matrix.iter_mut().enumerate().take(node_count) {
            row[i] = 0;
        }

        // Add paths through other keys using Floyd-Warshall
        for k in 0..node_count {
            let via_key = node_key_bits[k];
            for i in 0..node_count {
                for j in 0..node_count {
                    let candidate = dist_matrix[i][k].saturating_add(dist_matrix[k][j]);
                    if dist_matrix[i][j] > candidate {
                        dist_matrix[i][j] = candidate;
                        required_keys[i][j] = required_keys[i][k] | via_key | required_keys[k][j];
                    }
                }
            }
        }

        let mut graph = Vec::with_capacity(node_count);
        for from in 0..node_count {
            let mut edges = ArrayVec::new();
            for &(key, target_node) in &key_nodes {
                if target_node as usize == from {
                    continue;
                }

                let distance = dist_matrix[from][target_node as usize];
                if distance != u16::MAX {
                    edges
                        .push(Edge {
                            distance: u32::from(distance),
                            needed_mask: required_keys[from][target_node as usize] | (1 << key),
                            target_node_bit: 1u32 << target_node,
                            target_key_bit: 1 << key,
                        })
                        .expect("expected at most MAX_KEYS target key edges");
                }
            }
            graph.push(edges);
        }

        graph
    }

    fn search(
        graph: &[ArrayVec<Edge, MAX_KEYS>],
        positions: u32,
        remaining: u32,
        cache: &mut FastMap<(u32, u32), u32>,
    ) -> u32 {
        if remaining == 0 {
            return 0;
        }

        let key = (positions, remaining);
        if let Some(&cached) = cache.get(&key) {
            return cached;
        }

        let mut best = u32::MAX;
        for (from, from_bit) in BitIterator::ones(positions) {
            for &edge in &graph[from as usize] {
                // needed_mask includes the target key, so one comparison checks every needed
                // key has been collected and the target key is still remaining
                if remaining & edge.needed_mask == edge.target_key_bit {
                    let rest = Self::search(
                        graph,
                        positions ^ from_bit ^ edge.target_node_bit,
                        remaining ^ edge.target_key_bit,
                        cache,
                    );
                    if rest != u32::MAX {
                        best = best.min(edge.distance + rest);
                    }
                }
            }
        }

        cache.insert(key, best);
        best
    }
}

examples!(Day18 -> (u32, u32) [
    {file: "day18_example0.txt", part1: 8},
    {file: "day18_example1.txt", part1: 86},
    {file: "day18_example2.txt", part1: 132},
    {file: "day18_example3.txt", part1: 136},
    {file: "day18_example4.txt", part1: 81},
    {file: "day18_example5.txt", part2: 8},
    {file: "day18_example6.txt", part2: 24},
    {file: "day18_example7.txt", part2: 32},
    {file: "day18_example8.txt", part2: 72},
]);
