//! String helpers.
use std::fmt::{Debug, Display};
use std::fmt::{Formatter, Write};
use std::hash::Hash;
use std::num::NonZero;

/// 2 byte [`TinyStr`] using a [`u16`].
pub type TinyStr2 = TinyStr<NonZero<u16>>;
/// 4 byte [`TinyStr`] using a [`u32`].
pub type TinyStr4 = TinyStr<NonZero<u32>>;
/// 8 byte [`TinyStr`] using a [`u64`].
pub type TinyStr8 = TinyStr<NonZero<u64>>;

/// A short string packed into a big-endian [`NonZero`] integer.
///
/// `TinyStr` stores up to `N` bytes in a single value that fits in a register, enabling
/// single-instruction equality and ordering comparisons.
/// The big-endian layout means normal integer comparisons result in lexicographic order.
///
/// Strings are NUL-padded. Trailing NUL bytes are indistinguishable from padding, so two inputs
/// only differing in trailing NULs will be equal.
///
/// Empty strings or strings containing all NUL bytes are not representable.
///
/// # Examples
/// ```
/// # use utils::str::{TinyStr4, TinyStr8};
/// let s4 = TinyStr4::new(b"abc").unwrap();
/// assert_eq!(s4.len(), 3);
/// assert_eq!(format!("{s4}"), "abc");
/// assert!(s4 < TinyStr4::from_const(b"abd"));
/// assert!(s4 > TinyStr4::from_const(b"abb"));
///
/// let s8 = TinyStr8::new(b"abcdefg").unwrap();
/// assert_eq!(s8.len(), 7);
/// assert_eq!(format!("{s8}"), "abcdefg");
/// assert_eq!(s8, const { TinyStr8::from_const(b"abcdefg") });
/// ```
#[must_use]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TinyStr<T: TinyStrInt>(T);

impl<T: TinyStrInt> TinyStr<T> {
    /// Creates a new `TinyStr` from a byte slice.
    ///
    /// Returns [`None`] if the slice is empty, only contains NUL bytes, or is too long.
    ///
    /// # Examples
    /// ```
    /// # use utils::str::TinyStr4;
    /// let s = TinyStr4::new(b"abc").unwrap();
    /// assert_eq!(s.len(), 3);
    /// assert_eq!(format!("{s}"), "abc");
    ///
    /// assert!(TinyStr4::new(b"").is_none());
    /// assert!(TinyStr4::new(b"abcde").is_none());
    /// ```
    pub fn new(s: &[u8]) -> Option<Self> {
        if s.is_empty() || s.len() > T::LEN {
            return None;
        }

        let mut accumulator = T::Raw::default();
        let mut i = 0;
        while i < T::LEN && i < s.len() {
            accumulator = T::set_raw(accumulator, s[i], i);
            i += 1;
        }

        T::from_raw(accumulator).map(Self)
    }

    /// Creates a new `TinyStr` from a raw [`NonZero`] integer.
    ///
    /// # Examples
    /// ```
    /// # use utils::str::TinyStr4;
    /// # use std::num::NonZero;
    /// let raw = NonZero::new(0x61626300).unwrap();
    /// let s = TinyStr4::from_raw(raw);
    /// assert_eq!(s.len(), 3);
    /// assert_eq!(format!("{s}"), "abc");
    /// ```
    #[inline]
    pub const fn from_raw(raw: T) -> Self {
        TinyStr(raw)
    }

    /// Returns the number of bytes in the string.
    ///
    /// # Examples
    /// ```
    /// # use utils::str::TinyStr4;
    /// assert_eq!(TinyStr4::new(b"a").unwrap().len(), 1);
    /// assert_eq!(TinyStr4::new(b"ab").unwrap().len(), 2);
    /// assert_eq!(TinyStr4::new(b"abc").unwrap().len(), 3);
    /// assert_eq!(TinyStr4::new(b"abcd").unwrap().len(), 4);
    /// ```
    #[inline]
    #[must_use]
    #[expect(clippy::len_without_is_empty, reason = "TinyStr is never empty")]
    pub fn len(self) -> usize {
        let len = T::LEN - (self.0.trailing_zeros() as usize) / 8;
        #[cfg(feature = "unsafe")]
        unsafe {
            std::hint::assert_unchecked(len >= 1);
            std::hint::assert_unchecked(len <= T::LEN);
        }
        len
    }
}

/// Writes the string, replacing any invalid UTF-8 sequences with the replacement character.
///
/// This is a comparatively expensive operation, requiring the value to be copied onto the stack
/// and UTF-8 validation.
///
/// # Examples
/// ```
/// # use utils::str::TinyStr4;
/// let s = TinyStr4::new(b"abc").unwrap();
/// assert_eq!(format!("{s}"), "abc");
///
/// let invalid = TinyStr4::from_raw(std::num::NonZero::new(0x61FF6200).unwrap());
/// assert_eq!(format!("{invalid}"), "a\u{FFFD}b");
/// ```
impl<T: TinyStrInt> Display for TinyStr<T> {
    #[cold]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let bytes = self.0.to_be_bytes();
        let slice = &bytes.as_ref()[..self.len()];
        for chunk in slice.utf8_chunks() {
            f.write_str(chunk.valid())?;
            if !chunk.invalid().is_empty() {
                f.write_char(char::REPLACEMENT_CHARACTER)?;
            }
        }
        Ok(())
    }
}

