/// Macro for implementing an enum error type wrapping one or more other errors.
///
/// Implements:
///  - [`Display`](std::fmt::Display), passing through to wrapped variants automatically.
///  - [`Error`](std::error::Error), including [`Error::source`](std::error::Error::source) for
///    exposing the underlying error.
///  - [`From`] for each of wrapped errors.
///
/// # Examples
///
/// Without custom variants:
///
/// ```no_run
/// # use utils::error_type;
/// use std::num::ParseIntError;
/// use std::str::Utf8Error;
///
/// error_type!(
///     pub enum MyError {} wraps [
///         ParseIntError,
///         Utf8Error,
///     ]
/// );
/// ```
///
/// With custom variants:
///
/// ```no_run
/// # use utils::error_type;
/// use std::fmt::Formatter;
/// use std::num::ParseIntError;
///
/// error_type!(
///     pub enum MyError {
///         InvalidValue(u32),
///         MissingValue(String),
///     } wraps [
///         ParseIntError,
///     ]
///     impl Display match {
///         Self::InvalidValue(x) => |f: &mut Formatter| write!(f, "value {x} out of range"),
///         Self::MissingValue(s) => |f: &mut Formatter| write!(f, "no value provided for {s}"),
///     }
/// );
/// ```
#[allow(clippy::module_name_repetitions)] // Once exported name is utils::error_type
#[macro_export]
macro_rules! error_type {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {$($body:tt)*}
        wraps [$($(#[$w_meta:meta])* $w:ident),+$(,)?]
        $(impl Display match {
            $($display:pat => $display_fn:expr,)+
        })?
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        $vis enum $name {
            $($body)*
            $(
                $(#[$w_meta])*
                #[doc = concat!("\n\nWrapper for [", stringify!($w), "]")]
                $w($w),
            )+
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $($($display => ($display_fn)(f),)+)?
                    $(Self::$w(e) => std::fmt::Display::fmt(&e, f),)+
                }
            }
        }

        impl std::error::Error for $name {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                #[allow(unreachable_patterns)]
                match self {
                    $(Self::$w(e) => Some(e),)+
                    _ => None,
                }
            }
        }

        $(
        impl From<$w> for $name {
            fn from(value: $w) -> Self {
                Self::$w(value)
            }
        }
        )+
    };
}
