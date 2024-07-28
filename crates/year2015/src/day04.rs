use std::sync::atomic::{AtomicU64, Ordering};
use utils::prelude::*;
use utils::{md5, multithreading, multiversion};

/// Finding MD5 hashes with leading zeroes.
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
        // Do the first 1000 separately as the length varies
        let mut buf = Vec::with_capacity(self.prefix.len() + 20);
        buf.extend_from_slice(self.prefix.as_bytes());
        buf.resize(self.prefix.len() + 20, 0);

        for i in 1..1000 {
            let digits = u64_to_ascii(&mut buf[self.prefix.len()..], i);
            let [a, ..] = md5::hash(&buf[..self.prefix.len() + digits]);
            if a & mask == 0 {
                return i;
            }
        }

        let counter = AtomicU64::new(1000);
        let result = AtomicU64::new(u64::MAX);
        multithreading::worker_pool(|| worker(self.prefix, mask, &counter, &result));
        result.load(Ordering::Acquire)
    }
}

multiversion! {
    use {utils::simd::*, utils::md5::*};

    #[dyn_dispatch = md5::FASTEST]
    fn worker(prefix: &str, mask: u32, counter: &AtomicU64, result: &AtomicU64) {
        // Create a vector containing the prefix followed by space for numbers, repeated LANES times
        let lane_size = prefix.len() + 20; // u64::MAX is 20 digits long
        let mut buf = vec![0u8; lane_size * U32Vector::LANES];
        for i in 0..U32Vector::LANES {
            buf[(lane_size * i)..(lane_size * i) + prefix.len()].copy_from_slice(prefix.as_bytes());
        }

        while result.load(Ordering::Acquire) == u64::MAX {
            let thousands = counter.fetch_add(1000, Ordering::AcqRel);

            // Populate thousands in ascii in each lane
            let digits = u64_to_ascii(&mut buf[prefix.len()..], thousands);
            for i in 1..U32Vector::LANES {
                buf.copy_within(prefix.len()..prefix.len() + 20, (lane_size * i) + prefix.len());
            }

            for base in (0..1000).step_by(U32Vector::LANES) {
                // Update the last 3 digits of each number
                for (i, chunk) in buf.chunks_exact_mut(lane_size).enumerate() {
                    let n = base + i as u64;
                    chunk[prefix.len() + digits - 3] =  b'0' + (n / 100) as u8;
                    chunk[prefix.len() + digits - 2] =  b'0' + ((n / 10) % 10) as u8;
                    chunk[prefix.len() + digits - 1] =  b'0' + (n % 10) as u8;
                }

                let hashes = hash(&buf, lane_size, prefix.len() + digits);

                for (i, &[a, ..]) in hashes.iter().enumerate() {
                    if a & mask == 0 {
                        result.fetch_min(thousands + base + i as u64, Ordering::AcqRel);
                        return
                    }
                }
            }
        }
    }
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

examples!(Day04<'_> -> (u64, u64) [
    {input: "abcdef", part1: 609043},
    {input: "pqrstuv", part1: 1048970},
]);
