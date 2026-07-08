//! Hashing helpers.

// #[inline(always)] is required to ensure hashing is branchless for fixed-sized types.
#![allow(clippy::inline_always)]

#[expect(clippy::disallowed_types)]
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

// Reused wyhash secret, used as a non-zero initial state.
const SEED: u64 = 0xa076_1d64_78bd_642f;
// Reused wyhash secret, used for the fold multiply.
const MUL: u64 = 0xe703_7ed1_a0b4_28db;

/// Faster non-cryptographic hasher for small integer keys.
///
/// Intended to replace and outperform the default `SipHash` hasher for small keys used in puzzles.
/// It should not be used in cryptographic or sensitive contexts and is vulnerable to `HashDoS`
/// attacks.
///
/// Integer writes smaller than 64 bits are packed into a pending `u64`, so a key such as
/// `(u32, u16, u16)` hashes like the manually packed value `(a << 32) | (b << 16) | c`.
///
/// The mixing step is inspired by [`wyhash`](https://github.com/wangyi-fudan/wyhash). It uses XOR
/// to mix the current state and value, then does a 128bit multiplication with a large constant,
/// before XOR-ing the high and low 64bit halves.
///
/// The branches should be optimized out for fixed-sized types in release builds.
/// Example x86 assembly for [`u64`]:
///
/// ```text
/// movabsq $SEED,%rax
/// xorq    %rdi,%rax
/// movabsq $MUL,%rcx
/// mulq    %rcx
/// xorq    %rdx,%rax
/// retq
/// ```
///
/// Example x86 assembly for `(u32, u16, u16)`, showing bit packing followed by the same fold:
///
/// ```text
/// shlq    $32,%rdi
/// shll    $16,%esi
/// leaq    (%rsi,%rdi),%rax
/// movzwl  %dx,%ecx
/// orq     %rax,%rcx
/// movabsq $SEED,%rax
/// xorq    %rcx,%rax
/// movabsq $MUL,%rcx
/// mulq    %rcx
/// xorq    %rdx,%rax
/// retq
/// ```
#[derive(Clone, Debug)]
pub struct FastHasher {
    state: u64,
    pending: u64,
    bits: u32,
}

impl Default for FastHasher {
    #[inline(always)]
    fn default() -> Self {
        FastHasher {
            state: SEED,
            pending: 0,
            bits: 0,
        }
    }
}

impl FastHasher {
    #[inline]
    fn fold(state: u64, value: u64) -> u64 {
        let (lo, hi) = (state ^ value).carrying_mul(MUL, 0);
        lo ^ hi
    }

    #[inline(always)]
    fn write_bits(&mut self, value: u64, bits: u32) {
        if self.bits + bits > 64 {
            self.state = Self::fold(self.state, self.pending);
            self.pending = 0;
            self.bits = 0;
        }

        self.pending = (self.pending << bits) | value;
        self.bits += bits;
    }
}

impl Hasher for FastHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
        if self.bits > 0 {
            // Include the pending bit count if less than 64 bits so leading zeros hash differently.
            Self::fold(self.state, self.pending ^ (u64::from(self.bits & 63) << 56))
        } else {
            self.state
        }
    }

    #[inline]
    fn write(&mut self, mut bytes: &[u8]) {
        while let Some((chunk, rest)) = bytes.split_first_chunk() {
            self.write_u64(u64::from_ne_bytes(*chunk));
            bytes = rest;
        }

        if let Some((chunk, rest)) = bytes.split_first_chunk() {
            self.write_u32(u32::from_ne_bytes(*chunk));
            bytes = rest;
        }

        if let Some((chunk, rest)) = bytes.split_first_chunk() {
            self.write_u16(u16::from_ne_bytes(*chunk));
            bytes = rest;
        }

        if let Some(&byte) = bytes.first() {
            self.write_u8(byte);
        }
    }

    #[inline(always)]
    fn write_u8(&mut self, n: u8) {
        self.write_bits(u64::from(n), 8);
    }

    #[inline(always)]
    fn write_u16(&mut self, n: u16) {
        self.write_bits(u64::from(n), 16);
    }

    #[inline(always)]
    fn write_u32(&mut self, n: u32) {
        self.write_bits(u64::from(n), 32);
    }

    #[inline(always)]
    fn write_u64(&mut self, n: u64) {
        self.state = Self::fold(self.state, n);
    }

    #[inline(always)]
    #[expect(clippy::cast_possible_truncation)]
    fn write_u128(&mut self, n: u128) {
        self.write_u64(n as u64);
        self.write_u64((n >> 64) as u64);
    }

    #[inline(always)]
    #[cfg(target_pointer_width = "16")]
    fn write_usize(&mut self, n: usize) {
        self.write_u16(n as u16);
    }

    #[inline(always)]
    #[cfg(target_pointer_width = "32")]
    fn write_usize(&mut self, n: usize) {
        self.write_u32(n as u32);
    }

    #[inline(always)]
    #[cfg(target_pointer_width = "64")]
    fn write_usize(&mut self, n: usize) {
        self.write_u64(n as u64);
    }
}

