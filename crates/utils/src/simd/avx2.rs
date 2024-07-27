//! AVX2 vector implementations.

use std::ops::{Add, BitAnd, BitOr, BitXor, Not};

#[cfg(target_arch = "x86_64")]
#[allow(clippy::wildcard_imports)]
use std::arch::x86_64::*;

#[cfg(target_arch = "x86")]
#[allow(clippy::wildcard_imports)]
use std::arch::x86::*;

/// AVX2 vector with eight [u32] lanes.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct U32Vector(__m256i);

impl Add for U32Vector {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(unsafe { _mm256_add_epi32(self.0, rhs.0) })
    }
}

impl BitAnd for U32Vector {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(unsafe { _mm256_and_si256(self.0, rhs.0) })
    }
}

impl BitOr for U32Vector {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(unsafe { _mm256_or_si256(self.0, rhs.0) })
    }
}

impl BitXor for U32Vector {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(unsafe { _mm256_xor_si256(self.0, rhs.0) })
    }
}

impl Not for U32Vector {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        Self(unsafe { _mm256_xor_si256(self.0, _mm256_set1_epi8(!0)) })
    }
}

impl U32Vector {
    pub const LANES: usize = 8;

    #[inline]
    #[must_use]
    pub fn load(from: &[u32; Self::LANES]) -> Self {
        Self(unsafe {
            // _mm256_loadu_si256 is an unaligned load which requires no alignment
            #[allow(clippy::cast_ptr_alignment)]
            _mm256_loadu_si256(from.as_ptr().cast::<__m256i>())
        })
    }

    #[inline]
    pub fn store(self, to: &mut [u32; Self::LANES]) {
        unsafe {
            // _mm256_storeu_si256 is an unaligned store which requires no alignment
            #[allow(clippy::cast_ptr_alignment)]
            _mm256_storeu_si256(to.as_mut_ptr().cast::<__m256i>(), self.0);
        }
    }

    #[inline]
    #[must_use]
    pub fn andnot(self, rhs: Self) -> Self {
        Self(unsafe { _mm256_andnot_si256(rhs.0, self.0) })
    }

    #[inline]
    #[must_use]
    pub fn splat(v: u32) -> Self {
        Self(unsafe {
            #[allow(clippy::cast_possible_wrap)]
            _mm256_set1_epi32(v as i32)
        })
    }

    #[inline]
    #[must_use]
    pub fn rotate_left(self, n: u32) -> Self {
        Self(unsafe {
            #[allow(clippy::cast_possible_wrap)]
            _mm256_or_si256(
                _mm256_sll_epi32(self.0, _mm_cvtsi32_si128(n as i32)),
                _mm256_srl_epi32(self.0, _mm_cvtsi32_si128(32 - n as i32)),
            )
        })
    }
}
