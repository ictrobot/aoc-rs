//! SIMD vector implementations.
//!
//! These should be replaced by [`portable_simd`](https://github.com/rust-lang/rust/issues/86656)
//! once stabilized.

pub mod scalar;

#[cfg(all(feature = "unsafe", any(target_arch = "x86", target_arch = "x86_64")))]
pub mod avx2;
