//! Experimental replacement for [`std::thread::scope`] using a fixed worker pool.
//!
//! *Scoped tasks* are similar to *scoped threads* but run on an existing thread pool instead of
//! spawning dedicated threads.
//!
//! # WebAssembly support
//!
//! This module was originally designed for WebAssembly, where it can use a pool of
//! [web worker](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API) threads spawned
//! by the host JS environment to run scoped tasks.
//!
//! Requires the `atomics`, `bulk-memory` and `mutable-globals` target features to be enabled, and
//! for all threads to be using web workers as `memory.atomic.wait` doesn't work on the main thread.
//!
//! Catching unwinding panics should be supported, but at the time of writing, the Rust standard
//! library doesn't support `panic=unwind` on WebAssembly.
//!
//! # Examples
//!
//! ```
//! # use std::num::NonZero;
//! # use std::sync::atomic::AtomicU32;
//! # use std::sync::atomic::Ordering;
//! # use utils::multithreading::scoped_tasks;
//! // Setup pool of workers. In WebAssembly, where std::thread::spawn is not available, this would
//! // be implemented by spawning more web workers which then call the exported worker function.
//! for _ in 0..std::thread::available_parallelism().map_or(4, NonZero::get) {
//!     std::thread::spawn(scoped_tasks::worker);
//! }
//!
//! let data = vec![1, 2, 3];
//! let mut data2 = vec![10, 100, 1000];
//!
//! // Start scoped tasks which may run on other threads
//! scoped_tasks::scope(|s| {
//!     s.spawn(|| {
//!        println!("[task 1] data={:?}", data);
//!     });
//!
//!     s.spawn(|| {
//!         println!("[task 2] data={:?} data2={:?}", data, data2);
//!         data2.insert(0, 1);
//!
//!         s.spawn(|| {
//!             println!("[task 3] data={:?} data2={:?}", data, data2);
//!         });
//!     });
//! });
//!
//! // Borrowed data can be used after scope
//! println!("[main] data={:?} data2={:?}", data, data2);
//! drop(data2);
//!
//! // Start another set of scoped tasks
//! let counter = AtomicU32::new(0);
//! scoped_tasks::scope(|s| {
//!     let counter = &counter;
//!     for t in 0..4 {
//!         s.spawn(move || {
//!             while let n @ 0..100 = counter.fetch_add(1, Ordering::Relaxed) {
//!                 println!("[task {t}] {n}");
//!             }
//!         });
//!     }
//! });
//! ```

use std::any::Any;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{SyncSender, TrySendError};
use std::sync::{Arc, Condvar, Mutex};

/// Create a scope for spawning scoped tasks.
///
/// Scoped tasks may borrow non-`static` data, and may run in parallel depending on thread pool
/// worker availability.
///
/// All scoped tasks are automatically joined before this function returns.
///
/// Designed to match the [`std::thread::scope`] API.
#[inline(never)]
pub fn scope<'env, F, T>(f: F) -> T
where
    F: for<'scope> FnOnce(&'scope Scope<'scope, 'env>) -> T,
{
    let scope = Scope {
        data: Arc::new(ScopeData {
            mutex: Mutex::new(ScopeCounts::default()),
            condvar: Condvar::new(),
        }),
        _scope: PhantomData,
        _env: PhantomData,
    };

    let result = catch_unwind(AssertUnwindSafe(|| f(&scope)));

    // Wait for tasks to finish
    let mut guard = scope.data.mutex.lock().unwrap();
    while guard.running > 0 {
        guard = scope.data.condvar.wait(guard).unwrap();
    }

    match result {
        Err(e) => resume_unwind(e),
        Ok(_) if guard.unhandled_panics > 0 => panic!("scoped task panicked"),
        Ok(x) => x,
    }
}

/// Scope to spawn tasks in.
///
/// [`Scope::spawn()`] is designed to match the [`std::thread::Scope`] API.
///
/// # Lifetimes
///
/// The `'scope` lifetime represents the lifetime of the scope itself, starting when the closure
/// passed to [`scope`] is run and ending when all the tasks are joined.
///
/// The `'env` lifetime represents the lifetime of the data borrowed by the scoped tasks, and must
/// outlive `'scope`.
#[derive(Debug)]
#[expect(clippy::struct_field_names)]
pub struct Scope<'scope, 'env: 'scope> {
    data: Arc<ScopeData>,
    // &'scope mut &'scope is needed to prevent lifetimes from shrinking
    _scope: PhantomData<&'scope mut &'scope ()>,
    _env: PhantomData<&'env mut &'env ()>,
}

