//! Array vector implementations.
//!
//! Setting `N` to 1 could replace the scalar implementation, but is significantly slower in
//! unoptimised builds.

// #[inline(always)] significantly improves performance
#![allow(clippy::inline_always)]

use std::array::from_fn;
use std::ops::{Add, BitAnd, BitOr, BitXor, Not};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct U32Vector<const N: usize>([u32; N]);

impl<const N: usize> From<[u32; N]> for U32Vector<N> {
    #[inline]
    fn from(value: [u32; N]) -> Self {
        U32Vector(value)
    }
}

impl<const N: usize> From<U32Vector<N>> for [u32; N] {
    #[inline]
    fn from(value: U32Vector<N>) -> Self {
        value.0
    }
}

impl<const N: usize> Add for U32Vector<N> {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self(from_fn(|i| self.0[i].wrapping_add(rhs.0[i])))
    }
}

impl<const N: usize> BitAnd for U32Vector<N> {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(from_fn(|i| self.0[i] & rhs.0[i]))
    }
}

impl<const N: usize> BitOr for U32Vector<N> {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(from_fn(|i| self.0[i] | rhs.0[i]))
    }
}

impl<const N: usize> BitXor for U32Vector<N> {
    type Output = Self;

    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(from_fn(|i| self.0[i] ^ rhs.0[i]))
    }
}

impl<const N: usize> Not for U32Vector<N> {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Self(from_fn(|i| !self.0[i]))
    }
}

impl<const N: usize> U32Vector<N> {
    pub const LANES: usize = N;

    #[inline(always)]
    #[must_use]
    pub fn andnot(self, rhs: Self) -> Self {
        Self(from_fn(|i| self.0[i] & !rhs.0[i]))
    }

    #[inline(always)]
    #[must_use]
    pub fn splat(v: u32) -> Self {
        Self([v; N])
    }

    #[inline(always)]
    #[must_use]
    pub fn rotate_left(self, n: u32) -> Self {
        Self(from_fn(|i| self.0[i].rotate_left(n)))
    }
}

/// 128-bit wide vector implementations using arrays.
#[allow(clippy::module_name_repetitions)] // Reexported
pub mod array128 {
    /// Array vector with four [u32] lanes.
    pub type U32Vector = super::U32Vector<4>;
}

/// 256-bit wide vector implementations using arrays.
#[allow(clippy::module_name_repetitions)] // Reexported
pub mod array256 {
    /// Array vector with eight [u32] lanes.
    pub type U32Vector = super::U32Vector<8>;
}
