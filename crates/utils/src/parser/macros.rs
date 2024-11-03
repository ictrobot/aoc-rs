/// Macro to define a parser for one or more string literals, mapping the results.
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

/// Macro to define a custom parser using a `match` inspired parse tree syntax.
///
/// Each rule is made up of a list of chained parsers enclosed in brackets on the left-hand side.
/// Parsers can be prefixed with an identifier followed by `@` to store the result of that parser in
/// the supplied variable, similar to normal match patterns.
///
/// After the list of parsers, there is an arrow determining the functionality of the rule when the
/// parsers match:
/// - **Expression (`=>`)**: The expression on the right-hand is evaluated and returned.
/// - **Fallible (`=?>`)**: Similar to Expression, but the right-hand side evaluates a result. If
///         the expression evaluates to [`Ok`], the value contained inside is returned. Otherwise,
///         the string contained inside the [`Err`] is handled as a custom
///         [`ParseError`](super::ParseError), and parsing will continue with the following rule.
/// - **Subtree (`=>>`)**: The right-hand side is a nested set of rules enclosed in braces.
///
/// If none of the rules match successfully, the error from the rule which parsed furthest into
/// the input is returned.
///
/// # Examples
/// ```
/// # use utils::parser::{self, Parser};
/// #
/// #[derive(Debug, PartialEq)]
/// enum Register {
///     A, B, C
/// }
///
/// #[derive(Debug, PartialEq)]
/// enum Instruction {
///     Add(Register, Register),
///     AddConstant(Register, i32),
///     Copy(Register, Register),
///     Noop,
/// }
///
/// let register = parser::literal_map!(
///     "A" => Register::A, "B" => Register::B, "C" => Register::C,
/// );
///
/// let instruction = parser::parse_tree!(
///     ("add ", r @ register, ", ") =>> {
///         (r2 @ register) => Instruction::Add(r, r2),
///         (v @ parser::i32()) => Instruction::AddConstant(r, v),
///     },
///     ("copy ", r @ register, ", ", r2 @ register) =?> {
///         if r == r2 {
///             Err("cannot copy register to itself")
///         } else {
///             Ok(Instruction::Copy(r, r2))
///         }
///     },
///     ("noop") => Instruction::Noop,
/// );
///
/// assert_eq!(
///     instruction.parse_complete("add A, B").unwrap(),
///     Instruction::Add(Register::A, Register::B)
/// );
/// assert_eq!(
///     instruction.parse_complete("add C, 100").unwrap(),
///     Instruction::AddConstant(Register::C, 100)
/// );
/// assert_eq!(
///     instruction.parse_complete("copy A, B").unwrap(),
///     Instruction::Copy(Register::A, Register::B)
/// );
/// assert!(instruction
///     .parse_complete("copy A, A")
///     .is_err_and(|err| err.to_string().contains("cannot copy register to itself")));
/// ```
#[macro_export]
macro_rules! parser_parse_tree {
    (@rule $input:ident $furthest_err:ident $furthest_remaining:ident [$(,)?] @expr $rhs:expr) => {
        return Ok(($rhs, $input));
    };
    (@rule $input:ident $furthest_err:ident $furthest_remaining:ident [$(,)?] @expr_res $rhs:expr) => {
        match $rhs {
            Ok(v) => return Ok((v, $input)),
            Err(e) => {
                if $input.len() < $furthest_remaining {
                    $furthest_err = $crate::parser::ParseError::Custom(e);
                    $furthest_remaining = $input.len();
                }
            }
        };
    };
    (@rule $input:ident $furthest_err:ident $furthest_remaining:ident [$(,)?] @subtree $($rhs:tt)+) => {
        $crate::parser_parse_tree!(@toplevel $input $furthest_err $furthest_remaining $($rhs)+);
    };

    (@rule $input:ident $furthest_err:ident $furthest_remaining:ident
        [$n:ident @ $lhs:expr $(,$($tail:tt)*)?] $($rhs:tt)+
    ) => {
        match $crate::parser::Parser::parse(&($lhs), $input) {
            Ok(($n, $input)) => {
                $crate::parser_parse_tree!(@rule $input $furthest_err $furthest_remaining
                    [$($($tail)*)?] $($rhs)+
                );
            },
            Err((err, remaining)) => {
                if remaining.len() < $furthest_remaining {
                    $furthest_err = err;
                    $furthest_remaining = remaining.len();
                }
            }
        };
    };
    (@rule $input:ident $furthest_err:ident $furthest_remaining:ident
        [$lhs:expr $(,$($tail:tt)*)?] $($rhs:tt)+
    ) => {
        match $crate::parser::Parser::parse(&($lhs), $input) {
            Ok((_, $input)) => {
                $crate::parser_parse_tree!(@rule $input $furthest_err $furthest_remaining
                    [$($($tail)*)?] $($rhs)+
                );
            },
            Err((err, remaining)) => {
                if remaining.len() < $furthest_remaining {
                    $furthest_err = err;
                    $furthest_remaining = remaining.len();
                }
            }
        };
    };

    (@toplevel $input:ident $furthest_err:ident $furthest_remaining:ident
        ($($lhs:tt)+) => $rhs:expr $(, $($tail:tt)*)?
    ) => {
        $crate::parser_parse_tree!(@rule $input $furthest_err $furthest_remaining [$($lhs)+] @expr $rhs);
        $($crate::parser_parse_tree!(@toplevel $input $furthest_err $furthest_remaining $($tail)*);)?
    };
    (@toplevel $input:ident $furthest_err:ident $furthest_remaining:ident
        ($($lhs:tt)+) =?> $rhs:expr $(, $($tail:tt)*)?
    ) => {
        $crate::parser_parse_tree!(@rule $input $furthest_err $furthest_remaining [$($lhs)+] @expr_res $rhs);
        $($crate::parser_parse_tree!(@toplevel $input $furthest_err $furthest_remaining $($tail)*);)?
    };
    (@toplevel $input:ident $furthest_err:ident $furthest_remaining:ident
        ($($lhs:tt)+) =>> {$($rhs:tt)+} $(, $($tail:tt)*)?
    ) => {
        $crate::parser_parse_tree!(@rule $input $furthest_err $furthest_remaining [$($lhs)+] @subtree $($rhs)+);
        $($crate::parser_parse_tree!(@toplevel $input $furthest_err $furthest_remaining $($tail)*);)?
    };
    (@toplevel $input:ident $furthest_err:ident $furthest_remaining:ident $(,)?) => {};

    // Ensures this branch only matches inputs starting with (, giving each rule set a unique prefix
    (($($first:tt)+) $($tail:tt)+) => {{
        fn coerce_to_parser<F: Fn(&[u8]) -> $crate::parser::ParseResult<'_, O>, O>(f: F) -> F { f }

        coerce_to_parser(|input| {
            let mut furthest_err = $crate::parser::ParseError::Custom("unreachable");
            let mut furthest_remaining = usize::MAX;

            $crate::parser_parse_tree!(@toplevel input furthest_err furthest_remaining ($($first)+) $($tail)+);

            Err((furthest_err, &input[input.len() - furthest_remaining..]))
        })
    }};
}