impl<'scope> Scope<'scope, '_> {
    /// Spawn a new task within the scope.
    ///
    /// If no workers within the thread pool are available, the task will be executed on the current
    /// thread.
    pub fn spawn<F, T>(&'scope self, f: F) -> ScopedJoinHandle<'scope, T>
    where
        F: FnOnce() -> T + Send + 'scope,
        T: Send + 'scope,
    {
        let (closure, handle) = self.create_closure(f);
        if let Err(closure) = try_queue_task(closure) {
            // Fall back to running the closure on this thread
            closure();
        }
        handle
    }

    /// Spawn a new task within the scope if there is a worker available.
    ///
    /// If no workers within the thread pool are available, the task will not be executed and
    /// [`None`] will be returned.
    pub fn try_spawn<F, T>(&'scope self, f: F) -> Option<ScopedJoinHandle<'scope, T>>
    where
        F: FnOnce() -> T + Send + 'scope,
        T: Send + 'scope,
    {
        let (closure, handle) = self.create_closure(f);
        if let Ok(()) = try_queue_task(closure) {
            Some(handle)
        } else {
            // Closure will never be run
            self.data.task_end();

            None
        }
    }

    /// Spawn a new task within the scope, spawning a new worker if necessary.
    ///
    /// This function is not available on WebAssembly, as new threads have to be created from the
    /// host JS environment.
    #[cfg(not(target_family = "wasm"))]
    pub fn force_spawn<F, T>(&'scope self, f: F) -> ScopedJoinHandle<'scope, T>
    where
        F: FnOnce() -> T + Send + 'scope,
        T: Send + 'scope,
    {
        let (closure, handle) = self.create_closure(f);
        if let Err(closure) = try_queue_task(closure) {
            // Start a worker to process this closure and then join the pool.
            static THREAD_NUM: AtomicU32 = AtomicU32::new(1);
            std::thread::Builder::new()
                .name(format!(
                    "scoped-tasks-{}",
                    THREAD_NUM.fetch_add(1, Ordering::Relaxed)
                ))
                .spawn(move || {
                    // Pass the closure directly to the new worker to avoid race conditions where
                    // another scope queues a closure before this one.
                    worker_impl(closure);
                })
                .expect("failed to spawn worker thread");
        }
        handle
    }

    fn create_closure<F, T>(
        &'scope self,
        f: F,
    ) -> (Box<dyn FnOnce() + Send>, ScopedJoinHandle<'scope, T>)
    where
        F: FnOnce() -> T + Send + 'scope,
        T: Send + 'scope,
    {
        self.data.task_start();

        let handle = ScopedJoinHandle {
            data: Arc::new(HandleData {
                mutex: Mutex::new(None),
                condvar: Condvar::new(),
            }),
            scope_data: self.data.clone(),
            _scope: PhantomData,
        };

        let handle_data = handle.data.clone();
        let scope_data = self.data.clone();
        let closure: Box<dyn FnOnce() + Send + 'scope> = Box::new(
            #[inline(never)]
            move || {
                let result = catch_unwind(AssertUnwindSafe(f));

                if result.is_err() {
                    // Updating the panic count must happen before updating the handle data, to
                    // avoid the handle being joined in another thread which then tries to decrement
                    // the unhandled panic count before it is incremented
                    scope_data.task_panicked();
                }

                // Send the result to ScopedJoinHandle and wake any blocked threads
                let HandleData { mutex, condvar } = handle_data.as_ref();
                let mut guard = mutex.lock().unwrap();
                *guard = Some(result);
                condvar.notify_all();
            },
        );

        // SAFETY: The `scope` function ensures all closures are finished before returning
        let closure = unsafe {
            #[expect(clippy::unnecessary_cast, reason = "casting lifetimes")]
            Box::from_raw(Box::into_raw(closure) as *mut (dyn FnOnce() + Send + 'static))
        };

        let scope_data = self.data.clone();
        let task_closure = Box::new(move || {
            // Use a second closure to ensure that the closure which borrows from 'scope is dropped
            // before `ScopeData::task_end` is called. This prevents `scope()` from returning while
            // the inner closure still exists, which causes UB as detected by Miri.
            closure();
            scope_data.task_end();
        });

        (task_closure, handle)
    }
}

// Stores the number of currently running tasks and unhandled panics.
#[derive(Debug)]
struct ScopeData {
    mutex: Mutex<ScopeCounts>,
    condvar: Condvar,
}

#[derive(Debug, Default)]
struct ScopeCounts {
    running: usize,
    unhandled_panics: usize,
}

impl ScopeData {
    fn task_start(&self) {
        let mut guard = self.mutex.lock().unwrap();
        if let Some(new_running) = guard.running.checked_add(1) {
            guard.running = new_running;
        } else {
            panic!("too many running tasks in scope");
        }
    }

    fn task_end(&self) {
        let mut guard = self.mutex.lock().unwrap();
        if let Some(new_running) = guard.running.checked_sub(1) {
            guard.running = new_running;
            if new_running == 0 {
                self.condvar.notify_all();
            }
        } else {
            panic!("more tasks finished than started?")
        }
    }

    fn task_panicked(&self) {
        let mut guard = self.mutex.lock().unwrap();
        if let Some(new_panicked) = guard.unhandled_panics.checked_add(1) {
            guard.unhandled_panics = new_panicked;
        } else {
            panic!("too many panicking tasks in scope");
        }
    }

    fn panic_joined(&self) {
        let mut guard = self.mutex.lock().unwrap();
        if let Some(new_panicked) = guard.unhandled_panics.checked_sub(1) {
            guard.unhandled_panics = new_panicked;
        } else {
            panic!("more panics joined than tasks panicked?")
        }
    }
}

/// Handle to block on a task's termination.
///
/// Designed to match the [`std::thread::ScopedJoinHandle`] API, except
/// [`std::thread::ScopedJoinHandle::thread`] is not supported as tasks are not run on dedicated
/// threads.
#[derive(Debug)]
pub struct ScopedJoinHandle<'scope, T> {
    data: Arc<HandleData<T>>,
    scope_data: Arc<ScopeData>,
    _scope: PhantomData<&'scope mut &'scope ()>,
}

#[derive(Debug)]
struct HandleData<T> {
    mutex: Mutex<Option<Result<T, Box<dyn Any + Send + 'static>>>>,
    condvar: Condvar,
}

impl<T> ScopedJoinHandle<'_, T> {
    /// Wait for the task to finish.
    ///
    /// The [`Err`] variant contains the panic value if the task panicked.
    pub fn join(self) -> Result<T, Box<dyn Any + Send + 'static>> {
        let result = {
            let HandleData { mutex, condvar } = self.data.as_ref();
            let mut guard = mutex.lock().unwrap();
            loop {
                if let Some(result) = guard.take() {
                    break result;
                }
                guard = condvar.wait(guard).unwrap();
            }
        };

        if result.is_err() {
            self.scope_data.panic_joined();
        }

        result
    }

    /// Check if the task is finished.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.data.mutex.lock().unwrap().is_some()
    }
}

