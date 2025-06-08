//! Simple one-shot channel using a mutex.
use std::sync::{Arc, Condvar, Mutex};

#[derive(Debug)]
struct Shared<T> {
    value: Mutex<MutexInner<T>>,
    condvar: Condvar,
}

#[derive(Debug)]
struct MutexInner<T> {
    value: Option<T>,
    closed: bool,
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new(Shared {
        value: Mutex::new(MutexInner {
            value: None,
            closed: false,
        }),
        condvar: Condvar::new(),
    });
    (Sender(Some(inner.clone())), Receiver(Some(inner)))
}

#[derive(Debug)]
pub struct Sender<T>(Option<Arc<Shared<T>>>);

impl<T> Sender<T> {
    pub fn send(mut self, value: T) -> Result<(), T> {
        let shared = self.0.take().expect("sender not yet consumed or dropped");
        let mut guard = shared.value.lock().unwrap();
        if guard.closed {
            return Err(value);
        }
        guard.value = Some(value);
        shared.condvar.notify_one();
        Ok(())
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        if let Some(shared) = self.0.take() {
            let mut guard = shared.value.lock().unwrap();
            guard.closed = true;
            shared.condvar.notify_one();
        }
    }
}

#[derive(Debug)]
pub struct Receiver<T>(Option<Arc<Shared<T>>>);

impl<T> Receiver<T> {
    pub fn recv(mut self) -> Option<T> {
        let shared = self.0.take().expect("receiver not yet consumed or dropped");
        let mut guard = shared.value.lock().unwrap();
        loop {
            if let Some(value) = guard.value.take() {
                return Some(value);
            }
            if guard.closed {
                return None;
            }
            guard = shared.condvar.wait(guard).unwrap();
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        if let Some(shared) = self.0.take() {
            let mut guard = shared.value.lock().unwrap();
            guard.closed = true;
            shared.condvar.notify_one();
        }
    }
}
