//! Hashing helpers.

// #[inline(always)] is required to ensure hashing is branchless for fixed-sized types.
#![allow(clippy::inline_always)]

use std::fmt::{Debug, Formatter};
use std::hash::{BuildHasher, BuildHasherDefault, Hash, Hasher};
use std::ops::{Index, IndexMut};

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
    #[expect(clippy::cast_possible_truncation)]
    fn write_usize(&mut self, n: usize) {
        self.write_u16(n as u16);
    }

    #[inline(always)]
    #[cfg(target_pointer_width = "32")]
    #[expect(clippy::cast_possible_truncation)]
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
/// [`HashMap`](std::collections::HashMap) using [`FastHasher`].
#[expect(clippy::disallowed_types)]
pub type FastMap<K, V> = std::collections::HashMap<K, V, BuildFastHasher>;
/// [`HashSet`](std::collections::HashSet) using [`FastHasher`].
#[expect(clippy::disallowed_types)]
pub type FastSet<T> = std::collections::HashSet<T, BuildFastHasher>;

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

/// A simple hash map which assigns each key an index in insertion order.
///
/// Keys and values are stored in a [`Vec`] in insertion order. A separate hash table using linear
/// probing and [`FastHasher`] maps keys to their indexes. Keys cannot be removed.
///
/// The table is kept at most 25% full for performance. This can be faster than [`FastMap`] for
/// small maps with up to ~1000 entries.
///
/// # Examples
///
/// ```
/// # use utils::hash::IndexedMap;
/// let mut map = IndexedMap::<&str, u32>::new();
/// let index = map.intern("x");
/// map[index] += 2;
/// assert_eq!(map.try_insert("x"), Err(0));
/// assert_eq!(map.try_insert_with("y", || 5), Ok(1));
/// assert_eq!(map.into_values().collect::<Vec<_>>(), [2, 5]);
/// ```
#[derive(Clone)]
pub struct IndexedMap<K, V> {
    entries: Vec<(K, V)>,
    table: Vec<u32>,
}

/// Hash set using [`IndexedMap`].
///
/// Keys are assigned indexes in insertion order, which can be used to intern them.
///
/// # Examples
///
/// ```
/// # use utils::hash::IndexedSet;
/// let mut set = IndexedSet::new();
/// assert!(set.insert("b"));
/// assert!(!set.insert("b"));
/// assert_eq!(set.intern("a"), 1);
/// assert_eq!(set.index_of(&"a"), Some(1));
/// assert_eq!(set.index_of(&"c"), None);
/// ```
pub type IndexedSet<K> = IndexedMap<K, ()>;

const EMPTY: u32 = u32::MAX;

impl<K: Hash + Eq, V> IndexedMap<K, V> {
    /// Creates an empty map.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Creates an empty map with at least the specified capacity.
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            table: vec![EMPTY; (capacity * 4).next_power_of_two().max(64)],
        }
    }

    /// Inserts the key if it is not present, creating the value using `f`.
    ///
    /// Returns [`Ok`] containing the new index, or [`Err`] containing the existing index.
    #[inline(always)]
    #[expect(clippy::cast_possible_truncation)]
    pub fn try_insert_with(&mut self, key: K, f: impl FnOnce() -> V) -> Result<usize, usize> {
        if (self.entries.len() + 1) * 4 > self.table.len() {
            self.grow();
        }
        match self.find(&key) {
            Ok(index) => Err(index),
            Err(slot) => {
                let index = self.entries.len();
                self.table[slot] = index as u32;
                self.entries.push((key, f()));
                Ok(index)
            }
        }
    }

    /// Inserts the key if it is not present, using the default value.
    ///
    /// See [`try_insert_with`](Self::try_insert_with).
    #[inline(always)]
    pub fn try_insert(&mut self, key: K) -> Result<usize, usize>
    where
        V: Default,
    {
        self.try_insert_with(key, V::default)
    }

    /// Inserts the key if it is not present, returning whether it was inserted.
    #[inline]
    pub fn insert(&mut self, key: K) -> bool
    where
        V: Default,
    {
        self.try_insert(key).is_ok()
    }

    /// Returns the index of the key, inserting the key first if it is not present.
    #[inline]
    pub fn intern(&mut self, key: K) -> usize
    where
        V: Default,
    {
        let (Ok(index) | Err(index)) = self.try_insert(key);
        index
    }

    /// Returns the index of the provided key if it is present.
    #[inline]
    pub fn index_of(&self, key: &K) -> Option<usize> {
        self.find(key).ok()
    }

    /// Returns an iterator over the values, ordered by index.
    #[inline]
    pub fn into_values(self) -> impl Iterator<Item = V> {
        self.entries.into_iter().map(|(_, v)| v)
    }

    /// Returns the number of keys in the map.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the map is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clears the map in place.
    #[inline]
    pub fn clear(&mut self) {
        if !self.entries.is_empty() {
            self.entries.clear();
            self.table.fill(EMPTY);
        }
    }

    #[inline]
    #[expect(clippy::cast_possible_truncation)]
    fn find(&self, key: &K) -> Result<usize, usize> {
        let mask = self.table.len() - 1;
        let mut slot = BuildFastHasher::default().hash_one(key) as usize & mask;
        loop {
            match self.table[slot] {
                EMPTY => return Err(slot),
                index if self.entries[index as usize].0 == *key => return Ok(index as usize),
                _ => slot = (slot + 1) & mask,
            }
        }
    }

    #[cold]
    #[inline(never)]
    #[expect(clippy::cast_possible_truncation)]
    fn grow(&mut self) {
        let len = self.table.len() * 2;
        assert!(len / 4 < EMPTY as usize, "too many keys");
        self.table = vec![EMPTY; len];
        for (index, (key, _)) in self.entries.iter().enumerate() {
            let mut slot = BuildFastHasher::default().hash_one(key) as usize & (len - 1);
            while self.table[slot] != EMPTY {
                slot = (slot + 1) & (len - 1);
            }
            self.table[slot] = index as u32;
        }
    }
}

impl<K: Hash + Eq, V> Default for IndexedMap<K, V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Index<usize> for IndexedMap<K, V> {
    type Output = V;

    #[inline]
    fn index(&self, index: usize) -> &V {
        &self.entries[index].1
    }
}

impl<K, V> IndexMut<usize> for IndexedMap<K, V> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut V {
        &mut self.entries[index].1
    }
}

impl<K: Debug, V: Debug> Debug for IndexedMap<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.entries.iter().map(|(k, v)| (k, v)))
            .finish()
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

    #[test]
    fn indexed_map_resizes() {
        let mut map = IndexedMap::new();
        for i in 0..1000u32 {
            assert_eq!(map.try_insert_with(i * 3, || i * 5), Ok(i as usize));
        }
        assert_eq!(map.len(), 1000);
        for i in 0..1000u32 {
            let index = map.index_of(&(i * 3)).unwrap();
            assert_eq!((index, map[index]), (i as usize, i * 5));
        }

        map.clear();
        assert!(map.is_empty());
        assert_eq!(map.index_of(&3), None);
        assert_eq!(map.try_insert(6), Ok(0));
    }
}
