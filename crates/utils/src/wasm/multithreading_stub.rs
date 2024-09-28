//! Stub threading helpers for WebAssembly.

use std::num::NonZeroUsize;

#[must_use]
pub fn get_thread_count() -> NonZeroUsize {
    NonZeroUsize::new(1).unwrap()
}

pub fn set_thread_count(_: NonZeroUsize) {
    unreachable!();
}

pub fn worker_pool(worker: impl Fn() + Copy + Send) {
    worker();
}
