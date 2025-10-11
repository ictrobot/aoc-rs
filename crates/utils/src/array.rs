//! Array helpers.

use std::fmt;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

/// A fixed-size array-backed vector.
///
/// `ArrayVec` has a fixed-sized `N` long array and a length field to track how many elements are
/// populated. It is useful for storing a small but variable number of elements without heap
/// allocation.
///
/// The implementation is intentionally simple, and requires [`Copy`] and [`Default`] bounds on some
/// methods instead of using [`std::mem::MaybeUninit`].
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ArrayVec<T, const N: usize> {
    len: usize,
    data: [T; N],
}

impl<T, const N: usize> ArrayVec<T, N> {
    /// Creates a new empty `ArrayVec`.
    ///
    /// # Examples
    /// ```
    /// # use utils::array::ArrayVec;
    /// let vec: ArrayVec<i32, 5> = ArrayVec::new();
    /// assert_eq!(vec.len(), 0);
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self
    where
        T: Copy + Default,
    {
        Self {
            len: 0,
            data: [T::default(); N],
        }
    }

    /// Adds an element to the end of the vector.
    ///
    /// Returns [`Err`] containing the provided value if the vector is already full.
    ///
    /// # Examples
    /// ```
    /// # use utils::array::ArrayVec;
    /// let mut vec: ArrayVec<i32, 2> = ArrayVec::new();
    /// assert_eq!(vec.push(1), Ok(()));
    /// assert_eq!(vec.push(2), Ok(()));
    /// assert_eq!(vec.push(3), Err(3)); // Vector is full
    /// ```
    #[inline]
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.len < N {
            self.data[self.len] = value;
            self.len += 1;
            Ok(())
        } else {
            Err(value)
        }
    }

    /// Removes the last element from the vector and returns it, or [`None`] if it is empty.
    ///
    /// # Examples
    /// ```
    /// # use utils::array::ArrayVec;
    /// let mut vec: ArrayVec<i32, 3> = ArrayVec::new();
    /// vec.push(1).unwrap();
    /// assert_eq!(vec.pop(), Some(1));
    /// assert_eq!(vec.pop(), None);
    /// ```
    #[inline]
    pub fn pop(&mut self) -> Option<T>
    where
        T: Default,
    {
        if self.len > 0 {
            self.len -= 1;
            Some(std::mem::take(&mut self.data[self.len]))
        } else {
            None
        }
    }

    /// Returns a slice of all the populated elements in the vector.
    ///
    /// # Examples
    ///
    /// ```
    /// # use utils::array::ArrayVec;
    /// let mut vec: ArrayVec<i32, 3> = ArrayVec::new();
    /// vec.push(1).unwrap();
    /// vec.push(2).unwrap();
    /// assert_eq!(vec.as_slice(), &[1, 2]);
    /// ```
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        #[cfg(feature = "unsafe")]
        unsafe {
            std::hint::assert_unchecked(self.len <= N);
        }

        &self.data[..self.len]
    }

    /// Returns a mutable slice of all the populated elements in the vector.
    ///
    /// # Examples
    ///
    /// ```
    /// # use utils::array::ArrayVec;
    /// let mut vec: ArrayVec<i32, 3> = ArrayVec::new();
    /// vec.push(1).unwrap();
    /// vec.push(2).unwrap();
    /// let mut slice = vec.as_mut_slice();
    /// slice[1] = 10;
    /// assert_eq!(slice, &[1, 10]);
    /// ```
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        #[cfg(feature = "unsafe")]
        unsafe {
            std::hint::assert_unchecked(self.len <= N);
        }

        &mut self.data[..self.len]
    }

    /// Returns the capacity of the vector, which is always `N`.
    ///
    /// # Examples
    /// ```
    /// # use utils::array::ArrayVec;
    /// let vec: ArrayVec<i32, 5> = ArrayVec::new();
    /// assert_eq!(vec.capacity(), 5);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        N
    }

    /// Returns whether the vector is full.
    ///
    /// # Examples
    ///
    /// ```
    /// # use utils::array::ArrayVec;
    /// let mut vec: ArrayVec<i32, 2> = ArrayVec::new();
    /// assert!(!vec.is_full());
    /// vec.push(1).unwrap();
    /// assert!(!vec.is_full());
    /// vec.push(2).unwrap();
    /// assert!(vec.is_full());
    /// ```
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len == N
    }

    /// Returns the backing array.
    ///
    /// Any items after the current length will be set to the default value.
    ///
    /// # Examples
    /// ```
    /// # use utils::array::ArrayVec;
    /// let mut vec: ArrayVec<i32, 5> = ArrayVec::new();
    /// vec.push(1).unwrap();
    /// vec.push(2).unwrap();
    /// vec.push(3).unwrap();
    /// vec.pop().unwrap();
    /// assert_eq!(vec.into_array(), [1, 2, 0, 0, 0]);
    /// ```
    #[inline]
    pub fn into_array(self) -> [T; N] {
        self.data
    }
}

impl<T, const N: usize> Deref for ArrayVec<T, N> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const N: usize> DerefMut for ArrayVec<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a ArrayVec<T, N> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut ArrayVec<T, N> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T: fmt::Debug, const N: usize> fmt::Debug for ArrayVec<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArrayVec")
            .field("len", &self.len)
            .field("cap", &N)
            .field("data", &self.as_slice())
            .finish()
    }
}

impl<T: Default + Copy, const N: usize> Default for ArrayVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> From<[T; N]> for ArrayVec<T, N> {
    fn from(data: [T; N]) -> Self {
        ArrayVec { len: N, data }
    }
}
