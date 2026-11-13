use utils::prelude::*;

/// Matching image tiles and finding a pattern.
///
/// The key optimization is that each edge is on at most two tiles, so XOR can be used to find the
/// next tile.
#[derive(Clone, Debug)]
pub struct Day20 {
    image: Vec<u128>,
    part1: u64,
}

#[derive(Clone, Copy, Debug)]
struct Tile {
    id: u16,
    pixels: u64,
    edges: [u16; 4],
}

const MAX_SIZE: usize = u128::BITS as usize / 8;

impl Day20 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let tiles = parser::number_range(1000..=9999)
            .with_prefix("Tile ")
            .with_suffix(b':')
            .with_eol()
            .then(parser::bitmask_n::<10, u16>(b".#").repeat_n::<10, _>(parser::eol()))
            .map(|(id, rows)| Tile {
                id,
                pixels: rows[1..9].iter().rfold(0, |pixels, &row| {
                    (pixels << 8) | u64::from((row >> 1) & 0xFF)
                }),
                edges: [
                    rows[0],
                    rows.iter().rfold(0, |edge, &row| (edge << 1) | (row >> 9)),
                    rows[9],
                    rows.iter().rfold(0, |edge, &row| (edge << 1) | (row & 1)),
                ],
            })
            .with_consumed()
            .with_eol()
            .parse_lines(input)?;

        let size = tiles.len().isqrt();
        if !(3..=MAX_SIZE).contains(&size) || size * size != tiles.len() {
            return Err(InputError::new(
                input,
                0,
                "expected a 3x3 to 16x16 square of tiles",
            ));
        }

        let mut ids = [0u64; 10000usize.div_ceil(64)];
        let mut counts = [0u8; 1024];
        let mut edge_tiles = [0usize; 1024];
        for (tile_idx, &(tile, tile_str)) in tiles.iter().enumerate() {
            let id = usize::from(tile.id);
            let bit = 1 << (id % 64);
            if ids[id / 64] & bit != 0 {
                return Err(InputError::new(input, tile_str, "duplicate tile id"));
            }
            ids[id / 64] |= bit;

            for edge_idx in tile.edges.map(edge_index) {
                if counts[edge_idx] == 2 {
                    return Err(InputError::new(
                        input,
                        tile_str,
                        "expected each edge on at most two tiles",
                    ));
                }
                counts[edge_idx] += 1;
                edge_tiles[edge_idx] ^= tile_idx;
            }
        }

        let Some(corner) = tiles.iter().position(|(tile, _)| {
            tile.edges
                .iter()
                .filter(|&&e| counts[edge_index(e)] == 1)
                .count()
                == 2
        }) else {
            return Err(InputError::new(input, 0, "expected a corner tile"));
        };

        let mut above_edges = [None; MAX_SIZE];
        let mut row_start = corner;
        let mut part1 = 1;
        let mut image = vec![[0u8; MAX_SIZE]; size * 8];
        for y in 0..size {
            let mut tile_idx = row_start;
            let mut left_edge = None;
            for x in 0..size {
                let (tile, tile_str) = tiles[tile_idx];

                let id = usize::from(tile.id);
                let bit = 1 << (id % 64);
                if ids[id / 64] & bit == 0 {
                    return Err(InputError::new(input, tile_str, "duplicate tile in image"));
                }
                ids[id / 64] ^= bit;

                let Some(tile) = tile.fit(above_edges[x], left_edge, &counts) else {
                    return Err(InputError::new(input, tile_str, "tile does not fit"));
                };

                left_edge = Some(tile.edges[1]);
                above_edges[x] = Some(tile.edges[2]);
                if x == 0 {
                    row_start = edge_tiles[edge_index(tile.edges[2])] ^ tile_idx;
                }
                tile_idx ^= edge_tiles[edge_index(tile.edges[1])];

                if (x == 0 || x == size - 1) && (y == 0 || y == size - 1) {
                    part1 *= u64::from(tile.id);
                }

                for (i, row) in image[y * 8..y * 8 + 8].iter_mut().enumerate() {
                    row[x] = (tile.pixels >> (i * 8)) as u8;
                }
            }
        }

        Ok(Self {
            image: image.into_iter().map(u128::from_le_bytes).collect(),
            part1,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut monsters = vec![0u128; self.image.len()];
        self.find_monsters::<0>(&mut monsters);
        self.find_monsters::<1>(&mut monsters);
        self.find_monsters::<2>(&mut monsters);
        self.find_monsters::<3>(&mut monsters);
        self.find_monsters::<4>(&mut monsters);
        self.find_monsters::<5>(&mut monsters);
        self.find_monsters::<6>(&mut monsters);
        self.find_monsters::<7>(&mut monsters);

        self.image.iter().map(|r| r.count_ones()).sum::<u32>()
            - monsters.iter().map(|r| r.count_ones()).sum::<u32>()
    }

    fn find_monsters<const ORIENTATION: usize>(&self, monsters: &mut [u128]) {
        //                   #
        // #    ##    ##    ###
        //  #  #  #  #  #  #
        #[rustfmt::skip]
        let mut pattern = [
            (18, 0),
            (0, 1), (5, 1), (6, 1), (11, 1), (12, 1), (17, 1), (18, 1), (19, 1),
            (1, 2), (4, 2), (7, 2), (10, 2), (13, 2), (16, 2),
        ];
        for (x, y) in &mut pattern {
            if ORIENTATION & 1 != 0 {
                // Flip horizontally
                *x = 19 - *x;
            }
            if ORIENTATION & 2 != 0 {
                // Flip vertically
                *y = 2 - *y;
            }
            if ORIENTATION & 4 != 0 {
                // Transpose
                (*x, *y) = (*y, *x);
            }
        }
        let rows = if ORIENTATION & 4 == 0 { 3 } else { 20 };

        for y in 0..=self.image.len() - rows {
            let mut positions = u128::MAX;
            for &(dx, dy) in &pattern {
                // Keep x bits only where (x + dx, y + dy) is set
                positions &= self.image[y + dy] >> dx;
                if positions == 0 {
                    break;
                }
            }

            if positions != 0 {
                for &(dx, dy) in &pattern {
                    monsters[y + dy] |= positions << dx;
                }
            }
        }
    }
}

