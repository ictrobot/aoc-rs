//! 2D point implementation.

use crate::number::{Number, Signed};
use std::fmt::Debug;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

macro_rules! point_impl {
    ($v:vis struct $s:ident{$($f:ident),+}) => {
        #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
        $v struct $s<T: Number> {
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

point_impl! {pub struct Point2D{x, y}}

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
