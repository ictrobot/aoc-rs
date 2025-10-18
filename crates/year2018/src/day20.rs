use utils::prelude::*;
use utils::slice::merge_sorted_deduped_in_place;

/// Following a regex’s branching paths within a grid.
#[derive(Clone, Debug)]
pub struct Day20 {
    part1: u16,
    part2: u16,
}

const WIDTH: usize = 109;

impl Day20 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        if input.as_bytes().first() != Some(&b'^') {
            return Err(InputError::new(input, 0, "expected '^'"));
        }
        if input.as_bytes().last() != Some(&b'$') {
            return Err(InputError::new(input, input.len() - 1, "expected '$'"));
        }

        let mut grid = [0u16; WIDTH * WIDTH];
        let mut positions = vec![(WIDTH * WIDTH) / 2];
        let (mut start, mut end) = (Vec::new(), Vec::new());
        let mut stack = Vec::with_capacity(256);

        for (i, &b) in input.as_bytes()[..input.len() - 1]
            .iter()
            .enumerate()
            .skip(1)
        {
            let dir = match b {
                b'N' => -(WIDTH as isize),
                b'E' => 1,
                b'S' => WIDTH as isize,
                b'W' => -1,
                b'(' => {
                    stack.push((std::mem::take(&mut start), std::mem::take(&mut end)));
                    start.clone_from(&positions);

                    continue;
                }
                b'|' => {
                    if stack.is_empty() {
                        return Err(InputError::new(input, i, "unexpected '|'"));
                    }

                    merge_sorted_deduped_in_place(&mut end, &positions);
                    positions.clone_from(&start);

                    continue;
                }
                b')' => {
                    merge_sorted_deduped_in_place(&mut positions, &end);

                    let Some(entry) = stack.pop() else {
                        return Err(InputError::new(input, i, "unexpected ')'"));
                    };
                    (start, end) = entry;

                    continue;
                }
                _ => {
                    return Err(InputError::new(
                        input,
                        i,
                        "expected 'N', 'E', 'S', 'W', '(', '|' or ')'",
                    ));
                }
            };

            for p in positions.iter_mut() {
                let distance = grid[*p];
                *p = p.wrapping_add_signed(dir);
                if grid[*p] == 0 || distance + 1 < grid[*p] {
                    grid[*p] = distance + 1;
                }
            }
        }

        if !stack.is_empty() {
            return Err(InputError::new(input, input.len() - 1, "expected ')'"));
        }

        Ok(Self {
            part1: grid.iter().max().copied().unwrap_or(0),
            part2: grid.iter().filter(|&&d| d >= 1000).count() as u16,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u16 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u16 {
        self.part2
    }
}

examples!(Day20 -> (u16, u16) [
    {input: "^WNE$", part1: 3},
    {input: "^ENWWW(NEEE|SSE(EE|N))$", part1: 10},
    {input: "^ENNWSWW(NEWS|)SSSEEN(WNSE|)EE(SWEN|)NNN$", part1: 18},
    {input: "^ESSWWN(E|NNENN(EESS(WNSE|)SSS|WWWSSSSE(SW|NNNE)))$", part1: 23},
    {input: "^WSSEESWWWNW(S|NENNEEEENN(ESSSSW(NWSW|SSEN)|WSWWN(E|WWS(E|SS))))$", part1: 31},
]);
