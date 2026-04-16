//! Multithreading helpers.

#[cfg(feature = "scoped-tasks")]
pub mod scoped_tasks;

cfg_select! {
    all(target_family = "wasm", feature = "wasm-multithreading") => {
        #[path = "impl_wasm_scoped.rs"]
        mod multithreading_impl;
    }
    target_family = "wasm" => {
        #[path = "impl_wasm_stub.rs"]
        mod multithreading_impl;
    }
    _ => {
        #[path = "impl_native.rs"]
        mod multithreading_impl;
    }
}
pub use multithreading_impl::*;
