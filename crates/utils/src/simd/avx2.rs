//! AVX2 vector implementations.

use std::array::from_fn;
use std::ops::{Add, BitAnd, BitOr, BitXor, Not};

#[cfg(target_arch = "x86_64")]
#[allow(clippy::allow_attributes, clippy::wildcard_imports)]
use std::arch::x86_64::*;

#[cfg(target_arch = "x86")]
#[allow(clippy::allow_attributes, clippy::wildcard_imports)]
use std::arch::x86::*;

/// AVX2 [u32] vector implementation.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct U32Vector<const V: usize, const L: usize>([__m256i; V]);

impl<const V: usize, const L: usize> From<[u32; L]> for U32Vector<V, L> {
    #[inline]
    fn from(value: [u32; L]) -> Self {
        Self(from_fn(|i| unsafe {
            #[expect(
                clippy::cast_ptr_alignment,
                reason = "_mm256_loadu_si256 is an unaligned load which requires no alignment"
            )]
            _mm256_loadu_si256(value[i * 8..].as_ptr().cast::<__m256i>())
        }))
    }
}

impl<const V: usize, const L: usize> From<U32Vector<V, L>> for [u32; L] {
    #[inline]
    fn from(value: U32Vector<V, L>) -> Self {
        let mut result = [0; L];
        for (&v, r) in value.0.iter().zip(result.chunks_exact_mut(8)) {
            unsafe {
                #[expect(
                    clippy::cast_ptr_alignment,
                    reason = "_mm256_storeu_si256 is an unaligned store which requires no alignment"
                )]
                _mm256_storeu_si256(r.as_mut_ptr().cast::<__m256i>(), v);
            }
        }
        result
    }
}

impl<const V: usize, const L: usize> Add for U32Vector<V, L> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(from_fn(|i| unsafe {
            _mm256_add_epi32(self.0[i], rhs.0[i])
        }))
    }
}

impl<const V: usize, const L: usize> BitAnd for U32Vector<V, L> {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(from_fn(|i| unsafe {
            _mm256_and_si256(self.0[i], rhs.0[i])
        }))
    }
}

impl<const V: usize, const L: usize> BitOr for U32Vector<V, L> {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(from_fn(|i| unsafe { _mm256_or_si256(self.0[i], rhs.0[i]) }))
    }
}

impl<const V: usize, const L: usize> BitXor for U32Vector<V, L> {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(from_fn(|i| unsafe {
            _mm256_xor_si256(self.0[i], rhs.0[i])
        }))
    }
}

impl<const V: usize, const L: usize> Not for U32Vector<V, L> {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        Self(from_fn(|i| unsafe {
            _mm256_xor_si256(self.0[i], _mm256_set1_epi8(!0))
        }))
    }
}

impl<const V: usize, const L: usize> U32Vector<V, L> {
    pub const LANES: usize = {
        assert!(V * 8 == L);
        L
    };

    #[inline]
    #[must_use]
    pub fn andnot(self, rhs: Self) -> Self {
        Self(from_fn(|i| unsafe {
            _mm256_andnot_si256(rhs.0[i], self.0[i])
        }))
    }

    #[inline]
    #[must_use]
    pub fn splat(v: u32) -> Self {
        Self(
            [unsafe {
                #[expect(clippy::cast_possible_wrap)]
                _mm256_set1_epi32(v as i32)
            }; V],
        )
    }

    #[inline]
    #[must_use]
    pub fn rotate_left(self, n: u32) -> Self {
        Self(from_fn(|i| unsafe {
            #[expect(clippy::cast_possible_wrap)]
            _mm256_or_si256(
                _mm256_sll_epi32(self.0[i], _mm_cvtsi32_si128(n as i32)),
                _mm256_srl_epi32(self.0[i], _mm_cvtsi32_si128(32 - n as i32)),
            )
        }))
    }
}

/// Vector implementations using a single AVX2 vector.
pub mod avx2 {
    /// The name of this backend.
    pub const SIMD_BACKEND: &str = "avx2";

    /// AVX2 vector with eight [u32] lanes.
    pub type U32Vector = super::U32Vector<1, 8>;
}

/// Vector implementations using two AVX2 vectors.
#[cfg(feature = "all-simd")]
pub mod avx2x2 {
    /// The name of this backend.
    pub const SIMD_BACKEND: &str = "avx2x2";

    /// Two AVX2 vectors with sixteen total [u32] lanes.
    pub type U32Vector = super::U32Vector<2, 16>;
}

/// Vector implementations using four AVX2 vectors.
#[cfg(feature = "all-simd")]
pub mod avx2x4 {
    /// The name of this backend.
    pub const SIMD_BACKEND: &str = "avx2x4";

    /// Four AVX2 vectors with thirty-two total [u32] lanes.
    pub type U32Vector = super::U32Vector<4, 32>;
}

/// Vector implementations using eight AVX2 vectors.
#[cfg(feature = "all-simd")]
pub mod avx2x8 {
    /// The name of this backend.
    pub const SIMD_BACKEND: &str = "avx2x8";

    /// Eight AVX2 vectors with sixty-four total [u32] lanes.
    pub type U32Vector = super::U32Vector<8, 64>;
}
