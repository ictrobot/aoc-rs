use utils::point::Point2D;
use utils::prelude::*;

/// Following a path.
#[derive(Clone, Debug)]
pub struct Day19 {
    part1: String,
    part2: u32,
}

impl Day19 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let lines = input.lines().collect::<Vec<_>>();
        let lookup = |p: Point2D<usize>| {
            if p.y < lines.len() && p.x < lines[p.y].len() {
                Some(lines[p.y].as_bytes()[p.x])
            } else {
                None
            }
        };

        let Some(start_col) = lines.first().and_then(|l| l.find('|')) else {
            return Err(InputError::new(input, 0, "expected '|' on the first line"));
        };
        let mut pos = Point2D::new(start_col, 0);
        let mut dir = Point2D::new(0, 1);

        let mut letters = String::new();
        let mut steps = 1;
        loop {
            match lookup(pos)
                .ok_or_else(|| InputError::new(input, 0, "path leads outside the input"))?
            {
                b'|' | b'-' => {}
                b'+' => {
                    let left = lookup(pos.wrapping_add_signed(dir.turn_left())).unwrap_or(b' ');
                    let right = lookup(pos.wrapping_add_signed(dir.turn_right())).unwrap_or(b' ');
                    if matches!(left, b'|' | b'-' | b'A'..=b'Z') && right == b' ' {
                        dir = dir.turn_left();
                    } else if matches!(right, b'|' | b'-' | b'A'..=b'Z') && left == b' ' {
                        dir = dir.turn_right();
                    } else {
                        return Err(InputError::new(
                            input,
                            &lines[pos.y][pos.x..],
                            "invalid turn",
                        ));
                    }
                }
                letter @ b'A'..=b'Z' => {
                    letters.push(letter as char);

                    // The path is allowed to end after letters
                    if lookup(pos.wrapping_add_signed(dir)).unwrap_or(b' ') == b' ' {
                        break;
                    }
                }
                _ => {
                    return Err(InputError::new(
                        input,
                        &lines[pos.y][pos.x..],
                        "expected '|', '-', '+' or 'A'-'Z'",
                    ));
                }
            }

            pos = pos.wrapping_add_signed(dir);
            steps += 1;
        }

        Ok(Self {
            part1: letters,
            part2: steps,
        })
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

examples!(Day19 -> (&'static str, u32) [
    {file: "day19_example0.txt", part1: "ABCDEF", part2: 38},
]);
