use utils::point::Point2D;
use utils::prelude::*;

/// Solving a sliding puzzle.
///
/// The solution assumes the input has one empty node, similar to the
/// [15 puzzle](https://en.wikipedia.org/wiki/15_puzzle), as well as one immovable wall on the right
/// (or immovable nodes below the empty node, like the example input).
#[derive(Clone, Debug)]
pub struct Day22 {
    max_x: u32,
    max_y: u32,
    wall_x: u32,
    empty: Point2D<u32>,
}

#[derive(Copy, Clone, Debug)]
struct Node {
    x: u32,
    y: u32,
    used: u32,
    avail: u32,
}

impl Day22 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let Some((_, input)) = input.split_once("Use%") else {
            return Err(InputError::new(input, 0, "expected df header"));
        };

        let nodes = parser::u32()
            .with_prefix("/dev/grid/node-x")
            .then(parser::u32().with_prefix("-y"))
            .then(
                parser::u32()
                    .with_prefix(parser::take_while1(u8::is_ascii_whitespace))
                    .with_suffix(b'T')
                    .repeat_n(parser::noop()),
            )
            .map_res(|(x, y, [size, used, avail])| {
                if size == used + avail {
                    Ok(Node { x, y, used, avail })
                } else {
                    Err("expected Used + Avail to equal Size")
                }
            })
            .with_suffix(parser::take_while1(u8::is_ascii_whitespace))
            .with_suffix(parser::number_range(0..=100))
            .with_suffix("%")
            .parse_lines(input.trim_ascii_start())?;

        let max_x = nodes.iter().map(|n| n.x).max().unwrap();
        let max_y = nodes.iter().map(|n| n.y).max().unwrap();
        if ((max_x + 1) * (max_y + 1)) as usize != nodes.len() {
            return Err(InputError::new(input, 0, "expected rectangular grid"));
        }

        // Check input has a single empty node
        let (empty, mut non_empty): (Vec<_>, Vec<_>) = nodes.into_iter().partition(|n| n.used == 0);
        let [empty] = empty[..] else {
            return Err(InputError::new(input, 0, "expected one empty node"));
        };

        // Check no viable pairs can be formed between two non-empty nodes
        let min_used = non_empty.iter().map(|n| n.used).min().unwrap_or(0);
        let max_available = non_empty.iter().map(|n| n.avail).max().unwrap_or(0);
        if min_used < max_available {
            return Err(InputError::new(
                input,
                0,
                "expected the maximum available space on non-empty nodes to be less than \
                    the minimum used space on the non-empty nodes",
            ));
        }

        // Check immovable nodes
        non_empty.retain(|n| n.used > empty.avail);
        non_empty.sort_unstable_by_key(|n| (n.y, n.x));
        let wall_x = if !non_empty.is_empty() && non_empty[0].y < empty.y {
            // Immovable nodes above empty node (real input), check they form a single wall
            if non_empty.iter().any(|n| n.y != non_empty[0].y)
                || non_empty.windows(2).any(|w| w[0].x + 1 != w[1].x)
                || non_empty[0].x == 0
                || non_empty[0].y < 2
                || non_empty.last().unwrap().x != max_x
            {
                return Err(InputError::new(
                    input,
                    0,
                    "expected either a single wall of immovable nodes",
                ));
            }
            non_empty[0].x
        } else {
            // All immovable nodes are below the empty node (which happens in the example input)
            // Add a fake to wall to the right as this adds no extra steps and avoids extra logic
            empty.x + 1
        };

        Ok(Self {
            max_x,
            max_y,
            empty: Point2D::new(empty.x, empty.y),
            wall_x,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        // All pairs are (empty node, one of the movable nodes), so the number of pairs is the
        // number of movable nodes
        (self.max_x + 1) * (self.max_y + 1) // All nodes
            - (self.max_x - self.wall_x + 1) // Minus immovable nodes
            - 1 // Minus empty node
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        (self.empty.x - (self.wall_x - 1)) // Move empty node left to avoid wall
            + self.empty.y // Move empty node up to y=0
            + ((self.max_x - 1) - (self.wall_x - 1)) // Move empty node right to x=(max - 1)
            + ((self.max_x - 1) * 5) // Shuffle goal data to x=1
            + 1 // Move goal data into x=0
    }
}

examples!(Day22 -> (u32, u32) [
    {file: "day22_example0.txt", part2: 7},
]);
