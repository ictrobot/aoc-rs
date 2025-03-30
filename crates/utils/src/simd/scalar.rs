//! Scalar vector implementations.

// #[inline(always)] significantly improves the performance of dev builds
#![allow(clippy::inline_always)]

use std::ops::{Add, BitAnd, BitOr, BitXor, Not};

/// The name of this backend.
pub const SIMD_BACKEND: &str = "scalar";

/// Scalar vector with a single [u32] lane.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct U32Vector(u32);

impl From<[u32; 1]> for U32Vector {
    #[inline]
    fn from(value: [u32; 1]) -> Self {
        U32Vector(value[0])
    }
}

impl From<U32Vector> for [u32; 1] {
    #[inline]
    fn from(value: U32Vector) -> Self {
        [value.0]
    }
}

impl Add for U32Vector {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.wrapping_add(rhs.0))
    }
}

impl BitAnd for U32Vector {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for U32Vector {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitXor for U32Vector {
    type Output = Self;

    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl Not for U32Vector {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl U32Vector {
    pub const LANES: usize = 1;

    #[inline(always)]
    #[must_use]
    pub fn andnot(self, rhs: Self) -> Self {
        Self(self.0 & !rhs.0)
    }

    #[inline(always)]
    #[must_use]
    pub fn splat(v: u32) -> Self {
        Self(v)
    }

    #[inline(always)]
    #[must_use]
    pub fn rotate_left(self, n: u32) -> Self {
        Self(self.0.rotate_left(n))
    }
}
