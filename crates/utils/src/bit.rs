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

/// Computes the population count for each bit position across 8 input masks.
///
/// Returns four masks `[bit0, bit1, bit2, bit3]` that encode, per bit position, the 4-bit count
/// of how many of the 8 inputs have that bit set.
///
/// For example, if a given bit is set in 5 inputs, then that bit would be set in `bit0` and `bit2`.
/// If a given bit is set in all 8 inputs, then that bit would only be set in `bit3`.
///
/// # Examples
/// ```
/// # use utils::bit::bitwise_count8;
/// let [bit0, bit1, bit2, bit3] = bitwise_count8::<u16>(&[
///                  0b100000000,
///                  0b110000000,
///                  0b111000000,
///                  0b111100000,
///                  0b111110000,
///                  0b111111000,
///                  0b111111100,
///                  0b111111110,
/// ]);
/// assert_eq!(bit0, 0b010101010);
/// assert_eq!(bit1, 0b011001100);
/// assert_eq!(bit2, 0b011110000);
/// assert_eq!(bit3, 0b100000000);
/// ```
///
/// ```
/// # use utils::bit::bitwise_count8;
/// let [bit0, bit1, bit2, bit3] = bitwise_count8::<u64>(&[
///                  0b00111000_01001000_10000111_11111111_10111000_11100010_00110010_01010011,
///                  0b01110000_00100111_01011101_11011000_10001100_00011000_10100101_00110010,
///                  0b00000101_11010011_10110011_10000000_00000000_11110110_00000101_11111010,
///                  0b01101001_01001100_00111001_01101100_00110111_00101011_00010101_10011101,
///                  0b00011100_11101111_00111111_01101011_00011101_01011110_11101101_10101101,
///                  0b10111100_11101111_00001001_10100100_01010110_10101011_01011000_11111100,
///                  0b11110110_00010001_11101111_00101101_01110111_10000011_11110110_10101011,
///                  0b01110001_01100010_01111101_01001000_01011001_10110100_11100110_11100000,
/// ]);
/// assert_eq!(bit0, 0b00000011_10000011_11110100_01100001_01100110_11100101_00100010_00011100);
/// assert_eq!(bit1, 0b10110001_11010000_11001000_00011011_11110010_01000111_00001110_10100100);
/// assert_eq!(bit2, 0b01111100_01101111_00111110_11101100_00011101_10111010_11110101_11111011);
/// assert_eq!(bit3, 0b00000000_00000000_00000001_00000000_00000000_00000000_00000000_00000000);
/// ```
#[inline]
#[must_use]
pub fn bitwise_count8<T: UnsignedInteger>(m: &[T; 8]) -> [T; 4] {
    let (s1, c1) = carry_save_adder(m[0], m[1], m[2]);
    let (s2, c2) = carry_save_adder(m[3], m[4], m[5]);
    let (s3, c3) = carry_save_adder(m[6], m[7], T::ZERO);
    let (s4, c4) = carry_save_adder(c1, c2, c3);
    let (bit0, c5) = carry_save_adder(s1, s2, s3);
    let (bit1, c6) = carry_save_adder(s4, c5, T::ZERO);
    let (bit2, bit3) = carry_save_adder(c4, c6, T::ZERO);
    [bit0, bit1, bit2, bit3]
}

#[inline]
#[must_use]
fn carry_save_adder<T: UnsignedInteger>(a: T, b: T, c: T) -> (T, T) {
    let sum_ab = a ^ b;
    let carry_ab = a & b;
    let sum_abc = sum_ab ^ c;
    let carry_abc = carry_ab | (sum_ab & c);
    (sum_abc, carry_abc)
}
