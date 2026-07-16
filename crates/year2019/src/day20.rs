use std::cmp::Reverse;
use std::collections::BinaryHeap;
use utils::array::ArrayVec;
use utils::grid;
use utils::prelude::*;

/// Finding the shortest paths through a recursive maze.
///
/// Similar to [Day 18](crate::Day18), the key optimization is to precompute distances between
/// every portal.
#[derive(Clone, Debug)]
pub struct Day20 {
    graph: Vec<Vec<Edge>>,
    portals: Vec<Portal>,
}

#[derive(Clone, Copy, Debug, Default)]
struct Edge {
    target: u16,
    distance: u16,
}

#[derive(Clone, Copy, Debug)]
struct Portal {
    exit: u16,
    outer: bool,
}

const LABEL_COUNT: usize = 26 * 26;
const AA: usize = 0;
const ZZ: usize = LABEL_COUNT - 1;
const START: usize = 0;
const END: usize = 1;

impl Day20 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (rows, cols, grid) = grid::parse(
            input,
            1,
            b' ',
            |b| b,
            |b| matches!(b, b' ' | b'#' | b'.' | b'A'..=b'Z'),
            |_, _| Err("expected space, '#', '.' or uppercase letter"),
        )?;
        if grid.len() >= usize::from(u16::MAX) {
            return Err(InputError::new(input, 0, "grid too large"));
        }

        let mut endpoints = [[u16::MAX; 2]; LABEL_COUNT];
        let mut endpoint_count = [0u8; LABEL_COUNT];
        let mut portal_nodes = vec![u16::MAX; grid.len()];

        // Find and validate each portal label and entrance
        for (index, &tile) in grid.iter().enumerate() {
            if !tile.is_ascii_uppercase() {
                continue;
            }

            let letter_neighbours = [index - 1, index + 1, index - cols, index + cols]
                .into_iter()
                .filter(|&next| grid[next].is_ascii_uppercase())
                .count();
            if letter_neighbours != 1 {
                return Err(InputError::new(
                    input,
                    0,
                    "expected portal labels to contain two letters",
                ));
            }

            let (second, before, after) = if grid[index + 1].is_ascii_uppercase() {
                (index + 1, index - 1, index + 2)
            } else if grid[index + cols].is_ascii_uppercase() {
                (index + cols, index - cols, index + 2 * cols)
            } else {
                continue;
            };
            let before_open = grid[before] == b'.';
            let after_open = grid[after] == b'.';
            if before_open == after_open {
                return Err(InputError::new(
                    input,
                    0,
                    "expected one open tile next to portal label",
                ));
            }

            let entrance = if before_open { before } else { after };
            if portal_nodes[entrance] != u16::MAX {
                return Err(InputError::new(
                    input,
                    0,
                    "open tile belongs to multiple portals",
                ));
            }

            let label = usize::from(tile - b'A') * 26 + usize::from(grid[second] - b'A');
            let count = endpoint_count[label] as usize;
            if matches!(label, AA | ZZ) && count != 0 {
                return Err(InputError::new(
                    input,
                    0,
                    format!("duplicate {} portal", Self::label_string(label)),
                ));
            }
            if count == 2 {
                return Err(InputError::new(
                    input,
                    0,
                    format!("more than two {} portals", Self::label_string(label)),
                ));
            }

            endpoints[label][count] = entrance as u16;
            endpoint_count[label] += 1;
            portal_nodes[entrance] = label as u16;
        }

        for label in [AA, ZZ] {
            if endpoint_count[label] == 0 {
                return Err(InputError::new(
                    input,
                    0,
                    format!("expected one {} portal", Self::label_string(label)),
                ));
            }
        }
        for (label, &count) in endpoint_count.iter().enumerate().take(ZZ).skip(1) {
            if count == 1 {
                return Err(InputError::new(
                    input,
                    0,
                    format!("expected two {} portals", Self::label_string(label)),
                ));
            }
        }

        // Allocate IDs starting with AA/START and ZZ/END
        let mut positions = Vec::new();
        let mut portals = Vec::new();
        positions.extend_from_slice(&[endpoints[AA][0], endpoints[ZZ][0]]);
        portals.extend_from_slice(&[
            Portal {
                exit: u16::MAX,
                outer: false,
            },
            Portal {
                exit: u16::MAX,
                outer: false,
            },
        ]);
        for label in 1..ZZ {
            if endpoint_count[label] == 0 {
                continue;
            }

            let [first, second] = endpoints[label];
            let first_node = positions.len() as u16;
            let second_node = first_node + 1;
            positions.extend_from_slice(&[first, second]);
            portals.extend_from_slice(&[
                Portal {
                    exit: second_node,
                    outer: false,
                },
                Portal {
                    exit: first_node,
                    outer: false,
                },
            ]);
        }
        for (node, &position) in positions.iter().enumerate() {
            portal_nodes[usize::from(position)] = node as u16;
        }

        // Compute a grid graph with corridors between entrances and junctions as weighted edges
        let mut grid_node_ids = vec![u16::MAX; grid.len()];
        let mut grid_node_positions = Vec::new();
        let (mut min_row, mut max_row, mut min_col, mut max_col) = (rows, 0, cols, 0);
        for index in 0..grid.len() {
            if grid[index] != b'.' {
                continue;
            }

            let (row, col) = (index / cols, index % cols);
            min_row = min_row.min(row);
            max_row = max_row.max(row);
            min_col = min_col.min(col);
            max_col = max_col.max(col);

            let degree = [index - 1, index + 1, index - cols, index + cols]
                .into_iter()
                .filter(|&next| grid[next] == b'.')
                .count();
            if portal_nodes[index] != u16::MAX || degree != 2 {
                grid_node_ids[index] = grid_node_positions.len() as u16;
                grid_node_positions.push(index);
            }
        }
        let mut grid_edges = vec![ArrayVec::<Edge, 4>::new(); grid_node_positions.len()];
        for (from, &position) in grid_node_positions.iter().enumerate() {
            for mut next in [position - 1, position + 1, position - cols, position + cols] {
                if grid[next] != b'.' {
                    continue;
                }

                let mut previous = position;
                let mut distance = 1u16;
                while grid_node_ids[next] == u16::MAX {
                    let advance = [next - 1, next + 1, next - cols, next + cols]
                        .into_iter()
                        .find(|&candidate| candidate != previous && grid[candidate] == b'.')
                        .unwrap();
                    previous = next;
                    next = advance;
                    distance += 1;
                }

                grid_edges[from]
                    .push(Edge {
                        target: grid_node_ids[next],
                        distance,
                    })
                    .unwrap();
            }
        }

        // Run Dijkstra from each portal entrance to every other entrance
        let mut graph = vec![Vec::new(); positions.len()];
        let mut distances = vec![u16::MAX; grid_node_positions.len()];
        // Pack (distance, node) into an u32 so heap uses faster single integer comparisons
        let mut heap = BinaryHeap::with_capacity(grid_node_positions.len());
        for (from, &start) in positions.iter().enumerate() {
            distances.fill(u16::MAX);
            heap.push(Reverse(u32::from(grid_node_ids[usize::from(start)])));

            while let Some(Reverse(entry)) = heap.pop() {
                let (distance, node) = ((entry >> 16) as u16, entry as u16 as usize);
                if distance >= distances[node] {
                    continue;
                }
                distances[node] = distance;

                let target = portal_nodes[grid_node_positions[node]];
                if target != u16::MAX && usize::from(target) != from {
                    graph[from].push(Edge { target, distance });
                }

                for &edge in &grid_edges[node] {
                    let next_distance = distance.saturating_add(edge.distance);
                    if next_distance < distances[usize::from(edge.target)] {
                        heap.push(Reverse(
                            (u32::from(next_distance) << 16) | u32::from(edge.target),
                        ));
                    }
                }
            }
        }

        // Store which portals are on the outer edges
        for (portal, &position) in portals.iter_mut().zip(&positions) {
            let position = usize::from(position);
            let (row, col) = (position / cols, position % cols);
            portal.outer = row == min_row || row == max_row || col == min_col || col == max_col;
        }

        Ok(Self { graph, portals })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let mut distances = vec![u16::MAX; self.graph.len()];
        distances[START] = 0;

        // Pack (distance, node) into an u32
        let mut heap = BinaryHeap::new();
        heap.push(Reverse(START as u32));

        while let Some(Reverse(entry)) = heap.pop() {
            let (distance, node) = ((entry >> 16) as u16, entry as u16 as usize);
            if distance != distances[node] {
                continue;
            }
            if node == END {
                return u32::from(distance);
            }

            for &edge in &self.graph[node] {
                let mut target = usize::from(edge.target);
                if target == START {
                    continue;
                }

                let mut next_distance = distance + edge.distance;
                if target != END {
                    target = usize::from(self.portals[target].exit);
                    next_distance += 1;
                }

                if next_distance < distances[target] {
                    let next_distance = next_distance;
                    distances[target] = next_distance;
                    heap.push(Reverse((u32::from(next_distance) << 16) | (target as u32)));
                }
            }
        }

        panic!("no solution found")
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        assert!(
            self.portals[START].outer,
            "expected AA to be on the outer edge"
        );
        assert!(
            self.portals[END].outer,
            "expected ZZ to be on the outer edge"
        );
        assert!(
            self.portals[2..]
                .as_chunks::<2>()
                .0
                .iter()
                .all(|[a, b]| a.outer != b.outer),
            "expected portal pairs to have one inner and one outer portal"
        );

        let node_count = self.graph.len();

        let mut distances = vec![u32::MAX; node_count];
        distances[START] = 0;

        // Pack (distance, level, node) into an u64
        let mut heap = BinaryHeap::new();
        heap.push(Reverse(START as u64));

        while let Some(Reverse(entry)) = heap.pop() {
            let (distance, level, node) = (
                (entry >> 32) as u32,
                (entry >> 16) as u16 as usize,
                entry as u16 as usize,
            );
            if distance != distances[level * node_count + node] {
                continue;
            }
            if node == END && level == 0 {
                return distance;
            }

            for &edge in &self.graph[node] {
                let mut target = usize::from(edge.target);
                let mut next_level = level;
                let mut next_distance = distance + u32::from(edge.distance);

                if target == START {
                    continue;
                } else if target == END {
                    if level != 0 {
                        continue;
                    }
                } else {
                    let portal = self.portals[target];
                    if level == 0 && portal.outer {
                        continue;
                    }
                    next_level = if portal.outer { level - 1 } else { level + 1 };
                    target = usize::from(portal.exit);
                    next_distance += 1;
                }

                let next_index = next_level * node_count + target;
                if next_index >= distances.len() {
                    distances.resize(distances.len() + node_count, u32::MAX);
                }
                if next_distance < distances[next_index] {
                    distances[next_index] = next_distance;
                    heap.push(Reverse(
                        (u64::from(next_distance) << 32)
                            | ((next_level as u64) << 16)
                            | (target as u64),
                    ));
                }
            }
        }

        panic!("no solution found")
    }

    #[cold]
    fn label_string(label: usize) -> String {
        let first = char::from(b'A' + (label / 26) as u8);
        let second = char::from(b'A' + (label % 26) as u8);
        format!("{first}{second}")
    }
}

examples!(Day20 -> (u32, u32) [
    {file: "day20_example0.txt", part1: 23},
    {file: "day20_example1.txt", part1: 58},
    {file: "day20_example2.txt", part2: 396},
]);
