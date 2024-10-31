/// Parse one or more string literals, mapping the results.
///
/// This is a replacement for
/// [`parser::one_of`](crate::parser::one_of())`(("a".map(|_| Enum::A), "b".map(|_| Enum::b)))`
/// which produces more optimized assembly and is easier to read and write.
///
/// The string patterns are matched in the order provided, so strings should be ordered by length.
///
/// Using this makes [2017 day 11](../../year2017/struct.Day11.html), which parses a sequence of
/// literals separated by commas, over 2x faster.
///
/// # Examples
/// ```
/// # use utils::parser::{Parser, self};
///
/// #[derive(Debug, PartialEq)]
/// enum Example {
///     A,
///     B,
///     C,
/// }
///
/// let parser = parser::literal_map!(
///     "A" | "a" => Example::A,
///     "B" => Example::B,
///     "C" => Example::C,
/// );
/// assert_eq!(parser.parse(b"A"), Ok((Example::A, &b""[..])));
/// assert_eq!(parser.parse(b"a"), Ok((Example::A, &b""[..])));
/// assert_eq!(parser.parse(b"B"), Ok((Example::B, &b""[..])));
/// assert_eq!(parser.parse(b"C"), Ok((Example::C, &b""[..])));
/// assert!(parser.parse(b"D").is_err());
/// ```
#[macro_export]
macro_rules! parser_literal_map {
    (
        $($($l:literal)|+ => $e:expr),+$(,)?
    ) => {{
        fn coerce_to_parser<F: Fn(&[u8]) -> $crate::parser::ParseResult<'_, O>, O>(f: F) -> F { f }

        coerce_to_parser(|input| {
            $($(
                if input.len() >= const { $l.len() } && const { $l.as_bytes() } == &input[..const { $l.len() }] {
                    return Ok((($e), &input[const { $l.len() }..]));
                }
            )+)*

            Err(($crate::parser_literal_map!(@error $($($l)+)+), input))
        })
    }};
    (@error $first:literal $($l:literal)+) => {
        $crate::parser::ParseError::Custom(concat!("expected one of '", $first, "'", $(", '", $l, "'",)+))
    };
    (@error $first:literal) => {
        $crate::parser::ParseError::ExpectedLiteral($first)
    };
}
