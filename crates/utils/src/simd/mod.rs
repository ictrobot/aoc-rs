//! SIMD vector implementations.
//!
//! These should be replaced by [`portable_simd`](https://github.com/rust-lang/rust/issues/86656)
//! once stabilized.

mod array;
#[cfg(feature = "all-simd")]
pub use array::array4096;
pub use array::{array128, array256};

#[cfg(all(feature = "unsafe", any(target_arch = "x86", target_arch = "x86_64")))]
#[path = "avx2.rs"]
mod avx2_impl;
#[cfg(all(feature = "unsafe", any(target_arch = "x86", target_arch = "x86_64")))]
pub use avx2_impl::avx2;
#[cfg(all(
    feature = "unsafe",
    feature = "all-simd",
    any(target_arch = "x86", target_arch = "x86_64")
))]
pub use avx2_impl::{avx2x2, avx2x4, avx2x8};

#[cfg(all(feature = "unsafe", target_arch = "aarch64"))]
#[path = "neon.rs"]
mod neon_impl;
#[cfg(all(feature = "unsafe", target_arch = "aarch64"))]
pub use neon_impl::neon;
#[cfg(all(feature = "unsafe", feature = "all-simd", target_arch = "aarch64"))]
pub use neon_impl::{neonx2, neonx4, neonx8};

pub mod scalar;
