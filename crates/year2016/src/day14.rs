use std::collections::{BTreeMap, BTreeSet};
use std::ops::DerefMut;
use std::sync::Mutex;
use utils::md5;
use utils::prelude::*;

/// Finding MD5 hashes, part three.
///
/// Similar to [2015 Day 4](../year2015/struct.Day04.html) and [2016 Day 5](crate::Day05), but with
/// even more complex logic to assemble the answer from the matching hashes and key stretching for
/// part 2.
///
/// See [`md5::find_hash_with_appended_count()`] and
/// [`md5::find_stretched_hash_with_appended_count()`].
#[derive(Clone, Debug)]
pub struct Day14<'a> {
    prefix: &'a str,
}

impl<'a> Day14<'a> {
    pub fn new(input: &'a str, _: InputType) -> Result<Self, InputError> {
        Ok(Self { prefix: input })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        // TODO using multiple threads is not worth it for ~10,000 hashes
        self.find_64th_key(0)
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        self.find_64th_key(2016)
    }

    fn find_64th_key(&self, additional_hashes: u32) -> u64 {
        let mutex = Mutex::new((BTreeSet::new(), BTreeMap::new(), BTreeMap::new()));

        let predicate = |i: u64, [a, b, c, d]: [u32; 4]| {
            // Spread each nibble into bytes like utils::md5::u32_to_hex, but without the
            // unnecessary mapping to ASCII hex
            let mut nibble_bytes = [0u8; 32];
            nibble_bytes[0..8].copy_from_slice(&spread_nibbles(a).to_be_bytes());
            nibble_bytes[8..16].copy_from_slice(&spread_nibbles(b).to_be_bytes());
            nibble_bytes[16..24].copy_from_slice(&spread_nibbles(c).to_be_bytes());
            nibble_bytes[24..32].copy_from_slice(&spread_nibbles(d).to_be_bytes());

            let Some(triplet) = nibble_bytes
                .windows(3)
                .find(|&w| w[0] == w[1] && w[0] == w[2])
                .map(|w| 1u16 << w[0])
            else {
                // No triplet means there is also no quintuplet
                return false;
            };

            let quintuplet = nibble_bytes
                .windows(5)
                .filter(|&w| w[0] == w[1] && w[0] == w[2] && w[0] == w[3] && w[0] == w[4])
                .fold(0, |acc, w| acc | 1u16 << w[0]);

            let mut guard = mutex.lock().unwrap();
            let (ref mut keys, ref mut triplets, ref mut quintuplets) = guard.deref_mut();

            triplets.insert(i, triplet);

            // Check if a matching quintuplet has already been found in the following thousand
            if quintuplets
                .range(i + 1..i + 1001)
                .any(|(_, &m)| triplet & m != 0)
            {
                keys.insert(i);
            }

            if quintuplet != 0 {
                quintuplets.insert(i, quintuplet);

                // Check if any matching triplets have already been found in the previous thousand
                triplets
                    .range(i.saturating_sub(1000)..i)
                    .filter(|(_, &m)| quintuplet & m != 0)
                    .for_each(|(&k, _)| {
                        keys.insert(k);
                    });
            }

            keys.len() >= 64
        };

        if additional_hashes == 0 {
            md5::find_hash_with_appended_count(self.prefix, predicate);
        } else {
            md5::find_stretched_hash_with_appended_count(self.prefix, additional_hashes, predicate);
        }

        let (keys, ..) = mutex.into_inner().unwrap();
        *keys.iter().nth(63).unwrap()
    }
}

fn spread_nibbles(n: u32) -> u64 {
    let mut n = u64::from(n);
    // n = 0x0000_0000_1234_5678

    n = ((n & 0x0000_0000_FFFF_0000) << 16) | (n & 0x0000_0000_0000_FFFF);
    // n = 0x0000_1234_0000_5678

    n = ((n & 0x0000_FF00_0000_FF00) << 8) | (n & 0x0000_00FF_0000_00FF);
    // n = 0x0012_0034_0056_0078

    n = ((n & 0x00F0_00F0_00F0_00F0) << 4) | (n & 0x000F_000F_000F_000F);
    // n = 0x0102_0304_0506_0708

    n
}

examples!(Day14<'_> -> (u64, u64) [
    {input: "abc", part1: 22728, part2: 22551},
]);
