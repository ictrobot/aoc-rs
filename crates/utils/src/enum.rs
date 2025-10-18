//! Enum helpers.

/// Macro to generate helpers for fieldless unit-only enums.
///
/// Helpers for accessing variants:
/// - `COUNT`: The number of variants.
/// - `ALL`: All variants (reference to a static array).
/// - `iter()`: [`Iterator`] over all variants.
///
/// Helpers for converting to and from the discriminant (requires the enum to have an explicit
/// `#[repr(...)]` attribute before all other attributes):
/// - `checked_from_discriminant()` & `from_discriminant()`: Safe and panicking conversions from
///   the discriminant.
/// - [`From`] implementation from the enum to the discriminant.
/// - [`TryFrom`] implementation from the discriminant to the enum.
///
/// Helpers for using variants as array indices (requires all variants to use implicit
/// discriminants):
/// - [`Index`](std::ops::Index) and [`IndexMut`](std::ops::IndexMut) implementations for
///   `[T; COUNT]` arrays.
///
/// See also [`parser::parsable_enum!`](crate::parser::parsable_enum), which combined this macro
/// with building a parser.
///
/// # Examples
///
/// Variant helpers:
/// ```
/// utils::enumerable_enum! {
///     enum Direction {
///         North,
///         East,
///         South,
///         West,
///     }
/// }
///
/// assert_eq!(Direction::COUNT, 4);
/// # // Use matches! as PartialEq is not derived or implemented in this example
/// assert!(matches!(Direction::ALL, &[
///     Direction::North,
///     Direction::East,
///     Direction::South,
///     Direction::West,
/// ]));
/// assert!(matches!(Direction::iter().collect::<Vec<_>>().as_slice(), &[
///     Direction::North,
///     Direction::East,
///     Direction::South,
///     Direction::West,
/// ]));
/// ```
///
/// Discriminant helpers:
/// ```
/// utils::enumerable_enum! {
///     #[repr(u8)]
///     #[derive(Copy, Clone, Debug, Default, PartialEq)]
///     enum Operation {
///         Add,
///         Sub,
///         Mul,
///         Div,
///         #[default]
///         Noop = 255,
///     }
/// }
///
/// assert_eq!(Operation::COUNT, 5);
/// assert_eq!(Operation::ALL, &[
///     Operation::Add,
///     Operation::Sub,
///     Operation::Mul,
///     Operation::Div,
///     Operation::Noop
/// ]);
/// assert_eq!(Operation::iter().collect::<Vec<_>>(), Operation::ALL);
/// assert_eq!(Operation::from_discriminant(0), Operation::Add);
/// assert_eq!(Operation::checked_from_discriminant(255), Some(Operation::Noop));
/// assert_eq!(Operation::checked_from_discriminant(100), None);
/// assert_eq!(u8::from(Operation::Add), 0u8);
/// assert_eq!(Operation::try_from(2u8), Ok(Operation::Mul));
/// assert_eq!(Operation::try_from(4u8), Err(()));
/// ```
///
/// `from_discriminant` panics on invalid values:
/// ```should_panic
/// utils::enumerable_enum! {
///     #[repr(u8)]
///     enum Operation {
///         Add,
///         Sub,
///         Mul,
///         Div,
///     }
/// }
///
/// Operation::from_discriminant(64);
/// ```
///
/// Index helpers:
/// ```
/// utils::enumerable_enum! {
///     enum Register {
///         A,
///         B,
///         C,
///         D,
///     }
/// }
///
/// let mut registers = [0, 1, 2, 3];
/// assert_eq!(registers[Register::A], 0);
/// assert_eq!(registers[Register::B], 1);
/// assert_eq!(registers[Register::C], 2);
/// assert_eq!(registers[Register::D], 3);
/// registers[Register::C] = 123;
/// assert_eq!(registers[Register::C], 123);
/// ```
#[macro_export]
macro_rules! enumerable_enum {
    (
        #[repr($t:ty)]
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($(#[$variant_meta:meta])* $variant:ident $(= $discriminant:expr)?),+ $(,)?
        }
    ) => {
        // #[cfg(true) is used to force the second arm to be matched, even when no other attributes
        // are provided.
        $crate::enumerable_enum! {
            $(#[$meta])*
            #[cfg(true)]
            #[repr($t)]
            $vis enum $name {
                $($(#[$variant_meta])* $variant $(= $discriminant)?,)+
            }
        }

        impl $name {
            /// Returns the variant with the provided discriminant, or [`None`] on invalid values.
            #[inline]
            #[must_use]
            #[allow(non_snake_case, non_upper_case_globals)]
            pub const fn checked_from_discriminant(v: $t) -> Option<Self> {
                $(const $variant: $t = $name::$variant as $t;)+

                match v {
                    $($variant => Some(Self::$variant),)+
                    _ => None,
                }
            }

            /// Returns the variant with the provided discriminant, panicking on invalid values.
            #[inline]
            #[must_use]
            pub const fn from_discriminant(v: $t) -> Self {
                Self::checked_from_discriminant(v).expect("invalid discriminant")
            }
        }

        impl From<$name> for $t {
            #[inline]
            fn from(v: $name) -> Self {
                v as Self
            }
        }

        impl TryFrom<$t> for $name {
            type Error = ();

            #[inline]
            fn try_from(value: $t) -> Result<Self, ()> {
                Self::checked_from_discriminant(value).ok_or(())
            }
        }
    };
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($(#[$variant_meta:meta])* $variant:ident $(= $discriminant:expr)?),+ $(,)?
        }
    ) => {
        $crate::enumerable_enum! {@enum
            $(#[$meta])*
            $vis enum $name {
                $($(#[$variant_meta])* $variant $(= $discriminant)?,)+
            }
        }

        impl $name {
            /// The number of variants.
            pub const COUNT: usize = [$(Self::$variant),+].len();
            /// All variants.
            pub const ALL: &'static [Self; Self::COUNT] = &[$(Self::$variant),+];

            /// Iterator over all variants.
            #[inline]
            pub fn iter() -> ::std::array::IntoIter<Self, { $name::COUNT }> {
                // Returns items by value without references even for non-copy types
                [$(Self::$variant),+].into_iter()
            }
        }
    };
    (@enum
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($(#[$variant_meta:meta])* $variant:ident),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $($(#[$variant_meta])* $variant,)+
        }

        impl<T> ::std::ops::Index<$name> for [T; $name::COUNT] {
            type Output = T;

            #[inline]
            fn index(&self, index: $name) -> &Self::Output {
                &self[index as usize]
            }
        }
        impl<T> ::std::ops::IndexMut<$name> for [T; $name::COUNT] {
            #[inline]
            fn index_mut(&mut self, index: $name) -> &mut Self::Output {
                &mut self[index as usize]
            }
        }
    };
    (@enum
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($(#[$variant_meta:meta])* $variant:ident $(= $discriminant:expr)?),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $($(#[$variant_meta])* $variant $(= $discriminant)?,)+
        }
    };
}
