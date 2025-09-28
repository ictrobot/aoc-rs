use utils::geometry::{Direction, Turn};
use utils::grid;
use utils::prelude::*;

/// Simulating mine carts on intersecting tracks.
#[derive(Clone, Debug)]
pub struct Day13 {
    grid: Vec<u8>,
    carts: Vec<Cart>,
    cols: usize,
    offsets: [isize; 4],
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Cart {
    location: usize,
    direction: Direction,
    next_turn: Turn,
    crashed: bool,
}

impl Day13 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut carts = Vec::new();

        let (_, cols, grid) = grid::parse(
            input,
            1,
            0,
            |b| b,
            |b| matches!(b, b'|' | b'-' | b'/' | b'\\' | b'+' | b' '),
            |location, b| {
                let (track, direction) = match b {
                    b'^' => (b'|', Direction::Up),
                    b'v' => (b'|', Direction::Down),
                    b'<' => (b'-', Direction::Left),
                    b'>' => (b'-', Direction::Right),
                    _ => {
                        return Err(
                            "expected track ('|', '-', '/', '\\', '+'), cart ('^', 'v', '<', '>') or ' '",
                        );
                    }
                };
                carts.push(Cart {
                    location,
                    direction,
                    next_turn: Turn::Left,
                    crashed: false,
                });
                Ok(track)
            },
        )?;

        Ok(Self {
            grid,
            carts,
            cols,
            offsets: [-(cols as isize), 1, cols as isize, -1],
        })
    }

    #[must_use]
    pub fn part1(&self) -> String {
        let mut carts = self.carts.clone();

        let mut bitset = vec![0u64; self.grid.len().div_ceil(64)];
        for cart in &carts {
            bitset[cart.location / 64] |= 1 << (cart.location % 64);
        }

        loop {
            carts.sort_unstable_by_key(|cart| cart.location);

            for cart in &mut carts {
                bitset[cart.location / 64] &= !(1 << (cart.location % 64));

                self.move_cart(cart);

                if bitset[cart.location / 64] & (1 << (cart.location % 64)) != 0 {
                    return self.coordinates_str(cart.location);
                }

                bitset[cart.location / 64] |= 1 << (cart.location % 64);
            }
        }
    }

    #[must_use]
    pub fn part2(&self) -> String {
        assert!(
            !self.carts.len().is_multiple_of(2),
            "no solution found: part 2 requires an odd number of carts"
        );

        let mut carts = self.carts.clone();

        let mut bitset = vec![0u64; self.grid.len().div_ceil(64)];
        for cart in &carts {
            bitset[cart.location / 64] |= 1 << (cart.location % 64);
        }

        while carts.len() > 1 {
            carts.sort_unstable_by_key(|cart| cart.location);

            for i in 0..carts.len() {
                if carts[i].crashed {
                    continue;
                }

                let old_loc = carts[i].location;
                bitset[old_loc / 64] &= !(1 << (old_loc % 64));

                self.move_cart(&mut carts[i]);
                let new_loc = carts[i].location;

                if bitset[new_loc / 64] & (1 << (new_loc % 64)) != 0 {
                    for cart in carts.iter_mut() {
                        cart.crashed |= cart.location == new_loc;
                    }

                    bitset[new_loc / 64] &= !(1 << (new_loc % 64));
                } else {
                    bitset[new_loc / 64] |= 1 << (new_loc % 64);
                }
            }

            carts.retain(|cart| !cart.crashed);
        }
        self.coordinates_str(carts[0].location)
    }

    #[inline]
    fn move_cart(&self, cart: &mut Cart) {
        cart.location = cart
            .location
            .wrapping_add_signed(self.offsets[cart.direction as usize]);

        let track = self.grid[cart.location];
        match track {
            b'/' => {
                cart.direction = match cart.direction {
                    Direction::Up => Direction::Right,
                    Direction::Right => Direction::Up,
                    Direction::Down => Direction::Left,
                    Direction::Left => Direction::Down,
                };
            }
            b'\\' => {
                cart.direction = match cart.direction {
                    Direction::Up => Direction::Left,
                    Direction::Left => Direction::Up,
                    Direction::Right => Direction::Down,
                    Direction::Down => Direction::Right,
                };
            }
            b'+' => {
                cart.direction = cart.direction.turn(cart.next_turn);

                cart.next_turn = match cart.next_turn {
                    Turn::Left => Turn::None,
                    Turn::None => Turn::Right,
                    Turn::Right => Turn::Left,
                };
            }
            b'|' | b'-' => {}
            b' ' | 0 => panic!("no solution found: a cart left the track"),
            _ => unreachable!(),
        }
    }

    fn coordinates_str(&self, location: usize) -> String {
        format!(
            "{},{}",
            (location % self.cols) - 1,
            (location / self.cols) - 1
        )
    }
}

examples!(Day13 -> (&'static str, &'static str) [
    {
        input: "/->-\\        \n|   |  /----\\\n| /-+--+-\\  |\n| | |  | v  |\n\\-+-/  \\-+--/\n  \\------/   ",
        part1: "7,3",
    },
    {
        input: "/>-<\\  \n|   |  \n| /<+-\\\n| | | v\n\\>+</ |\n  |   ^\n  \\<->/",
        part2: "6,4",
    },
]);
