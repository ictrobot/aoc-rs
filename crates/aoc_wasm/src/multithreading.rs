use std::alloc::{Layout, alloc_zeroed};

/// Allocate stack for worker threads.
///
/// **WARNING**: Stack overflows on worker threads will corrupt other parts of the linear memory.
#[unsafe(no_mangle)]
extern "C" fn allocate_stack(size: usize, align: usize) -> *mut u8 {
    let layout = Layout::from_size_align(size, align).unwrap();
    unsafe { alloc_zeroed(layout) }
}

/// Run worker thread.
#[unsafe(no_mangle)]
extern "C" fn worker_thread() {
    #[cfg(target_family = "wasm")]
    aoc::utils::wasm::scoped_tasks::worker();

    #[cfg(not(target_family = "wasm"))]
    panic!("worker_thread is not supported on this target");
}
