//! Multithreading helpers.
//!
//! The main purpose of this module is to allow the number of worker threads used by each puzzle
//! solution to be controlled by a CLI argument.

use std::num::NonZeroUsize;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;

static NUM_THREADS: AtomicUsize = AtomicUsize::new(0);

/// Get the number of worker threads to use.
///
/// Defaults to [`std::thread::available_parallelism`] unless set by [`set_thread_count`].
#[must_use]
pub fn get_thread_count() -> NonZeroUsize {
    if let Some(threads) = NonZeroUsize::new(NUM_THREADS.load(Relaxed)) {
        threads
    } else {
        let default = std::thread::available_parallelism().unwrap_or(NonZeroUsize::new(8).unwrap());
        match NUM_THREADS.compare_exchange(0, default.get(), Relaxed, Relaxed) {
            Ok(_) => default,
            Err(not_zero) => NonZeroUsize::new(not_zero).unwrap(),
        }
    }
}

/// Set the number of worker threads to use.
///
/// This will affect any future call to [`get_thread_count`].
pub fn set_thread_count(count: NonZeroUsize) {
    NUM_THREADS.store(count.get(), Relaxed);
}

/// Run a worker function concurrently using a pool of worker threads.
///
/// This is a wrapper around [`std::thread::scope`] for spawning a pool of identical worker threads.
///
/// The number of workers is controlled by [`get_thread_count`].
pub fn worker_pool(worker: impl Fn() + Copy + Send) {
    let threads = get_thread_count().get();
    if threads == 1 {
        worker();
    } else {
        std::thread::scope(|scope| {
            for _ in 0..threads {
                scope.spawn(worker);
            }
        });
    }
}
