use crate::{md5, multithreading, multiversion};
use std::array;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// Brute force hashes of a prefix followed by an increasing integer.
///
/// This function calls the predicate repeatedly until it returns true from a pool of worker threads
/// each using the [`FASTEST`](super::FASTEST) supported vectorized MD5 implementation to hash
/// multiple inputs at once.
///
/// When `additional_hashes` is zero, the predicate will be called with:
/// ```ignore
/// predicate(i, hash(prefix + i.to_string()))
/// ```
///
/// When `additional_hashes` is more than zero, key stretching is used. For example, passing 2 will
/// cause the predicate to be called with:
/// ```ignore
/// predicate(i, hash(to_hex(hash(to_hex(hash(prefix + i.to_string()))))))
/// ```
pub fn find_hash_with_appended_count(
    prefix: &str,
    additional_hashes: u32,
    predicate: impl Fn(u32, [u32; 4]) -> bool + Copy + Sync,
) {
    let counter = AtomicU32::new(0);
    let done = AtomicBool::new(false);
    multithreading::worker_pool(|| {
        worker(
            prefix.as_bytes(),
            additional_hashes,
            &predicate,
            &counter,
            &done,
        );
    });
}

fn u32_to_ascii(buf: &mut [u8], mut value: u32) -> usize {
    assert!(buf.len() >= 10);

    let length = 1 + value.checked_ilog10().unwrap_or(0) as usize;
    assert!(length < 10);

    for d in (0..length).rev() {
        let new = (value % 10) as u8 + b'0';
        buf[d] = new;
        value /= 10;
    }

    length
}

multiversion! {
    use {crate::simd::*, crate::md5::*};

    #[dyn_dispatch = md5::FASTEST]
    #[allow(clippy::cast_possible_truncation)]
    fn worker(
        prefix: &[u8],
        additional_hashes: u32,
        predicate: impl Fn(u32, [u32; 4]) -> bool + Copy + Send,
        counter: &AtomicU32,
        done: &AtomicBool,
    ) {
        let lane_size = prefix.len() + 10; // u32::MAX is 10 digits long

        let mut buf = vec![0u8; lane_size * U32Vector::LANES];
        for i in 0..prefix.len() {
            buf[i * U32Vector::LANES..(i + 1) * U32Vector::LANES].fill(prefix[i]);
        }

        let mut single = vec![0u8; lane_size];
        single[..prefix.len()].copy_from_slice(prefix);

        let batch_size = if additional_hashes > 0 {
            U32Vector::LANES as u32
        }  else {
            1000u32.next_multiple_of(U32Vector::LANES as u32)
        };

        while !done.load(Ordering::Acquire) {
            let batch_start = counter.fetch_add(batch_size, Ordering::AcqRel);
            for base in (batch_start..batch_start + batch_size).step_by(U32Vector::LANES) {
                let mut hashes = match u32_to_ascii_multi(&mut buf[U32Vector::LANES * prefix.len()..], base) {
                    Some(length) => hash(&buf[..U32Vector::LANES * (prefix.len() + length.get())]),
                    None => {
                        // Lengths are different
                        array::from_fn(|i| {
                            let digits = u32_to_ascii(&mut single[prefix.len()..], base + i as u32);
                            md5::hash(&single[..prefix.len() + digits])
                        })
                    }
                };

                let mut hex_buf = [0u8; 32 * U32Vector::LANES];
                for _ in 0..additional_hashes {
                    for i in 0..U32Vector::LANES {
                        let hex = md5::to_hex(hashes[i]);
                        for h in 0..32 {
                            hex_buf[h * U32Vector::LANES + i] = hex[h];
                        }
                    }
                    hashes = hash(&hex_buf);
                }

                for (i, &hash) in hashes.iter().enumerate() {
                    if predicate(base + i as u32, hash) {
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

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    pub fn u32_to_ascii_multi(buf: &mut [u8], base: u32) -> Option<NonZeroUsize> {
        assert!(buf.len() >= U32Vector::LANES * 10);

        let length = 1 + base.checked_ilog10().unwrap_or(0) as usize;
        assert!(length <= 10);

        let mut values: [u32; U32Vector::LANES] = array::from_fn(|i| base + i as u32);
        for d in (0..length).rev() {
            let digits: &mut [u8; U32Vector::LANES] =
                (&mut buf[d * U32Vector::LANES..(d + 1) * U32Vector::LANES]).try_into().unwrap();
            for i in 0..U32Vector::LANES {
                digits[i] = (values[i] % 10) as u8 + b'0';
                values[i] /= 10;
            }
        }

        if values.iter().any(|&x| x > 0) {
            // At least one number has an extra digit, fallback to scalar code
            return None;
        }

        Some(NonZeroUsize::new(length).unwrap())
    }
}
