//! Multithreading helpers.

#[cfg(feature = "scoped-tasks")]
pub mod scoped_tasks;

#[cfg_attr(
    all(target_family = "wasm", feature = "wasm-multithreading"),
    path = "impl_wasm_scoped.rs"
)]
#[cfg_attr(
    all(target_family = "wasm", not(feature = "wasm-multithreading")),
    path = "impl_wasm_stub.rs"
)]
#[cfg_attr(not(target_family = "wasm"), path = "impl_native.rs")]
mod multithreading_impl;
pub use multithreading_impl::*;
