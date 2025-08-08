//! Items relating to puzzle input.

use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Enum for distinguishing between example and real inputs.
///
/// Some puzzles require this as different constants may be used for example inputs to simplify the
/// problem. For example [2022 day 15](https://adventofcode.com/2022/day/15) part 1, which uses
/// `y=10` in the example, but `y=2000000` for real inputs.
///
/// Most puzzle solutions should ignore this value.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum InputType {
    Example,
    Real,
}

/// Error type that shows the error's location in the input, returned by puzzle `new` functions.
///
/// # Examples
///
/// ```
/// # use utils::input::InputError;
/// let input = "12 34\n56 78\n90 abc";
/// let error = InputError::new(input, 15, "expected number");
/// assert_eq!(error.to_string(), "
/// invalid input: expected number
///   --> line 3 column 4
///   |
/// 3 | 90 abc
///   |    ^
/// ".trim_start());
/// ```
#[derive(Debug)]
pub struct InputError {
    line_number: usize,
    column_number: usize,
    line: String,
    source: Box<dyn Error>,
}

impl InputError {
    /// Create a new [`InputError`].
    ///
    /// See [`ToIndex`] implementations for details on supported indexes.
    #[cold]
    pub fn new(input: &str, index: impl ToIndex, source: impl Into<Box<dyn Error>>) -> Self {
        let index = index.input_index(input);
        let (line_number, column_number, line) = Self::line_position(input, index);
        let line = line.replace('\t', " ");

        InputError {
            line_number,
            column_number,
            line,
            source: source.into(),
        }
    }

    #[cold]
    fn line_position(input: &str, index: usize) -> (usize, usize, String) {
        let start = input[..index].rfind('\n').map_or(0, |p| p + 1);
        let end = input[start..].find('\n').map_or(input.len(), |p| p + start);
        let line = input[start..end].trim_end_matches('\r');

        let line_number = input[..start].matches('\n').count() + 1;
        let column_number = index - start + 1;

        (line_number, column_number, line.to_string())
    }

    /// Returns the source error.
    #[must_use]
    pub fn into_source(self) -> Box<dyn Error> {
        self.source
    }
}

impl Display for InputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let pad = " ".repeat(self.line_number.to_string().len());

        write!(
            f,
            "invalid input: {}\n  --> line {} column {}\n{pad} |\n{} | {}\n{pad} |{}^\n",
            self.source,
            self.line_number,
            self.column_number,
            self.line_number,
            self.line,
            " ".repeat(self.column_number),
        )
    }
}

impl Error for InputError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.source)
    }
}

/// Helper trait to simplify error location tracking.
///
/// Used in [`InputError::new`].
pub trait ToIndex {
    fn input_index(self, input: &str) -> usize;
}

impl ToIndex for &str {
    /// Find index of this substring in the provided input.
    ///
    /// Uses the pointer offset, meaning it works if this substring is not the first occurrence in
    /// the string. This allows recovering the error position without tracking an offset into the
    /// string, which is useful when using [`Iterator`]s such as [`str::lines`] on an input.
    ///
    /// # Panics
    ///
    /// This function panics if this string is not a substring inside the provided string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use utils::input::ToIndex;
    /// let string = "abcabc";
    /// assert_eq!(string[4..].input_index(string), 4);
    /// ```
    ///
    /// ```should_panic
    /// # use utils::input::ToIndex;
    /// let string = "abcabc";
    /// let mut other = String::new();
    /// other.push('b');
    /// other.push('c');
    /// other.input_index(string);
    /// ```
    fn input_index(self, input: &str) -> usize {
        self.as_bytes().input_index(input)
    }
}

impl ToIndex for &[u8] {
    /// Find index of this subslice in the provided input.
    ///
    /// For use with functions that iterate over a string's bytes.
    /// See the [`&str`](#impl-ToIndex-for-%26str) implementation.
    fn input_index(self, input: &str) -> usize {
        let self_ptr = self.as_ptr() as usize;
        let input_ptr = input.as_ptr() as usize;
        match self_ptr.checked_sub(input_ptr) {
            Some(offset) if offset + self.len() <= input.len() => offset,
            _ => panic!("invalid string index: {self_ptr:#x} is not a substring of {input_ptr:#x}"),
        }
    }
}

