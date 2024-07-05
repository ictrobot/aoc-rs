use utils::md5;
use utils::prelude::*;

/// Finding MD5 hashes with leading zeroes.
#[derive(Clone, Debug)]
pub struct Day04 {
    prefix: String,
}

impl Day04 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InvalidInputError> {
        Ok(Self {
            prefix: input.to_string(),
        })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        self.find_hash_matching(0xFFFF_F000)
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        self.find_hash_matching(0xFFFF_FF00)
    }

    fn find_hash_matching(&self, mask: u32) -> u64 {
        let mut buf = Vec::with_capacity(self.prefix.len() + 20);
        buf.extend_from_slice(self.prefix.as_bytes());
        buf.resize(self.prefix.len() + 20, 0);

        for i in 1.. {
            let digits = Self::u64_to_ascii(&mut buf[self.prefix.len()..], i);
            let (a, ..) = md5::hash(&buf[..self.prefix.len() + digits]);
            if a & mask == 0 {
                return i;
            }
        }

        panic!("not found");
    }

    fn u64_to_ascii(buf: &mut [u8], mut value: u64) -> usize {
        assert!(buf.len() >= 20);
        let digits = if value == 0 {
            1
        } else {
            1 + value.ilog10() as usize
        };

        for i in (0..digits).rev() {
            let new = (value % 10) as u8 + b'0';
            buf[i] = new;
            value /= 10;
        }

        digits
    }
}

examples!(Day04<u64, u64> => [
    "abcdef" part1=609043,
    "pqrstuv" part1=1048970,
]);
