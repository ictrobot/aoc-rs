use aoc::utils::wasm::scoped_tasks::worker;
use std::alloc::{alloc_zeroed, Layout};

/// Allocate stack for worker threads.
///
/// **WARNING**: Stack overflows on worker threads will corrupt other parts of the linear memory.
#[no_mangle]
extern "C" fn allocate_stack(size: usize, align: usize) -> *mut u8 {
    let layout = Layout::from_size_align(size, align).unwrap();
    unsafe { alloc_zeroed(layout) }
}

/// Run worker thread.
#[no_mangle]
extern "C" fn worker_thread() {
    worker();
}