impl Tile {
    fn fit(self, top: Option<u16>, left: Option<u16>, counts: &[u8; 1024]) -> Option<Self> {
        'orientations: for orientation in 0..8 {
            let [mut t, mut r, mut b, mut l] = self.edges;
            if orientation & 1 != 0 {
                // Flip horizontally
                (t, r, b, l) = (reverse(t), l, reverse(b), r);
            }
            if orientation & 2 != 0 {
                // Flip vertically
                (t, r, b, l) = (b, reverse(r), t, reverse(l));
            }
            if orientation & 4 != 0 {
                // Transpose
                (t, r, b, l) = (l, b, r, t);
            }

            for (edge, other) in [(l, left), (t, top)] {
                if other != Some(edge) && !(other.is_none() && counts[edge_index(edge)] == 1) {
                    continue 'orientations;
                }
            }

            let mut pixels = self.pixels;
            if orientation & 1 != 0 {
                pixels = pixels.reverse_bits().swap_bytes();
            }
            if orientation & 2 != 0 {
                pixels = pixels.swap_bytes();
            }
            if orientation & 4 != 0 {
                pixels = transpose(pixels);
            }

            return Some(Self {
                id: self.id,
                pixels,
                edges: [t, r, b, l],
            });
        }
        None
    }
}

fn transpose(pixels: u64) -> u64 {
    // Swap the top right and bottom left in each 2x2 block
    // .a.a.a.a
    // b.b.b.b.
    // .a.a.a.a
    // b.b.b.b.
    // .a.a.a.a
    // b.b.b.b.
    // .a.a.a.a
    // b.b.b.b.
    let pixels = delta_swap(pixels, 0x00AA_00AA_00AA_00AA, 7);

    // Swap the top right and bottom left in each 4x4 block
    // ..aa..aa
    // ..aa..aa
    // bb..bb..
    // bb..bb..
    // ..aa..aa
    // ..aa..aa
    // bb..bb..
    // bb..bb..
    let pixels = delta_swap(pixels, 0x0000_CCCC_0000_CCCC, 14);

    // Swap the top right and bottom left
    // ....aaaa
    // ....aaaa
    // ....aaaa
    // ....aaaa
    // bbbb....
    // bbbb....
    // bbbb....
    // bbbb....
    delta_swap(pixels, 0x0000_0000_F0F0_F0F0, 28)
}

fn delta_swap(x: u64, mask: u64, shift: u32) -> u64 {
    // Swap each bit in mask with the bit shift places after it
    let swap = (x ^ (x >> shift)) & mask;
    x ^ swap ^ (swap << shift)
}

fn edge_index(edge: u16) -> usize {
    usize::from(edge.min(reverse(edge)))
}

fn reverse(edge: u16) -> u16 {
    // Using a precomputed table is ~20% faster
    const REVERSED: [u16; 1024] = {
        let mut table = [0; 1024];
        let mut i = 0;
        while i < 1024 {
            table[i] = (i as u16).reverse_bits() >> 6;
            i += 1;
        }
        table
    };
    REVERSED[usize::from(edge & 1023)]
}

examples!(Day20 -> (u64, u32) [
    {file: "day20_example0.txt", part1: 20899048083289, part2: 273},
]);
