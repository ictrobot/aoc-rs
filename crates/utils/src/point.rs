//! 2D & 3D point implementations.

use crate::number::{Integer, Number, Signed, UnsignedInteger};
use std::fmt::Debug;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

macro_rules! point_impl {
    ($n:literal, $tuple:tt =>
        $(#[$m:meta])* $v:vis struct $s:ident{$($i:tt => $f:ident),+}
    ) => {
        #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
        $(#[$m])* $v struct $s<T: Number> {
            $(pub $f: T,)+
        }

        impl<T: Number> $s<T> {
            pub const ORIGIN: Self = Self{$($f: T::ZERO),+};

            #[inline]
            #[must_use]
            pub const fn new($($f: T),+) -> Self {
                Self{$($f),+}
            }

            /// Returns the manhattan distance from the origin.
            #[inline]
            #[must_use]
            pub fn manhattan_distance(self) -> T::Unsigned
            where
                T: Integer
            {
                T::Unsigned::ZERO $(+ self.$f.unsigned_abs())+
            }

            /// Returns the manhattan distance from the specified point.
            #[inline]
            #[must_use]
            pub fn manhattan_distance_from(self, rhs: Self) -> T::Unsigned
            where
                T: Integer
            {
                T::Unsigned::ZERO $(+ self.$f.abs_diff(rhs.$f))+
            }

            /// Add the provided signed point, wrapping on overflow.
            ///
            /// Useful for adding a signed direction onto an unsigned position.
            #[inline]
            #[must_use]
            pub fn wrapping_add_signed(self, rhs: $s<T::Signed>) -> Self
            where
                T: UnsignedInteger,
            {
                Self{
                    $($f: self.$f.wrapping_add_signed(rhs.$f),)+
                }
            }
        }

        impl<T: Number> Add for $s<T> {
            type Output = Self;

            #[inline]
            fn add(self, rhs: Self) -> Self {
                Self{
                    $($f: self.$f + rhs.$f,)+
                }
            }
        }

        impl<T: Number> AddAssign for $s<T> {
            #[inline]
            fn add_assign(&mut self, rhs: Self) {
                $(self.$f += rhs.$f;)+
            }
        }

        impl<T: Number> Mul<T> for $s<T> {
            type Output = Self;

            #[inline]
            fn mul(self, rhs: T) -> Self {
                Self{
                    $($f: self.$f * rhs,)+
                }
            }
        }

        impl<T: Number> MulAssign<T> for $s<T> {
            #[inline]
            fn mul_assign(&mut self, rhs: T) {
                $(self.$f *= rhs;)+
            }
        }

        impl<T: Number> Sub for $s<T> {
            type Output = Self;

            #[inline]
            fn sub(self, rhs: Self) -> Self {
                Self{
                    $($f: self.$f - rhs.$f,)+
                }
            }
        }

        impl<T: Number> SubAssign for $s<T> {
            #[inline]
            fn sub_assign(&mut self, rhs: Self) {
                $(self.$f -= rhs.$f;)+
            }
        }

        impl<T: Number> From<[T; $n]> for $s<T> {
            #[inline]
            fn from(arr: [T; $n]) -> Self {
                Self{$(
                    $f: arr[$i],
                )+}
            }
        }

        impl<T: Number> From<$tuple> for $s<T> {
            #[inline]
            fn from(arr: $tuple) -> Self {
                Self{$(
                    $f: arr.$i,
                )+}
            }
        }

        impl<T: Number> From<$s<T>> for [T; $n] {
            #[inline]
            fn from(value: $s<T>) -> Self {
                [$(value.$f),+]
            }
        }

        impl<T: Number> From<$s<T>> for $tuple {
            #[inline]
            fn from(value: $s<T>) -> Self {
                ($(value.$f),+)
            }
        }
    };
}

point_impl! {2, (T, T) =>
    /// Struct representing a 2D point or vector.
    pub struct Point2D{0 => x, 1 => y}
}

impl<T: Signed> Point2D<T> {
    pub const UP: Self = Self {
        x: T::ZERO,
        y: T::ONE,
    };
    pub const RIGHT: Self = Self {
        x: T::ONE,
        y: T::ZERO,
    };
    pub const DOWN: Self = Self {
        x: T::ZERO,
        y: T::MINUS_ONE,
    };
    pub const LEFT: Self = Self {
        x: T::MINUS_ONE,
        y: T::ZERO,
    };

    /// Rotate this vector 90 degrees clockwise.
    #[inline]
    #[must_use]
    pub fn turn_right(self) -> Self {
        Self {
            x: self.y,
            y: -self.x,
        }
    }

    /// Rotate this vector 90 degrees counterclockwise.
    #[inline]
    #[must_use]
    pub fn turn_left(self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }
}

point_impl! {3, (T, T, T) =>
    /// Struct representing a 3D point or vector.
    pub struct Point3D{0 => x, 1 => y, 2 => z}
}