/// Writes a debug representation of the string.
///
/// # Examples
/// ```
/// # use utils::str::TinyStr4;
/// let s = TinyStr4::new(b"abc").unwrap();
/// assert_eq!(format!("{s:?}"), "TinyStr(\"abc\")");
///
/// let invalid = TinyStr4::from_raw(std::num::NonZero::new(0x61FF6200).unwrap());
/// assert_eq!(format!("{invalid:?}"), "TinyStr([97, 255, 98])");
/// ```
impl<T: TinyStrInt> Debug for TinyStr<T> {
    #[cold]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let bytes = self.0.to_be_bytes();
        let slice = &bytes.as_ref()[..self.len()];
        match std::str::from_utf8(slice) {
            Ok(s) => f.debug_tuple("TinyStr").field(&s).finish(),
            Err(_) => f.debug_tuple("TinyStr").field(&slice).finish(),
        }
    }
}

/// Helper trait for integer types that can be used as storage for a [`TinyStr`].
pub trait TinyStrInt: Copy + Clone + Eq + Ord + Hash + Sized {
    type Bytes: Copy + Default + AsMut<[u8]> + AsRef<[u8]>;
    type Raw: Copy + Default;
    const LEN: usize;

    fn to_be_bytes(self) -> Self::Bytes;
    fn trailing_zeros(self) -> u32;
    fn set_raw(accumulator: Self::Raw, byte: u8, pos: usize) -> Self::Raw;
    fn from_raw(raw: Self::Raw) -> Option<Self>;
}

macro_rules! int_impl {
    ($t:ident) => {
        impl TinyStrInt for NonZero<$t> {
            type Raw = $t;
            type Bytes = [u8; Self::LEN];
            const LEN: usize = $t::BITS as usize / 8;

            #[inline]
            fn to_be_bytes(self) -> Self::Bytes {
                self.get().to_be_bytes()
            }
            #[inline]
            fn trailing_zeros(self) -> u32 {
                self.get().trailing_zeros()
            }
            #[inline]
            fn set_raw(accumulator: Self::Raw, byte: u8, pos: usize) -> Self::Raw {
                accumulator | ($t::from(byte) << ((Self::LEN - 1 - pos) * 8))
            }
            #[inline]
            fn from_raw(raw: Self::Raw) -> Option<Self> {
                NonZero::new(raw)
            }
        }

        // Workaround for const trait limitations
        impl TinyStr<NonZero<$t>> {
            #[doc = concat!(
                "Creates a `TinyStr` from a byte slice at compile time, panicking if the string is invalid.\n",
                "\n",
                "# Examples\n",
                "```\n",
                "# use utils::str::TinyStr;\n",
                "# use std::num::NonZero;\n",
                "const S: TinyStr<NonZero<", stringify!($t), ">> = TinyStr::<NonZero<", stringify!($t), ">>::from_const(b\"ab\");\n",
                "assert_eq!(S, TinyStr::new(b\"ab\").unwrap());\n",
                "assert_eq!(S.to_string(), \"ab\");\n",
                "assert_eq!(S.len(), 2);\n",
                "```"
            )]
            pub const fn from_const(s: &[u8]) -> Self {
                const LEN: usize = $t::BITS as usize / 8;

                assert!(!s.is_empty(), "string is empty");
                assert!(s.len() <= LEN, "string is too long");

                let mut accumulator: $t = 0;
                let mut i = 0;
                while i < s.len() {
                    accumulator |= (s[i] as $t) << ((LEN - 1 - i) * 8);
                    i += 1;
                }

                match NonZero::new(accumulator) {
                    Some(v) => Self::from_raw(v),
                    None => panic!("string only contains zero bytes"),
                }
            }
        }
    };
}
int_impl!(u16);
int_impl!(u32);
int_impl!(u64);

/// Helper trait to map between length `N` and the smallest [`TinyStrInt`] type that can store it.
///
/// See [`parser::tinystr`](crate::parser::tinystr).
pub trait TinyStrLen<const N: usize> {
    type Int: TinyStrInt;
}
impl TinyStrLen<2> for () {
    type Int = NonZero<u16>;
}
impl TinyStrLen<3> for () {
    type Int = NonZero<u32>;
}
impl TinyStrLen<4> for () {
    type Int = NonZero<u32>;
}
impl TinyStrLen<5> for () {
    type Int = NonZero<u64>;
}
impl TinyStrLen<6> for () {
    type Int = NonZero<u64>;
}
impl TinyStrLen<7> for () {
    type Int = NonZero<u64>;
}
impl TinyStrLen<8> for () {
    type Int = NonZero<u64>;
}