#[expect(clippy::type_complexity)]
static WORKERS: Mutex<VecDeque<SyncSender<Box<dyn FnOnce() + Send>>>> = Mutex::new(VecDeque::new());

fn try_queue_task(mut closure: Box<dyn FnOnce() + Send>) -> Result<(), Box<dyn FnOnce() + Send>> {
    let mut guard = WORKERS.lock().unwrap();
    let queue = &mut *guard;

    for _ in 0..queue.len() {
        let Some(sender) = queue.pop_front() else {
            break;
        };

        match sender.try_send(closure) {
            Ok(()) => {
                queue.push_back(sender);
                return Ok(());
            }
            Err(TrySendError::Full(v)) => {
                closure = v;
                queue.push_back(sender);
            }
            Err(TrySendError::Disconnected(v)) => {
                closure = v;
                // Worker thread gone, discard sender
            }
        }
    }
    drop(guard);

    Err(closure)
}

/// Use this thread as a worker in the thread pool for scoped tasks.
///
/// This function never returns.
pub fn worker() {
    worker_impl(|| {});
}

fn worker_impl(initial: impl FnOnce() + Send) {
    initial();

    let (tx, rx) = std::sync::mpsc::sync_channel(0);

    {
        let mut guard = WORKERS.lock().unwrap();
        guard.push_front(tx);
    }

    for closure in rx {
        closure();
    }
}

/// Returns the current number of workers for scoped tasks.
pub fn worker_count() -> usize {
    WORKERS.lock().unwrap().len()
}
