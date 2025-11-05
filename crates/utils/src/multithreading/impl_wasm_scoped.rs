//! Experimental threading helpers for WebAssembly.

use super::scoped_tasks::{scope, worker_count};
use std::num::{NonZero, NonZeroUsize};

#[cfg(not(all(
    target_feature = "atomics",
    target_feature = "bulk-memory",
    target_feature = "mutable-globals",
)))]
compile_error!("Required target features not enabled");

#[must_use]
pub fn get_thread_count() -> NonZeroUsize {
    // If there are no workers, `scoped_task` will fall back to running tasks on the current thread.
    NonZeroUsize::new(worker_count()).unwrap_or(NonZero::new(1).unwrap())
}

pub fn set_thread_count(_: NonZeroUsize) {
    unreachable!();
}

pub fn worker_pool(worker: impl Fn() + Copy + Send) {
    let threads = get_thread_count().get();
    scope(|scope| {
        for _ in 0..threads {
            scope.spawn(worker);
        }
    });
}