/// [`BuildHasherDefault`] using [`FastHasher`].
pub type BuildFastHasher = BuildHasherDefault<FastHasher>;
/// [`HashMap`] using [`FastHasher`].
#[expect(clippy::disallowed_types)]
pub type FastMap<K, V> = HashMap<K, V, BuildFastHasher>;
/// [`HashSet`] using [`FastHasher`].
#[expect(clippy::disallowed_types)]
pub type FastSet<T> = HashSet<T, BuildFastHasher>;

/// Helper trait providing `new` and `with_capacity` functions for [`FastMap`] and [`FastSet`].
pub trait FastCollectionBuilder {
    /// Creates an empty collection.
    fn new() -> Self;

    /// Creates an empty collection with at least the specified capacity.
    fn with_capacity(capacity: usize) -> Self;
}

#[expect(clippy::implicit_hasher)]
impl<K, V> FastCollectionBuilder for FastMap<K, V> {
    #[inline]
    fn new() -> Self {
        Self::with_hasher(BuildFastHasher::default())
    }

    #[inline]
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, BuildFastHasher::default())
    }
}

#[expect(clippy::implicit_hasher)]
impl<T> FastCollectionBuilder for FastSet<T> {
    #[inline]
    fn new() -> Self {
        Self::with_hasher(BuildFastHasher::default())
    }

    #[inline]
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, BuildFastHasher::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_of(f: impl FnOnce(&mut FastHasher)) -> u64 {
        let mut hasher = FastHasher::default();
        f(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn packs_small_writes() {
        let a = hash_of(|h| {
            h.write_u16(0x1234);
            h.write_u16(0x5678);
            h.write_u32(0x9abc_def0);
        });
        let b = hash_of(|h| h.write_u64(0x1234_5678_9abc_def0));
        assert_eq!(a, b);
    }

    #[test]
    fn packs_partial_writes_with_length() {
        let a = hash_of(|h| {
            h.write_u16(0x1234);
            h.write_u16(0x5678);
            h.write_u8(0x9a);
        });
        let b = hash_of(|h| h.write_u64(0x2800_0012_3456_789a));
        assert_eq!(a, b);
    }

    #[test]
    fn field_order_matters() {
        let a = hash_of(|h| {
            h.write_u32(1);
            h.write_u32(2);
        });
        let packed_a = hash_of(|h| h.write_u64(0x0000_0001_0000_0002));
        assert_eq!(a, packed_a);

        let b = hash_of(|h| {
            h.write_u32(2);
            h.write_u32(1);
        });
        let packed_b = hash_of(|h| h.write_u64(0x0000_0002_0000_0001));
        assert_eq!(b, packed_b);

        assert_ne!(a, b);
    }

    #[test]
    fn packs_full_word_then_tags_tail() {
        let a = hash_of(|h| {
            h.write_u32(1);
            h.write_u32(2);
            h.write_u32(3);
        });
        let b = hash_of(|h| {
            h.write_u64(0x0000_0001_0000_0002);
            h.write_u64(0x2000_0000_0000_0003);
        });
        assert_eq!(a, b);
    }

    #[test]
    fn u64_write_order_matters() {
        let a = hash_of(|h| {
            h.write_u64(1);
            h.write_u64(2);
            h.write_u64(3);
        });
        let b = hash_of(|h| {
            h.write_u64(3);
            h.write_u64(2);
            h.write_u64(1);
        });
        assert_ne!(a, b);
    }

    #[test]
    fn zero_bytes_are_significant() {
        let inputs: &[&[u8]] = &[
            b"a",
            b"ab",
            b"ab\0",
            b"a\0\0\0",
            b"\0a",
            b"\0a\0",
            b"\0\0\0a",
            b"\0\0\0\0\0\0\0\0a",
            b"a\0\0\0\0\0\0\0\0",
            b"\0",
            b"\0\0",
            b"\0\0\0",
            b"\0\0\0\0",
            b"\0\0\0\0\0\0\0\0",
            b"\0\0\0\0\0\0\0\0\0",
            b"\0\0\0\0\0\0\0\0\0\0",
        ];
        for (i, a) in inputs.iter().enumerate() {
            for b in &inputs[i + 1..] {
                assert_ne!(
                    hash_of(|h| h.write(a)),
                    hash_of(|h| h.write(b)),
                    "{a:?} and {b:?} hashed the same",
                );
            }
        }
    }
}
