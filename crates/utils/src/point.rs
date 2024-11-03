//! 2D point implementation.

use crate::number::{Number, Signed, SignedInteger, UnsignedInteger};
use std::fmt::Debug;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

macro_rules! point_impl {
    ($(#[$m:meta])* $v:vis struct $s:ident{$($f:ident),+}) => {
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
            pub fn manhattan_distance(self) -> T {
                T::ZERO $(+ self.$f.abs())+
            }

            /// Returns the manhattan distance from the origin.
            #[inline]
            #[must_use]
            pub fn manhattan_distance_unsigned(self) -> T::Unsigned
            where
                T: SignedInteger
            {
                T::Unsigned::ZERO $(+ self.$f.unsigned_abs())+
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
            #[must_use]
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
            #[must_use]
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
            #[must_use]
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
    };
}

point_impl! {
    /// Struct representing a 2D point or vector.
    pub struct Point2D{x, y}
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

// point_impl! {pub struct Point3D{x, y, z}}
