use utils::md5;
use utils::prelude::*;

/// Finding the shortest and longest path in a MD5 maze.
#[derive(Clone, Debug)]
pub struct Day17 {
    part1: String,
    part2: u32,
}

impl Day17 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut path = input.as_bytes().to_vec();
        let mut shortest = Vec::new();
        let mut longest = 0;
        Self::find_paths(0, 0, &mut path, input.len(), &mut shortest, &mut longest);

        Ok(Self {
            part1: String::from_utf8(shortest).unwrap(),
            part2: longest,
        })
    }

    fn find_paths(
        x: u32,
        y: u32,
        path: &mut Vec<u8>,
        prefix_len: usize,
        shortest_path: &mut Vec<u8>,
        longest_path: &mut u32,
    ) {
        if x == 3 && y == 3 {
            let path_len = path.len() - prefix_len;
            if shortest_path.is_empty() || path_len < shortest_path.len() {
                shortest_path.clear();
                shortest_path.extend_from_slice(&path[prefix_len..]);
            }
            if path_len as u32 > *longest_path {
                *longest_path = path_len as u32;
            }
            return;
        }

        let [first_u32, ..] = md5::hash(path);
        if y > 0 && (first_u32 >> 28) & 0xF > 0xA {
            path.push(b'U');
            Self::find_paths(x, y - 1, path, prefix_len, shortest_path, longest_path);
            path.pop();
        }
        if y < 3 && (first_u32 >> 24) & 0xF > 0xA {
            path.push(b'D');
            Self::find_paths(x, y + 1, path, prefix_len, shortest_path, longest_path);
            path.pop();
        }
        if x > 0 && (first_u32 >> 20) & 0xF > 0xA {
            path.push(b'L');
            Self::find_paths(x - 1, y, path, prefix_len, shortest_path, longest_path);
            path.pop();
        }
        if x < 3 && (first_u32 >> 16) & 0xF > 0xA {
            path.push(b'R');
            Self::find_paths(x + 1, y, path, prefix_len, shortest_path, longest_path);
            path.pop();
        }
    }

    #[must_use]
    pub fn part1(&self) -> &str {
        &self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.part2
    }
}

examples!(Day17 -> (&'static str, u32) [
    {
        input: "ihgpwlah",
        part1: "DDRRRD",
        part2: 370,
    },
    {
        input: "kglvqrro",
        part1: "DDUDRLRRUDRD",
        part2: 492,
    },
    {
        input: "ulqzkmiv",
        part1: "DRURDRUDDLLDLUURRDULRLDUUDDDRR",
        part2: 830,
    },
]);
