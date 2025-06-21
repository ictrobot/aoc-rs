//! Bit manipulation helpers.

use crate::number::UnsignedInteger;

/// Iterator which yields all the set or unset bits in a provided number.
pub struct BitIterator<T: UnsignedInteger> {
    n: T,
}

impl<T: UnsignedInteger> BitIterator<T> {
    /// Returns an iterator that yields the positions and values of all the set bits in the provided
    /// number.
    ///
    /// # Examples
    /// ```
    /// # use utils::bit::BitIterator;
    /// assert_eq!(
    ///     BitIterator::ones(0b1001_1101u8).collect::<Vec<(u32, u8)>>(),
    ///     vec![
    ///         (0, 1),
    ///         (2, 4),
    ///         (3, 8),
    ///         (4, 16),
    ///         (7, 128),
    ///     ],
    /// );
    /// ```
    pub fn ones(n: T) -> Self {
        BitIterator { n }
    }

    /// Returns an iterator that yields the positions and values of all the unset bits in the
    /// provided number.
    ///
    /// # Examples
    /// ```
    /// # use utils::bit::BitIterator;
    /// assert_eq!(
    ///     BitIterator::zeroes(0b1001_1101u8).collect::<Vec<(u32, u8)>>(),
    ///     vec![
    ///         (1, 2),
    ///         (5, 32),
    ///         (6, 64),
    ///     ],
    /// );
    /// ```
    pub fn zeroes(n: T) -> Self {
        BitIterator { n: !n }
    }
}

impl<T: UnsignedInteger> Iterator for BitIterator<T> {
    type Item = (u32, T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.n == T::ZERO {
            None
        } else {
            let position = self.n.trailing_zeros();
            let value = T::ONE << position;
            self.n &= !value;
            Some((position, value))
        }
    }
}
