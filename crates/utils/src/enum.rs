//! Enum helpers.

/// Macro to generate helpers for fieldless unit-only enums.
///
/// Helpers for accessing variants:
/// - `COUNT`: The number of variants.
/// - `ALL`: All variants (reference to a static array).
/// - `iter()`: [`Iterator`] over all variants.
///
/// Helpers for converting to and from the discriminant:
/// - `checked_from_discriminant()` & `from_discriminant()`: Safe and panicking conversions from
///   the discriminant.
/// - [`From`] implementation from the enum to the discriminant.
/// - [`TryFrom`] implementation from the discriminant to the enum.
///
/// The discriminant utilities are only generated if the enum has an explicit `#[repr(...)]`
/// attribute before all other attributes.
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
        $(#[$meta])*
        $vis enum $name {
            $($(#[$variant_meta])* $variant $(= $discriminant)?,)+
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
    }
}
