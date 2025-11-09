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
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
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
        running: Arc::default(),
        panicked: Arc::default(),
        _scope: PhantomData,
        _env: PhantomData,
    };

    let result = catch_unwind(AssertUnwindSafe(|| f(&scope)));

    scope.running.wait_for_tasks();

    match result {
        Err(e) => resume_unwind(e),
        Ok(_) if scope.panicked.did_panic() => panic!("scoped task panicked"),
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
    running: Arc<ScopeRunning>,
    panicked: Arc<ScopePanicked>,
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
            self.running.task_finished();

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
        self.running.task_created();

        let handle = ScopedJoinHandle {
            data: Arc::new(TaskResult {
                mutex: Mutex::new(None),
                condvar: Condvar::new(),
                scope_panicked: self.panicked.clone(),
            }),
            _scope: PhantomData,
        };

        let task_result = handle.data.clone();
        let scope_running = self.running.clone();
        let closure: Box<dyn FnOnce() + Send + 'scope> = Box::new(move || {
            task_result.store(catch_unwind(AssertUnwindSafe(f)));

            // If the JoinHandle has already been dropped, this will drop the TaskResult inside the
            // Arc, dropping the result and storing if an unhandled panic occurred.
            drop(task_result);

            // Mark the task as finished after all the borrows from the environment are dropped.
            scope_running.task_finished();
        });

        // SAFETY: The `scope` function ensures all closures are finished before returning
        let closure = unsafe {
            #[expect(clippy::unnecessary_cast, reason = "casting lifetimes")]
            Box::from_raw(Box::into_raw(closure) as *mut (dyn FnOnce() + Send + 'static))
        };

        (closure, handle)
    }
}

/// Stores the number of currently running tasks.
#[derive(Debug, Default)]
struct ScopeRunning {
    counter: AtomicUsize,
    wait_mutex: Mutex<()>,
    wait_condvar: Condvar,
}

impl ScopeRunning {
    fn task_created(&self) {
        self.counter.fetch_add(1, Ordering::AcqRel);
    }

    fn task_finished(&self) {
        let prev = self.counter.fetch_sub(1, Ordering::AcqRel);
        if prev == 1 {
            self.wait_condvar.notify_all();
        } else if prev == 0 {
            panic!("more tasks finished than started?")
        }
    }

    fn wait_for_tasks(&self) {
        let mut guard = self.wait_mutex.lock().unwrap();
        while self.counter.load(Ordering::Acquire) > 0 {
            guard = self.wait_condvar.wait(guard).unwrap();
        }
    }
}

/// Stores whether any of the tasks panicked.
#[derive(Debug, Default)]
struct ScopePanicked {
    value: AtomicBool,
}

impl ScopePanicked {
    fn store_panic(&self) {
        self.value.store(true, Ordering::Release);
    }

    fn did_panic(&self) -> bool {
        self.value.load(Ordering::Acquire)
    }
}

/// Stores the result of a task, ensuring the result is dropped safely and [`ScopePanicked`] is
/// updated.
#[derive(Debug)]
struct TaskResult<T> {
    mutex: Mutex<Option<Result<T, Box<dyn Any + Send + 'static>>>>,
    condvar: Condvar,
    scope_panicked: Arc<ScopePanicked>,
}

impl<T> TaskResult<T> {
    fn store(&self, result: Result<T, Box<dyn Any + Send + 'static>>) {
        let mut guard = self.mutex.lock().unwrap();
        *guard = Some(result);
        self.condvar.notify_all();
    }

    fn wait_and_take(&self) -> Result<T, Box<dyn Any + Send + 'static>> {
        let mut guard = self.mutex.lock().unwrap();
        loop {
            if let Some(result) = guard.take() {
                return result;
            }
            guard = self.condvar.wait(guard).unwrap();
        }
    }

    fn is_finished(&self) -> bool {
        self.mutex.lock().unwrap().is_some()
    }
}

impl<T> Drop for TaskResult<T> {
    #[expect(clippy::print_stderr)]
    fn drop(&mut self) {
        let Some(result) = self
            .mutex
            .get_mut()
            .expect("worker panicked while storing result")
            .take()
        else {
            return; // Result was already taken and handled
        };

        let panic;
        match result {
            Ok(v) => match catch_unwind(AssertUnwindSafe(|| drop(v))) {
                Ok(()) => return,
                Err(e) => panic = e,
            },
            Err(e) => panic = e,
        }

        if let Err(_panic) = catch_unwind(AssertUnwindSafe(|| drop(panic))) {
            eprintln!("panic while dropping scoped task panic");
            std::process::abort();
        }

        self.scope_panicked.store_panic();
    }
}

/// Handle to block on a task's termination.
///
/// Designed to match the [`std::thread::ScopedJoinHandle`] API, except
/// [`std::thread::ScopedJoinHandle::thread`] is not supported as tasks are not run on dedicated
/// threads.
#[derive(Debug)]
pub struct ScopedJoinHandle<'scope, T> {
    data: Arc<TaskResult<T>>,
    _scope: PhantomData<&'scope mut &'scope ()>,
}

impl<T> ScopedJoinHandle<'_, T> {
    /// Wait for the task to finish.
    ///
    /// The [`Err`] variant contains the panic value if the task panicked.
    pub fn join(self) -> Result<T, Box<dyn Any + Send + 'static>> {
        self.data.wait_and_take()
    }

    /// Check if the task is finished.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.data.is_finished()
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
