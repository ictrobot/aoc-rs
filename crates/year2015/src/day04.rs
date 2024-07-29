use std::sync::atomic::{AtomicU64, Ordering};
use utils::md5;
use utils::prelude::*;

/// Finding MD5 hashes with leading zeroes.
///
/// See [`md5::find_hash_with_appended_count()`].
#[derive(Clone, Debug)]
pub struct Day04<'a> {
    prefix: &'a str,
}

impl<'a> Day04<'a> {
    pub fn new(input: &'a str, _: InputType) -> Result<Self, InputError> {
        Ok(Self { prefix: input })
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
        let result = AtomicU64::new(u64::MAX);

        md5::find_hash_with_appended_count(self.prefix, |i, [a, ..]| {
            if i > 0 && a & mask == 0 {
                result.fetch_min(i, Ordering::AcqRel);
                true
            } else {
                false
            }
        });

        result.load(Ordering::Acquire)
    }
}

examples!(Day04<'_> -> (u64, u64) [
    {input: "abcdef", part1: 609043},
    {input: "pqrstuv", part1: 1048970},
]);
