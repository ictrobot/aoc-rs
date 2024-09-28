//! Experimental drop-in replacement for [`std::thread::scope`] for WebAssembly.
//!
//! Uses a pool of web worker threads spawned by the host JS environment to run scoped tasks.
//!
//! Requires the `atomics`, `bulk-memory` and `mutable-globals` target features to be enabled, and
//! for all threads to be using web workers as `memory.atomic.wait` doesn't work on the main thread.
//!
//! Catching unwinding panics should be supported, but at the time of writing, the Rust standard
//! library doesn't support panic=unwind on WebAssembly.
//!
//! # Examples
//!
//! ```
//! // Setup pool of workers. In WebAssembly, this would be done by spawning more web workers which
//! // then call the exported worker function.
//! for _ in 0..std::thread::available_parallelism().map_or(4, NonZero::get) {
//!     std::thread::spawn(scoped::worker);
//! }
//!
//! let data = vec![1, 2, 3];
//! let mut data2 = vec![10, 100, 1000];
//!
//! // Start scoped tasks which may run on other threads
//! scoped::scope(|s| {
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
//! scoped::scope(|s| {
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
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::mpsc::{SyncSender, TrySendError};
use std::sync::{Arc, Condvar, Mutex};

#[cfg(not(all(
    target_feature = "atomics",
    target_feature = "bulk-memory",
    target_feature = "mutable-globals",
)))]
compile_error!("Required target features not enabled");

/// Create a scope for spawning scoped tasks.
///
/// Scoped tasks may borrow non-`static` data, and may run in parallel depending on thread pool
/// worker availability.
#[inline(never)]
pub fn scope<'env, F, T>(f: F) -> T
where
    F: for<'scope> FnOnce(&'scope Scope<'scope, 'env>) -> T,
{
    let scope = Scope {
        data: Arc::new(ScopeData {
            mutex: Mutex::new((0, false)),
            condvar: Condvar::new(),
        }),
        _scope: PhantomData,
        _env: PhantomData,
    };

    let result = catch_unwind(AssertUnwindSafe(|| f(&scope)));

    // Wait for tasks to finish
    let mut guard = scope.data.mutex.lock().unwrap();
    while guard.0 > 0 {
        guard = scope.data.condvar.wait(guard).unwrap();
    }

    match result {
        Err(e) => resume_unwind(e),
        Ok(_) if guard.1 => panic!("scoped task panicked"),
        Ok(x) => x,
    }
}

/// Scope to spawn tasks in.
///
/// # Lifetimes
///
/// The `'scope` lifetime represents the lifetime of the scope itself, starting when the closure
/// passed to [`scope`] is run and ending when all the tasks are joined.
///
/// The `'env` lifetime represents the lifetime of the data borrowed by the scoped tasks, and must
/// outlive `'scope`.
#[derive(Debug)]
pub struct Scope<'scope, 'env: 'scope> {
    data: Arc<ScopeData>,
    // &'scope mut &'scope is needed to prevent lifetimes from shrinking
    _scope: PhantomData<&'scope mut &'scope ()>,
    _env: PhantomData<&'env mut &'env ()>,
}

impl<'scope, 'env> Scope<'scope, 'env> {
    /// Spawn a new task within the scope.
    ///
    /// If no workers within the thread pool are available, the task will be executed on the current
    /// thread.
    pub fn spawn<F, T>(&'scope self, f: F) -> ScopedJoinHandle<'scope, T>
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
            _scope: PhantomData,
        };

        let handle_data = handle.data.clone();
        let closure: Box<dyn FnOnce() -> bool + Send + 'scope> = Box::new(
            #[inline(never)]
            move || {
                let result = catch_unwind(AssertUnwindSafe(f));
                let panicked = result.is_err();

                // Send the result to ScopedJoinHandle and wake any blocked threads
                let HandleData { mutex, condvar } = handle_data.as_ref();
                let mut guard = mutex.lock().unwrap();
                *guard = Some(result);
                condvar.notify_all();

                panicked
            },
        );

        // SAFETY: The `scope` function ensures all closures are finished before returning
        let closure = unsafe {
            #[expect(clippy::unnecessary_cast, reason = "casting lifetimes")]
            Box::from_raw(Box::into_raw(closure) as *mut (dyn FnOnce() -> bool + Send + 'static))
        };

        let scope_data = self.data.clone();
        scoped_task(Box::new(
            #[inline(never)]
            move || {
                // Use a second closure to ensure that the closure which borrows from 'scope is
                // dropped before `ScopeData::task_end` is called. This prevents `scope` from
                // returning too soon, while the closures still exist, which causes UB as detected
                // by Miri.
                let panicked = closure();
                dbg!("finished");
                scope_data.task_end(panicked);
            },
        ));

        handle
    }
}

// Stores the number of currently running tasks, and if a panic occurred.
#[derive(Debug)]
struct ScopeData {
    mutex: Mutex<(usize, bool)>,
    condvar: Condvar,
}

impl ScopeData {
    fn task_start(&self) {
        let mut guard = self.mutex.lock().unwrap();
        if let Some(new_running) = guard.0.checked_add(1) {
            guard.0 = new_running;
        } else {
            panic!("too many running tasks in scope");
        }
    }

    fn task_end(&self, panicked: bool) {
        let mut guard = self.mutex.lock().unwrap();
        guard.1 |= panicked;
        if let Some(new_running) = guard.0.checked_sub(1) {
            guard.0 = new_running;
            if new_running == 0 {
                self.condvar.notify_all();
            }
        } else {
            panic!("more tasks finished than started?")
        }
    }
}

/// Handle to block on a task's termination.
#[derive(Debug)]
pub struct ScopedJoinHandle<'scope, T> {
    data: Arc<HandleData<T>>,
    _scope: PhantomData<&'scope mut &'scope ()>,
}

#[derive(Debug)]
struct HandleData<T> {
    mutex: Mutex<Option<Result<T, Box<dyn Any + Send + 'static>>>>,
    condvar: Condvar,
}

impl<'scope, T> ScopedJoinHandle<'scope, T> {
    // Unsupported
    // pub fn thread(&self) -> &Thread {}

    /// Wait for the task to finish.
    pub fn join(self) -> Result<T, Box<dyn Any + Send + 'static>> {
        let HandleData { mutex, condvar } = self.data.as_ref();
        let mut guard = mutex.lock().unwrap();
        while guard.is_none() {
            guard = condvar.wait(guard).unwrap();
        }
        guard.take().unwrap()
    }

    /// Check if the task is finished.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.data.mutex.lock().unwrap().is_some()
    }
}

#[expect(clippy::type_complexity)]
static WORKERS: Mutex<VecDeque<SyncSender<Box<dyn FnOnce() + Send>>>> = Mutex::new(VecDeque::new());

fn scoped_task(mut closure: Box<dyn FnOnce() + Send>) {
    let mut guard = WORKERS.lock().unwrap();
    let queue = &mut *guard;

    for _ in 0..queue.len() {
        let Some(sender) = queue.pop_front() else {
            break;
        };

        match sender.try_send(closure) {
            Ok(()) => {
                queue.push_back(sender);
                return;
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

    // Fall back to run the closure on this thread
    closure();
}

/// Use this thread as a worker in the thread pool for scoped tasks.
pub fn worker() {
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
