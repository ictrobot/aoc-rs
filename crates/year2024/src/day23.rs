use utils::prelude::*;

/// Finding the largest clique in a graph.
#[derive(Clone, Debug)]
pub struct Day23 {
    nodes: Vec<Node>,
}

#[derive(Clone, Debug)]
struct Node {
    name: [u8; 2],
    edges: Vec<usize>,
}

impl Day23 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut indexes = [None; 26 * 26];
        let mut nodes = vec![];

        for item in parser::byte_range(b'a'..=b'z')
            .repeat_n(parser::noop())
            .repeat_n(b'-')
            .with_suffix(parser::eol())
            .parse_iterator(input)
        {
            let [n1, n2] = item?;

            let mut index_of = |n: [u8; 2]| {
                let i = 26 * (n[0] - b'a') as usize + (n[1] - b'a') as usize;
                match indexes[i] {
                    Some(index) => index,
                    None => {
                        let index = nodes.len();
                        nodes.push(Node {
                            name: n,
                            edges: Vec::new(),
                        });
                        indexes[i] = Some(index);
                        index
                    }
                }
            };

            let index1 = index_of(n1);
            let index2 = index_of(n2);

            nodes[index1].edges.push(index2);
            nodes[index2].edges.push(index1);
        }

        Ok(Self { nodes })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let mut count = 0;
        for (i1, n1) in self.nodes.iter().enumerate() {
            for (i2, n2) in n1.edges.iter().map(|&i| (i, &self.nodes[i])) {
                if i1 > i2 {
                    continue;
                }
                for (i3, n3) in n2.edges.iter().map(|&i| (i, &self.nodes[i])) {
                    if i2 < i3
                        && n1.edges.contains(&i3)
                        && (n1.name[0] == b't' || n2.name[0] == b't' || n3.name[0] == b't')
                    {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    #[must_use]
    pub fn part2(&self) -> String {
        let mut assigned = vec![false; self.nodes.len()];
        let mut node_lists = Vec::with_capacity(self.nodes.len());
        for i in 0..self.nodes.len() {
            if !assigned[i] {
                let c = node_lists.len();
                node_lists.push(Vec::new());
                self.try_add(i, &mut assigned, &mut node_lists[c]);
            }
        }

        let mut nodes = node_lists.into_iter().max_by_key(|l| l.len()).unwrap();
        nodes.sort_unstable_by_key(|&n| self.nodes[n].name);
        nodes
            .iter()
            .fold(String::with_capacity(nodes.len() * 3), |mut acc, &i| {
                if !acc.is_empty() {
                    acc.push(',');
                }
                acc.push(self.nodes[i].name[0] as char);
                acc.push(self.nodes[i].name[1] as char);
                acc
            })
    }

    fn try_add(&self, n: usize, assigned: &mut [bool], group: &mut Vec<usize>) {
        for &existing in group.iter() {
            if !self.nodes[n].edges.contains(&existing) {
                return;
            }
        }

        group.push(n);
        assigned[n] = true;

        for &neighbour in self.nodes[n].edges.iter() {
            if !assigned[neighbour] {
                self.try_add(neighbour, assigned, group);
            }
        }
    }
}

examples!(Day23 -> (u32, &'static str) []);
