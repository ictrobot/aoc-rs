//! AVX512 vector implementations.
//!
//! Currently only requires AVX-512F.

use std::array::from_fn;
use std::ops::{Add, BitAnd, BitOr, BitXor, Not};

#[cfg(target_arch = "x86_64")]
#[allow(clippy::allow_attributes, clippy::wildcard_imports)]
use std::arch::x86_64::*;

#[cfg(target_arch = "x86")]
#[allow(clippy::allow_attributes, clippy::wildcard_imports)]
use std::arch::x86::*;

/// AVX512 [u32] vector implementation.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct U32Vector<const V: usize, const L: usize>([__m512i; V]);

impl<const V: usize, const L: usize> From<[u32; L]> for U32Vector<V, L> {
    #[inline]
    fn from(value: [u32; L]) -> Self {
        Self(from_fn(|i| unsafe {
            #[expect(
                clippy::cast_ptr_alignment,
                reason = "_mm512_loadu_si512 is an unaligned load which requires no alignment"
            )]
            _mm512_loadu_si512(value[i * 16..].as_ptr().cast::<__m512i>())
        }))
    }
}

impl<const V: usize, const L: usize> From<U32Vector<V, L>> for [u32; L] {
    #[inline]
    fn from(value: U32Vector<V, L>) -> Self {
        let mut result = [0; L];
        for (&v, r) in value.0.iter().zip(result.chunks_exact_mut(16)) {
            unsafe {
                #[expect(
                    clippy::cast_ptr_alignment,
                    reason = "_mm512_storeu_si512 is an unaligned store which requires no alignment"
                )]
                _mm512_storeu_si512(r.as_mut_ptr().cast::<__m512i>(), v);
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
            _mm512_add_epi32(self.0[i], rhs.0[i])
        }))
    }
}

impl<const V: usize, const L: usize> BitAnd for U32Vector<V, L> {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(from_fn(|i| unsafe {
            _mm512_and_si512(self.0[i], rhs.0[i])
        }))
    }
}

impl<const V: usize, const L: usize> BitOr for U32Vector<V, L> {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(from_fn(|i| unsafe { _mm512_or_si512(self.0[i], rhs.0[i]) }))
    }
}

impl<const V: usize, const L: usize> BitXor for U32Vector<V, L> {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(from_fn(|i| unsafe {
            _mm512_xor_si512(self.0[i], rhs.0[i])
        }))
    }
}

impl<const V: usize, const L: usize> Not for U32Vector<V, L> {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        Self(from_fn(|i| unsafe {
            _mm512_xor_si512(self.0[i], _mm512_set1_epi8(!0))
        }))
    }
}

impl<const V: usize, const L: usize> U32Vector<V, L> {
    pub const LANES: usize = {
        assert!(V * 16 == L);
        L
    };

    #[inline]
    #[must_use]
    #[target_feature(enable = "avx512f")]
    pub fn andnot(self, rhs: Self) -> Self {
        Self(from_fn(|i| _mm512_andnot_si512(rhs.0[i], self.0[i])))
    }

    #[inline]
    #[must_use]
    #[target_feature(enable = "avx512f")]
    pub fn splat(v: u32) -> Self {
        Self(
            #[expect(clippy::cast_possible_wrap)]
            [_mm512_set1_epi32(v as i32); V],
        )
    }

    #[inline]
    #[must_use]
    #[target_feature(enable = "avx512f")]
    pub fn rotate_left(self, n: u32) -> Self {
        Self(from_fn(|i| {
            #[expect(clippy::cast_possible_wrap)]
            _mm512_or_si512(
                _mm512_sll_epi32(self.0[i], _mm_cvtsi32_si128(n as i32)),
                _mm512_srl_epi32(self.0[i], _mm_cvtsi32_si128(32 - n as i32)),
            )
        }))
    }
}

/// Vector implementations using a single AVX512 vector.
pub mod avx512 {
    /// The name of this backend.
    pub const SIMD_BACKEND: &str = "avx512";

    /// AVX512 vector with sixteen [u32] lanes.
    pub type U32Vector = super::U32Vector<1, 16>;
}

/// Vector implementations using two AVX512 vectors.
#[cfg(feature = "all-simd")]
pub mod avx512x2 {
    /// The name of this backend.
    pub const SIMD_BACKEND: &str = "avx512x2";

    /// Two AVX512 vectors with thirty-two total [u32] lanes.
    pub type U32Vector = super::U32Vector<2, 32>;
}

/// Vector implementations using four AVX512 vectors.
#[cfg(feature = "all-simd")]
pub mod avx512x4 {
    /// The name of this backend.
    pub const SIMD_BACKEND: &str = "avx512x4";

    /// Four AVX512 vectors with sixty-four total [u32] lanes.
    pub type U32Vector = super::U32Vector<4, 64>;
}

/// Vector implementations using eight AVX512 vectors.
#[cfg(feature = "all-simd")]
pub mod avx512x8 {
    /// The name of this backend.
    pub const SIMD_BACKEND: &str = "avx512x8";

    /// Eight AVX512 vectors with 128 total [u32] lanes.
    pub type U32Vector = super::U32Vector<8, 128>;
}