impl ToIndex for char {
    /// Find the first instance of this character in the string.
    ///
    /// Intended for puzzles where the entire input should be a certain set of characters, so
    /// if an invalid character is found, the instance in the error doesn't matter.
    ///
    /// # Panics
    ///
    /// This function panics if this character is not present in the string
    ///
    /// # Examples
    ///
    /// ```
    /// # use utils::input::ToIndex;
    /// let string = "abca bc";
    /// assert_eq!(' '.input_index(string), 4);
    /// ```
    ///
    /// ```should_panic
    /// # use utils::input::ToIndex;
    /// let string = "abcdef";
    /// ' '.input_index(string);
    /// ```
    fn input_index(self, input: &str) -> usize {
        input
            .find(self)
            .unwrap_or_else(|| panic!("invalid string index: char {self:?} not found in {input:?}"))
    }
}

impl ToIndex for usize {
    /// Index into the input string.
    ///
    /// # Panics
    ///
    /// This function panics if the index is out of range for the provided string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use utils::input::ToIndex;
    /// let string = "abcdef";
    /// assert_eq!(4.input_index(string), 4);
    /// ```
    ///
    /// ```should_panic
    /// # use utils::input::ToIndex;
    /// let string = "abcdef";
    /// 10.input_index(string);
    /// ```
    fn input_index(self, input: &str) -> usize {
        assert!(
            self <= input.len(),
            "invalid string index: index {self} out of range"
        );
        self
    }
}

/// Extension trait to simplify converting errors and locations into [`InputError`]s.
///
/// Note that constructing [`InputError`] is expensive, and therefore conversion should be done as
/// late as possible, to avoid unnecessary work if the error is discarded (for example,
/// by [`Parser::or`](crate::parser::Parser::or)).
pub trait MapWithInputExt {
    type Output;
    fn map_with_input(self, input: &str) -> Self::Output;
}

impl<E: Into<Box<dyn Error>>, I: ToIndex> MapWithInputExt for (E, I) {
    type Output = InputError;

    #[cold]
    fn map_with_input(self, input: &str) -> Self::Output {
        InputError::new(input, self.1, self.0)
    }
}

impl<T, E: Into<Box<dyn Error>>, I: ToIndex> MapWithInputExt for Result<T, (E, I)> {
    type Output = Result<T, InputError>;

    #[inline]
    fn map_with_input(self, input: &str) -> Self::Output {
        self.map_err(|err| err.map_with_input(input))
    }
}

/// Strips the final newline from a borrowed string.
///
/// Equivalent to `s.strip_suffix("\r\n").or_else(|| s.strip_suffix("\n")).unwrap_or(s)`.
///
/// # Examples
/// ```
/// # use utils::input::strip_final_newline;
/// assert_eq!(
///     strip_final_newline("abc\ndef\n"),
///     "abc\ndef"
/// );
/// assert_eq!(
///     strip_final_newline("12\r\n34\r\n\r\n"),
///     "12\r\n34\r\n"
/// );
/// ```
#[must_use]
#[inline]
pub const fn strip_final_newline(s: &str) -> &str {
    match s.as_bytes() {
        // Use split_at as string slicing isn't const
        [.., b'\r', b'\n'] => s.split_at(s.len() - 2).0,
        [.., b'\n'] => s.split_at(s.len() - 1).0,
        _ => s,
    }
}

/// Convert a string to both LF and CRLF if it contains a newline.
///
/// # Examples
/// ```
/// # use utils::input::to_lf_crlf;
/// assert_eq!(
///     to_lf_crlf("abc\ndef\nghi"),
///     ("abc\ndef\nghi".into(), Some("abc\r\ndef\r\nghi".into()))
/// );
/// assert_eq!(
///     to_lf_crlf("12\r\n34\r\n56\r\n78"),
///     ("12\n34\n56\n78".into(), Some("12\r\n34\r\n56\r\n78".into()))
/// );
/// assert_eq!(
///     to_lf_crlf("abc123"),
///     ("abc123".into(), None),
/// );
/// ```
#[must_use]
pub fn to_lf_crlf(s: &str) -> (Cow<'_, str>, Option<Cow<'_, str>>) {
    let (mut has_lf, mut has_crlf) = (false, false);
    let mut prev = 0;
    for b in s.bytes() {
        has_lf |= b == b'\n' && prev != b'\r';
        has_crlf |= b == b'\n' && prev == b'\r';
        prev = b;
    }
    if !has_lf && !has_crlf {
        return (Cow::Borrowed(s), None);
    }

    let lf = if has_crlf {
        Cow::Owned(s.replace("\r\n", "\n"))
    } else {
        Cow::Borrowed(s)
    };
    let crlf = if has_lf {
        Cow::Owned(lf.replace('\n', "\r\n"))
    } else {
        Cow::Borrowed(s)
    };
    (lf, Some(crlf))
}
