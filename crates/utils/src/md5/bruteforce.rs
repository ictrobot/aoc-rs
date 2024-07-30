use crate::{md5, multithreading, multiversion};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Brute force hashes of a prefix followed by an increasing integer.
///
/// This function will call `predicate(i, hash(prefix + i.to_string()))` until the function returns
/// true from a pool of worker threads using vectorized MD5 implementations.
///
/// See [`md5`].
pub fn find_hash_with_appended_count(
    prefix: &str,
    predicate: impl Fn(u64, [u32; 4]) -> bool + Copy + Sync,
) {
    // Do the first 1000 separately as the length varies
    let mut buf = Vec::with_capacity(prefix.len() + 20);
    buf.extend_from_slice(prefix.as_bytes());
    buf.resize(prefix.len() + 20, 0);

    for i in 0..1000 {
        let digits = u64_to_ascii(&mut buf[prefix.len()..], i);
        let hash = super::hash(&buf[..prefix.len() + digits]);
        if predicate(i, hash) {
            return;
        }
    }

    let counter = AtomicU64::new(1000);
    let done = AtomicBool::new(false);
    multithreading::worker_pool(|| worker(prefix, &predicate, &counter, &done));
}

/// Brute force stretched hashes of a prefix followed by an increasing integer.
///
/// Similar to [`find_hash_with_appended_count`], but with key stretching (used in
/// [2016 day 14](../../year2016/struct.Day14.html)).
///
/// This function will call `predicate(i, hash(to_hex(hash(...(hash(prefix + i.to_string())))))`
/// until the function returns true from a pool of worker threads using vectorized MD5
/// implementations.
///
/// See [`md5`].
pub fn find_stretched_hash_with_appended_count(
    prefix: &str,
    additional_hashes: u32,
    predicate: impl Fn(u64, [u32; 4]) -> bool + Copy + Sync,
) {
    // Immediately use a pool of workers, which can always use the vectorized MD5 hash functions
    // when hashing the previous hashes, which takes the vast majority of the time.
    // Scalar implementations are only needed for the first round where the input lengths differ.
    let counter = AtomicU64::new(0);
    let done = AtomicBool::new(false);
    multithreading::worker_pool(|| {
        worker_stretched(prefix, additional_hashes, &predicate, &counter, &done);
    });
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

multiversion! {
    use {crate::simd::*, crate::md5::*};

    #[dyn_dispatch = md5::FASTEST]
    #[allow(clippy::cast_possible_truncation)]
    fn worker(
        prefix: &str,
        predicate: impl Fn(u64, [u32; 4]) -> bool + Copy + Send,
        counter: &AtomicU64,
        done: &AtomicBool,
    ) {
        // Create a vector containing the prefix followed by space for numbers, repeated LANES times
        let lane_size = prefix.len() + 20; // u64::MAX is 20 digits long
        let mut buf = vec![0u8; lane_size * U32Vector::LANES];
        for i in 0..U32Vector::LANES {
            buf[(lane_size * i)..(lane_size * i) + prefix.len()].copy_from_slice(prefix.as_bytes());
        }

        while !done.load(Ordering::Acquire) {
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

                for (i, &hash) in hashes.iter().enumerate() {
                    if predicate(thousands + base + i as u64, hash) {
                        // Don't return early. For example, in 2016 day 5, this block of a thousand
                        // could include more than one password letter. If we break early after
                        // completing the password with the first letter, we won't process the
                        // second letter which may have a lower count than the letter stored at that
                        // position.
                        done.store(true, Ordering::Release);
                    }
                }
            }
        }
    }
}

multiversion! {
    use {crate::simd::*, crate::md5::*};

    #[dyn_dispatch = md5::FASTEST]
    #[allow(clippy::cast_possible_truncation)]
    fn worker_stretched(
        prefix: &str,
        additional_hashes: u32,
        predicate: impl Fn(u64, [u32; 4]) -> bool + Copy + Send,
        counter: &AtomicU64,
        done: &AtomicBool,
    ) {
        // Create a vector containing the prefix followed by space for numbers, repeated LANES times
        let lane_size = prefix.len() + 20; // u64::MAX is 20 digits long
        let mut buf = vec![0u8; lane_size * U32Vector::LANES];
        for i in 0..U32Vector::LANES {
            buf[(lane_size * i)..(lane_size * i) + prefix.len()].copy_from_slice(prefix.as_bytes());
        }

        while !done.load(Ordering::Acquire) {
            let base = counter.fetch_add(U32Vector::LANES as u64, Ordering::AcqRel);

            let mut digits = [0; U32Vector::LANES];
            for (i, chunk) in buf.chunks_exact_mut(lane_size).enumerate() {
                digits[i] = u64_to_ascii(&mut chunk[prefix.len()..], base + i as u64);
            }

            let mut hashes = if digits[0] == digits[U32Vector::LANES - 1] {
                hash(&buf, lane_size, prefix.len() + digits[0])
            } else {
                // Inputs are different lengths, process the first round with scalar code
                std::array::from_fn(
                    |i| md5::hash(&buf[i*lane_size..i*lane_size + prefix.len() + digits[i]])
                )
            };

            // MD5 hashes in hex are always 32 bytes
            let mut hex_buf = [0u8; 32 * U32Vector::LANES];
            for _ in 0..additional_hashes {
                for (chunk, &hash) in hex_buf.chunks_exact_mut(32).zip(hashes.iter()) {
                    chunk.copy_from_slice(&md5::to_hex(hash));
                }
                hashes = hash(&hex_buf, 32, 32);
            }

            for (i, &hash) in hashes.iter().enumerate() {
                if predicate(base + i as u64, hash) {
                    // Don't return early, see worker
                    done.store(true, Ordering::Release);
                }
            }
        }
    }
}
