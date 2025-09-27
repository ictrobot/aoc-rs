//! ASCII helpers.

use crate::bit::BitIterator;
use std::fmt::{Debug, Display, Formatter};

/// A set of ASCII characters.
///
/// # Examples
/// ```
/// # use utils::ascii::AsciiSet;
/// let set1 = AsciiSet::new((1 << b'A') | (1 << b'B') | (1 << b'C'));
/// assert_eq!(set1.len(), 3);
/// assert_eq!(set1.to_string(), "'A', 'B', 'C'");
/// assert_eq!(format!("{set1:?}"), "{'A', 'B', 'C'}");
///
/// let mut array = [false; 128];
/// array[b'A' as usize] = true;
/// array[b'B' as usize] = true;
/// array[b'C' as usize] = true;
/// assert_eq!(AsciiSet::from(array), set1);
///
/// assert_eq!(AsciiSet::from(|b| (b'A'..=b'C').contains(&b)), set1);
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Default)]
#[repr(transparent)]
#[must_use]
pub struct AsciiSet {
    set: u128,
}

impl AsciiSet {
    /// Creates a new `AsciiSet` from the specified bitset.
    pub const fn new(set: u128) -> Self {
        Self { set }
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.set == 0
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.set.count_ones() as usize
    }

    #[expect(clippy::cast_possible_truncation)]
    fn write_next_entry(&mut self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let b = self.set.trailing_zeros() as u8;
        let run = (self.set >> b).trailing_ones() as u8;

        let class_end = match b {
            b'0'..=b'9' => b'9',
            b'a'..=b'z' => b'z',
            b'A'..=b'Z' => b'Z',
            _ => b,
        };
        let range_len = (class_end - b + 1).min(run);

        if range_len >= 4 {
            self.set &= !(((1 << range_len) - 1) << b);
            write!(f, "{:?}-{:?}", b as char, (b + range_len - 1) as char)
        } else {
            self.set &= !(1 << b);
            write!(f, "{:?}", b as char)
        }
    }
}

/// Format the set for display, combining digit and letter ranges.
///
/// # Examples
///
/// ```
/// # use utils::ascii::AsciiSet;
/// assert_eq!(
///     AsciiSet::from(|b: u8| b.is_ascii_lowercase()).to_string(),
///     "'a'-'z'"
/// );
/// assert_eq!(
///     AsciiSet::from(|b: u8| b.is_ascii_alphabetic()).to_string(),
///     "'A'-'Z', 'a'-'z'"
/// );
/// assert_eq!(
///     AsciiSet::from(|b: u8| matches!(b, b'.' | b'#' | b'0'..=b'9')).to_string(),
///     "'#', '.', '0'-'9'"
/// );
/// assert_eq!(
///     AsciiSet::from(|b: u8| b.is_ascii_graphic()).to_string(),
///     concat!(
///         "'!', '\"', '#', '$', '%', '&', '\\'', '(', ')', '*', '+', ',', '-', '.', '/', ",
///         "'0'-'9', ':', ';', '<', '=', '>', '?', '@', 'A'-'Z', '[', '\\\\', ']', '^', '_', ",
///         "'`', 'a'-'z', '{', '|', '}', '~'"
///     )
/// );
/// ```
impl Display for AsciiSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return write!(f, "(empty)");
        }
        let mut set = *self;
        set.write_next_entry(f)?;
        while !set.is_empty() {
            write!(f, ", ")?;
            set.write_next_entry(f)?;
        }
        Ok(())
    }
}

impl Debug for AsciiSet {
    #[expect(clippy::cast_possible_truncation)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_set()
            .entries(BitIterator::ones(self.set).map(|(c, _)| c as u8 as char))
            .finish()
    }
}

impl From<u128> for AsciiSet {
    fn from(set: u128) -> Self {
        Self { set }
    }
}

impl From<[bool; 128]> for AsciiSet {
    fn from(value: [bool; 128]) -> Self {
        Self {
            set: value
                .iter()
                .enumerate()
                .fold(0, |s, (i, &b)| s | u128::from(b) << i),
        }
    }
}

impl<F: Fn(u8) -> bool> From<F> for AsciiSet {
    fn from(value: F) -> Self {
        Self {
            set: (0u8..=127).fold(0, |s, i| s | u128::from(value(i)) << i),
        }
    }
}
