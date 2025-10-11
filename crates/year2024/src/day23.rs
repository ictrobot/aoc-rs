use utils::bit::BitIterator;
use utils::prelude::*;

/// Finding the largest clique in a graph.
///
/// Assumes each node has the same degree N, and the largest clique contains N nodes.
#[derive(Clone, Debug)]
pub struct Day23 {
    nodes: [[u64; (26usize * 26).div_ceil(64)]; 26 * 26],
    degree: u32,
}

impl Day23 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut nodes = [[0u64; 11]; 676];

        for item in parser::byte_range(b'a'..=b'z')
            .repeat_n::<2, _>(parser::noop())
            .repeat_n::<2, _>(b'-')
            .with_eol()
            .parse_iterator(input)
        {
            let [n1, n2] = item?;

            let index1 = 26 * (n1[0] - b'a') as usize + (n1[1] - b'a') as usize;
            let index2 = 26 * (n2[0] - b'a') as usize + (n2[1] - b'a') as usize;

            nodes[index1][index2 / 64] |= 1 << (index2 % 64);
            nodes[index2][index1 / 64] |= 1 << (index1 % 64);
        }

        let Some(first_node) = nodes.iter().find(|&b| b.iter().any(|&n| n != 0)) else {
            return Err(InputError::new(input, 0, "expected non-empty graph"));
        };

        let degree = first_node.iter().map(|&n| n.count_ones()).sum::<u32>();
        if nodes.iter().any(|&b| {
            let d = b.iter().map(|&n| n.count_ones()).sum::<u32>();
            d != 0 && d != degree
        }) {
            return Err(InputError::new(
                input,
                0,
                "expected all nodes to have same degree",
            ));
        }

        Ok(Self { nodes, degree })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let mut count = 0;

        // 19 = b't' - b'a'
        for n1 in 19 * 26..20 * 26 {
            for n2 in self.neighbors(n1) {
                // Ensure the combination is only counted once if the second node also starts with t
                if n2 / 26 == 19 && n2 >= n1 {
                    continue;
                }

                for n3 in Self::iter(Self::intersect(self.nodes[n1], self.nodes[n2])) {
                    if n3 >= n2 {
                        break;
                    }
                    if n3 / 26 == 19 && n3 >= n1 {
                        continue;
                    }

                    count += 1;
                }
            }
        }

        count
    }

    #[must_use]
    pub fn part2(&self) -> String {
        for i in 0..self.nodes.len() {
            'sets: for skip in self.neighbors(i) {
                if skip > i {
                    break;
                }

                // Set of N nodes is (neighbours + starting node - skipped neighbour)
                let mut connected = self.nodes[i];
                connected[i / 64] |= 1 << (i % 64);
                connected[skip / 64] &= !(1 << (skip % 64));

                for n in self.neighbors(i).filter(|&n| n != skip) {
                    connected = Self::intersect(connected, self.nodes[n]);
                    connected[n / 64] |= 1 << (n % 64);

                    if connected.iter().map(|&n| n.count_ones()).sum::<u32>() != self.degree {
                        continue 'sets;
                    }
                }

                return Self::iter(connected).fold(String::new(), |mut acc, i| {
                    if !acc.is_empty() {
                        acc.push(',');
                    }
                    let name = [b'a' + (i / 26) as u8, b'a' + (i % 26) as u8];
                    acc.push(name[0] as char);
                    acc.push(name[1] as char);
                    acc
                });
            }
        }

        panic!("no solution found")
    }

    #[inline]
    fn neighbors(&self, n: usize) -> impl Iterator<Item = usize> {
        Self::iter(self.nodes[n])
    }

    #[inline]
    fn iter(bitset: [u64; 11]) -> impl Iterator<Item = usize> {
        bitset.into_iter().enumerate().flat_map(|(element, b)| {
            BitIterator::ones(b).map(move |(bit, _)| element * 64 + bit as usize)
        })
    }

    #[inline]
    fn intersect(mut bitset1: [u64; 11], bitset2: [u64; 11]) -> [u64; 11] {
        for (a, &b) in bitset1.iter_mut().zip(bitset2.iter()) {
            *a &= b;
        }
        bitset1
    }
}

examples!(Day23 -> (u32, &'static str) [
    {file: "day23_example0.txt", part1: 7, part2: "co,de,ka,ta"},
]);
