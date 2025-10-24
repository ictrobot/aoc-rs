//! Queue data structures.

use std::fmt::{Debug, Formatter};

/// A priority queue using a circular array of buckets for priorities within a sliding window.
///
/// New items must have a priority within `current_priority..current_priority + N`.
/// `current_priority` is only updated on pop and when the first item is pushed.
///
/// Items are popped in LIFO order within each priority bucket as [`Vec`] is used internally to
/// avoid the extra overhead of [`VecDeque`](std::collections::VecDeque).
///
/// # Example
/// ```
/// # use utils::queue::BucketQueue;
/// let mut queue = BucketQueue::<i32, 8>::new();
/// queue.push(0, 100);
/// queue.push(2, 200);
/// queue.push(7, 400);
/// queue.push(2, 300);
/// assert_eq!(queue.pop(), Some(100));
/// assert_eq!(queue.pop(), Some(300));
/// assert_eq!(queue.pop(), Some(200));
/// assert_eq!(queue.peek(), Some(&400));
/// assert_eq!(queue.peek_entry(), Some((7, &400)));
/// assert_eq!(queue.pop_entry(), Some((7, 400)));
/// assert_eq!(queue.pop(), None);
/// assert_eq!(queue.peek(), None);
/// ```
#[must_use]
#[derive(Clone)]
pub struct BucketQueue<T, const N: usize> {
    buckets: [Vec<T>; N],
    current_priority: usize,
    size: usize,
}

impl<T, const N: usize> BucketQueue<T, N> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn with_capacity(per_bucket_capacity: usize) -> Self {
        BucketQueue {
            buckets: std::array::from_fn(|_| Vec::with_capacity(per_bucket_capacity)),
            current_priority: 0,
            size: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, priority: usize, value: T) {
        if self.size == 0 {
            self.current_priority = priority;
        } else {
            assert!(
                priority >= self.current_priority && priority < self.current_priority + N,
                "priority {priority} out of range {}..{}",
                self.current_priority,
                self.current_priority + N,
            );
        }

        self.buckets[priority % N].push(value);
        self.size += 1;
    }

    #[inline]
    #[must_use]
    pub fn pop(&mut self) -> Option<T> {
        self.pop_entry().map(|(_, v)| v)
    }

    #[inline]
    #[must_use]
    pub fn pop_entry(&mut self) -> Option<(usize, T)> {
        if self.size == 0 {
            return None;
        }

        loop {
            let idx = self.current_priority % N;
            if let Some(value) = self.buckets[idx].pop() {
                self.size -= 1;
                return Some((self.current_priority, value));
            }
            self.current_priority += 1;
        }
    }

    #[inline]
    #[must_use]
    pub fn peek(&self) -> Option<&T> {
        self.peek_entry().map(|(_, v)| v)
    }

    #[inline]
    #[must_use]
    pub fn peek_entry(&self) -> Option<(usize, &T)> {
        if self.size == 0 {
            return None;
        }

        // peek takes &self so can't advance current_priority
        for priority in self.current_priority..self.current_priority + N {
            if let Some(value) = self.buckets[priority % N].last() {
                return Some((priority, value));
            }
        }
        unreachable!()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.buckets.iter_mut().for_each(Vec::clear);
        self.size = 0;
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.size
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

impl<T, const N: usize> Default for BucketQueue<T, N> {
    #[inline]
    fn default() -> Self {
        BucketQueue {
            buckets: std::array::from_fn(|_| Vec::new()),
            current_priority: 0,
            size: 0,
        }
    }
}

impl<T: Debug, const N: usize> Debug for BucketQueue<T, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(
                (self.current_priority..self.current_priority + N).flat_map(|priority| {
                    self.buckets[priority % N]
                        .iter()
                        .rev() // Match LIFO order within buckets
                        .map(move |value| (priority, value))
                }),
            )
            .finish()
    }
}
