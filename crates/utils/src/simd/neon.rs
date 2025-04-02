//! Neon vector implementations.

use std::array::from_fn;
use std::ops::{Add, BitAnd, BitOr, BitXor, Not};

#[expect(clippy::wildcard_imports)]
use std::arch::aarch64::*;

/// Neon [u32] vector implementation.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct U32Vector<const V: usize, const L: usize>([uint32x4_t; V]);

impl<const V: usize, const L: usize> From<[u32; L]> for U32Vector<V, L> {
    #[inline]
    fn from(value: [u32; L]) -> Self {
        Self(from_fn(|i| unsafe { vld1q_u32(value[i * 4..].as_ptr()) }))
    }
}

impl<const V: usize, const L: usize> From<U32Vector<V, L>> for [u32; L] {
    #[inline]
    fn from(value: U32Vector<V, L>) -> Self {
        let mut result = [0; L];
        for (&v, r) in value.0.iter().zip(result.chunks_exact_mut(4)) {
            unsafe {
                vst1q_u32(r.as_mut_ptr(), v);
            }
        }
        result
    }
}

impl<const V: usize, const L: usize> Add for U32Vector<V, L> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(from_fn(|i| unsafe { vaddq_u32(self.0[i], rhs.0[i]) }))
    }
}

impl<const V: usize, const L: usize> BitAnd for U32Vector<V, L> {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(from_fn(|i| unsafe { vandq_u32(self.0[i], rhs.0[i]) }))
    }
}

impl<const V: usize, const L: usize> BitOr for U32Vector<V, L> {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(from_fn(|i| unsafe { vorrq_u32(self.0[i], rhs.0[i]) }))
    }
}

impl<const V: usize, const L: usize> BitXor for U32Vector<V, L> {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(from_fn(|i| unsafe { veorq_u32(self.0[i], rhs.0[i]) }))
    }
}

impl<const V: usize, const L: usize> Not for U32Vector<V, L> {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        Self(from_fn(|i| unsafe {
            veorq_u32(self.0[i], vdupq_n_u32(!0))
        }))
    }
}

impl<const V: usize, const L: usize> U32Vector<V, L> {
    pub const LANES: usize = {
        assert!(V * 4 == L);
        L
    };

    #[inline]
    #[must_use]
    pub fn andnot(self, rhs: Self) -> Self {
        Self(from_fn(|i| unsafe { vbicq_u32(self.0[i], rhs.0[i]) }))
    }

    #[inline]
    #[must_use]
    pub fn splat(v: u32) -> Self {
        Self([unsafe { vdupq_n_u32(v) }; V])
    }

    #[inline]
    #[must_use]
    pub fn rotate_left(self, n: u32) -> Self {
        Self(from_fn(|i| unsafe {
            #[expect(clippy::cast_possible_wrap)]
            vorrq_u32(
                vshlq_u32(self.0[i], vdupq_n_s32(n as i32)),
                vshlq_u32(self.0[i], vdupq_n_s32(-(32 - n as i32))),
            )
        }))
    }
}

/// Vector implementations using a single Neon vector.
pub mod neon {
    /// The name of this backend.
    pub const SIMD_BACKEND: &str = "neon";

    /// Neon vector with four [u32] lanes.
    pub type U32Vector = super::U32Vector<1, 4>;
}

/// Vector implementations using two Neon vectors.
#[cfg(feature = "all-simd")]
pub mod neonx2 {
    /// The name of this backend.
    pub const SIMD_BACKEND: &str = "neonx2";

    /// Two Neon vectors with eight total [u32] lanes.
    pub type U32Vector = super::U32Vector<2, 8>;
}

/// Vector implementations using four Neon vectors.
#[cfg(feature = "all-simd")]
pub mod neonx4 {
    /// The name of this backend.
    pub const SIMD_BACKEND: &str = "neonx4";

    /// Four Neon vectors with sixteen total [u32] lanes.
    pub type U32Vector = super::U32Vector<4, 16>;
}

/// Vector implementations using eight Neon vectors.
#[cfg(feature = "all-simd")]
pub mod neonx8 {
    /// The name of this backend.
    pub const SIMD_BACKEND: &str = "neonx8";

    /// Eight Neon vectors with thirty-two total [u32] lanes.
    pub type U32Vector = super::U32Vector<8, 32>;
}
