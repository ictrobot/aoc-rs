//! Simple multi-producer, multi-consumer FIFO channel using a mutex.
//! Replace with [`std::sync::mpmc`](https://github.com/rust-lang/rust/issues/126840) once
//! stabilized.
use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
};

#[derive(Debug)]
struct Shared<T> {
    mutex: Mutex<MutexInner<T>>,
    condvar: Condvar,
}

#[derive(Debug)]
struct MutexInner<T> {
    queue: VecDeque<T>,
    sender_count: usize,
    receiver_count: usize,
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new(Shared {
        mutex: Mutex::new(MutexInner {
            queue: VecDeque::new(),
            sender_count: 1,
            receiver_count: 1,
        }),
        condvar: Condvar::new(),
    });

    (Sender(inner.clone()), Receiver(inner))
}

#[derive(Debug)]
pub struct Sender<T>(Arc<Shared<T>>);

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), T> {
        let mut guard = self.0.mutex.lock().unwrap();
        if guard.receiver_count == 0 {
            return Err(value);
        }
        guard.queue.push_back(value);
        self.0.condvar.notify_one();
        Ok(())
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        {
            let mut guard = self.0.mutex.lock().unwrap();
            guard.sender_count = guard.sender_count.checked_add(1).unwrap();
        }
        Sender(self.0.clone())
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut guard = self.0.mutex.lock().unwrap();
        guard.sender_count = guard.sender_count.checked_sub(1).unwrap();
        if guard.sender_count == 0 {
            self.0.condvar.notify_all();
        }
    }
}

#[derive(Debug)]
pub struct Receiver<T>(Arc<Shared<T>>);

impl<T> Receiver<T> {
    pub fn recv(&self) -> Option<T> {
        let mut guard = self.0.mutex.lock().unwrap();
        loop {
            if let Some(v) = guard.queue.pop_front() {
                return Some(v);
            }
            if guard.sender_count == 0 {
                return None;
            }
            guard = self.0.condvar.wait(guard).unwrap();
        }
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        {
            let mut guard = self.0.mutex.lock().unwrap();
            guard.receiver_count = guard.receiver_count.checked_add(1).unwrap();
        }
        Receiver(self.0.clone())
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        let mut guard = self.0.mutex.lock().unwrap();
        guard.receiver_count = guard.receiver_count.checked_sub(1).unwrap();
        if guard.receiver_count == 0 {
            self.0.condvar.notify_all();
        }
    }
}
