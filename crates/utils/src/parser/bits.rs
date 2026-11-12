use crate::number::UnsignedInteger;
use crate::parser::{Leaf, LeafResult, ParseError};
use std::marker::PhantomData;

#[derive(Copy, Clone)]
pub struct BitmaskParser<U: UnsignedInteger> {
    bytes: &'static [u8; 2],
    _phantom: PhantomData<U>,
}
impl<'i, U: UnsignedInteger> Leaf<'i> for BitmaskParser<U> {
    type Output = (U, u32);

    #[inline]
    fn parse(&self, input: &'i [u8]) -> LeafResult<'i, Self::Output> {
        let [zero, one] = *self.bytes;
        let (mut value, mut len) = (U::ZERO, 0);

        // Bits are inserted at the top so every shift is constant
        while len + 8 <= U::BITS
            && let Some(&chunk) = input[len as usize..].first_chunk()
            && let Some(packed) = pack(chunk, zero, one)
        {
            // Shift twice as u8 can't be shifted by 8 at once
            value = (value >> 4 >> 4) | (U::from(packed) << (U::BITS - 8));
            len += 8;
        }
        while let Some(&byte) = input.get(len as usize)
            && (byte == zero || byte == one)
        {
            if len == U::BITS {
                return Err((
                    ParseError::ExpectedLessItems(U::BITS as usize),
                    &input[len as usize..],
                ));
            }
            value = (value >> 1) | (U::from(byte == one) << (U::BITS - 1));
            len += 1;
        }

        if len == 0 {
            return Err((ParseError::ExpectedOneOfBytes(self.bytes), input));
        }

        Ok(((value >> (U::BITS - len), len), &input[len as usize..]))
    }
}

#[inline]
fn pack(chunk: [u8; 8], zero: u8, one: u8) -> Option<u8> {
    const LOW_BITS: u64 = 0x0101_0101_0101_0101;
    let xor = zero ^ one;

    // XORing with `zero` turns valid bytes into either 0 or `zero ^ one`
    let difference = u64::from_le_bytes(chunk) ^ (u64::from(zero) * LOW_BITS);
    let ones = (difference >> xor.trailing_zeros()) & LOW_BITS;
    if difference != ones * u64::from(xor) {
        return None;
    }

    // Multiply the low bits of each byte into a single u8
    Some((ones.wrapping_mul(0x0102_0408_1020_4080) >> 56) as u8)
}

/// [`Leaf`] parser for strings made of two different bytes, packed into a bitmask.
///
/// Bytes in the input matching the first provided byte are mapped to 0 bits, and bytes matching the
/// second provided byte are mapped to 1 bits.
///
/// The first input byte is put in the least significant bit. Returns the bitmask and its length, or
/// an error if the string is empty or too long for the integer type.
///
/// # Examples
/// ```
/// # use utils::parser::{self, Leaf};
/// let parser = parser::bitmask::<u8>(b".#");
/// assert_eq!(
///     parser.parse(b"#.###. abc"),
///     Ok(((0b011101, 6), &b" abc"[..]))
/// );
/// assert_eq!(
///     parser.parse(b".##...##"),
///     Ok(((0b11000110, 8), &b""[..]))
/// );
/// assert!(parser.parse(b"abc").is_err());
/// assert!(parser.parse(b"#########").is_err());
///
/// let parser = parser::bitmask::<u64>(b".#");
/// assert_eq!(
///     parser.parse(b".##...####.....######.......######## 123"),
///     Ok(((0b111111110000000111111000001111000110, 36), &b" 123"[..]))
/// );
/// ```
#[inline]
#[must_use]
pub fn bitmask<U: UnsignedInteger>(bytes: &'static [u8; 2]) -> BitmaskParser<U> {
    BitmaskParser {
        bytes,
        _phantom: PhantomData,
    }
}
