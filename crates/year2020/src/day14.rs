use utils::bit::BitIterator;
use utils::hash::FastSet;
use utils::prelude::*;

/// Applying bitmasks to values and memory addresses.
#[derive(Clone, Debug)]
pub struct Day14 {
    writes: Vec<Write>,
}

#[derive(Copy, Clone, Debug)]
struct Write {
    address: u64,
    value: u64,
    ones: u64,
    floating: u64,
}

#[derive(Copy, Clone, Debug)]
struct AddressSet {
    ones: u64,
    floating: u64,
}

const VALUE_MASK: u64 = (1 << 36) - 1;
const INDEX_BITS: usize = 8;
const INDEX_MASK: u64 = (1 << INDEX_BITS) - 1;

impl Day14 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        enum Instruction {
            Mask((u64, u64)),
            Mem(u64, u64),
        }

        if input.is_empty() {
            return Err(InputError::new(input, 0, "expected instruction"));
        }

        let mask = parser::byte_map!(b'0' => (0, 0), b'1' => (1, 0), b'X' => (0, 1))
            .repeat_n::<36, _>(parser::noop())
            .map(|bits| {
                bits.iter().fold((0, 0), |(ones, floating), &(one, x)| {
                    ((ones << 1) | one, (floating << 1) | x)
                })
            });
        let number = parser::number_range(0..=VALUE_MASK);
        let instruction = parser::parse_tree!(
            ("mask = ", mask @ mask) => Instruction::Mask(mask),
            ("mem[", address @ number, "] = ", value @ number) => Instruction::Mem(address, value),
        );

        let mut writes = Vec::new();
        let mut current_mask = None;
        for item in instruction.with_eol().parse_iterator(input) {
            match item? {
                Instruction::Mask(mask) => current_mask = Some(mask),
                Instruction::Mem(address, value) => {
                    let Some((ones, floating)) = current_mask else {
                        return Err(InputError::new(input, 0, "expected mask before write"));
                    };
                    writes.push(Write {
                        address,
                        value,
                        ones,
                        floating,
                    });
                }
            }
        }

        Ok(Self { writes })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        let mut seen = FastSet::with_capacity(self.writes.len());
        let mut total = 0;
        for write in self.writes.iter().rev() {
            if seen.insert(write.address) {
                total += (write.value & write.floating) | write.ones;
            }
        }
        total
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        let sets = self
            .writes
            .iter()
            .map(|write| AddressSet {
                ones: (write.address | write.ones) & !write.floating,
                floating: write.floating,
            })
            .collect::<Vec<_>>();

        // Overlapping writes must share an address, so index later writes by the low 8 bits of
        // their addresses to avoid checking every pair
        let width = sets.len().div_ceil(64);
        let mut index = vec![0_u64; (1 << INDEX_BITS) * width];
        let mut candidates = vec![0_u64; width];
        let mut overwritten = Vec::new();
        let mut total = 0;
        for (i, &set) in sets.iter().enumerate().rev() {
            // For each index key, read the candidates from the index, then add this set
            candidates.fill(0);
            for key in set.index_keys() {
                let indexed = &index[key * width..][..width];
                for (candidate, &word) in candidates.iter_mut().zip(indexed) {
                    *candidate |= word;
                }
                index[key * width + (i / 64)] |= 1 << (i % 64);
            }

            overwritten.clear();
            for (w, &word) in candidates.iter().enumerate() {
                for (bit, _) in BitIterator::ones(word) {
                    let later = w * 64 + bit as usize;
                    if let Some(intersection) = set.intersect(sets[later]) {
                        overwritten.push(intersection);
                    }
                }
            }
            total += set.size_excluding(&overwritten) * self.writes[i].value;
        }

        total
    }
}

impl AddressSet {
    #[inline]
    fn intersect(self, other: Self) -> Option<Self> {
        let disjoint = (self.ones ^ other.ones) & !(self.floating | other.floating);
        (disjoint == 0).then_some(Self {
            ones: self.ones | other.ones,
            floating: self.floating & other.floating,
        })
    }

    #[inline]
    fn size(self) -> u64 {
        1 << self.floating.count_ones()
    }

    #[inline]
    fn index_keys(self) -> impl Iterator<Item = usize> {
        let floating = (self.floating & INDEX_MASK) as usize;
        let ones = (self.ones & INDEX_MASK) as usize;
        std::iter::successors(Some(floating), move |&subset| {
            (subset != 0).then(|| (subset - 1) & floating)
        })
        .map(move |subset| ones | subset)
    }

    #[inline]
    fn size_excluding(self, others: &[AddressSet]) -> u64 {
        let mut size = self.size();
        for (i, &other) in others.iter().enumerate() {
            if let Some(intersection) = self.intersect(other) {
                size -= intersection.size_excluding(&others[(i + 1)..]);
            }
        }
        size
    }
}

examples!(Day14 -> (u64, u64) [
    {
        input: "mask = XXXXXXXXXXXXXXXXXXXXXXXXXXXXX1XXXX0X\n\
            mem[8] = 11\n\
            mem[7] = 101\n\
            mem[8] = 0",
        part1: 165
    },
    {
        input: "mask = 000000000000000000000000000000X1001X\n\
            mem[42] = 100\n\
            mask = 00000000000000000000000000000000X0XX\n\
            mem[26] = 1",
        part2: 208
    },
]);
