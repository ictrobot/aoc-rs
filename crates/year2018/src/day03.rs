use utils::prelude::*;

/// Identifying the region that isn't overlapped.
#[derive(Clone, Debug)]
pub struct Day03 {
    part1: u32,
    part2: u32,
}

#[derive(Debug)]
struct Claim {
    id: u32,
    y: usize,
    height: usize,
    idx: usize,
    mask0: u32,
    mask1: u32,
}

impl Day03 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let coordinate = parser::number_range(0usize..=991);
        let dimension = parser::number_range(1usize..=31);

        let claims: Vec<Claim> = parser::u32()
            .with_prefix(b'#')
            .with_suffix(" @ ")
            .then(coordinate.with_suffix(b','))
            .then(coordinate.with_suffix(": "))
            .then(dimension.with_suffix(b'x'))
            .then(dimension)
            .map_res(|(id, x, y, width, height)| {
                if x + width > 1000 || y + height > 1000 {
                    Err("claim out of bounds")
                } else {
                    let mask = ((1u64 << width) - 1) << (x % 32);

                    Ok(Claim {
                        id,
                        y,
                        height,
                        idx: (x / 32),
                        mask0: mask as u32,
                        mask1: (mask >> 32) as u32,
                    })
                }
            })
            .parse_lines(input)?;

        let mut grid = [[0u32; 32]; 1000];
        let mut overlaps = [[0u32; 32]; 1000];
        for claim in &claims {
            // This is enforced by the max accepted coordinate. 991 div 32 = 30
            assert!(claim.idx + 1 < 32);

            for y in claim.y..claim.y + claim.height {
                overlaps[y][claim.idx] |= grid[y][claim.idx] & claim.mask0;
                grid[y][claim.idx] |= claim.mask0;

                overlaps[y][claim.idx + 1] |= grid[y][claim.idx + 1] & claim.mask1;
                grid[y][claim.idx + 1] |= claim.mask1;
            }
        }

        Ok(Self {
            part1: overlaps.as_flattened().iter().map(|x| x.count_ones()).sum(),
            part2: claims
                .iter()
                .find(|claim| {
                    overlaps[claim.y..claim.y + claim.height].iter().all(|o| {
                        o[claim.idx] & claim.mask0 == 0 && o[claim.idx + 1] & claim.mask1 == 0
                    })
                })
                .ok_or_else(|| InputError::new(input, 0, "all claims overlap"))?
                .id,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.part2
    }
}

examples!(Day03 -> (u32, u32) [
    {input: "#1 @ 1,3: 4x4\n#2 @ 3,1: 4x4\n#3 @ 5,5: 2x2", part1: 4},
]);
