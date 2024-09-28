//! WebAssembly helpers.

#[cfg(feature = "wasm-multithreading")]
pub mod scoped_tasks;

#[cfg_attr(feature = "wasm-multithreading", path = "multithreading_wasm.rs")]
#[cfg_attr(not(feature = "wasm-multithreading"), path = "multithreading_stub.rs")]
pub mod multithreading;
