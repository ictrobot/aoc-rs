//! 2D, 3D and 4D vector implementations.

use crate::number::{Integer, Number, Signed, UnsignedInteger};
use std::fmt::Debug;
use std::ops::{Add, AddAssign, Mul, MulAssign, Not, Sub, SubAssign};

macro_rules! vec_impl {
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

            /// Returns a vector with all components set to the provided value.
            #[inline]
            #[must_use]
            pub const fn splat(v: T) -> Self {
                Self{$($f: v),+}
            }

            /// Map each component of this vector using the provided function.
            #[inline]
            #[must_use]
            pub fn map<O: Number>(self, mut f: impl FnMut(T) -> O) -> $s<O> {
                $s{$($f: f(self.$f)),+}
            }

            /// Convert each component of this vector.
            #[inline]
            #[must_use]
            pub fn cast<O: Number + From<T>>(self) -> $s<O> {
                self.map(O::from)
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

            /// Returns the manhattan distance to the specified point.
            #[inline]
            #[must_use]
            pub fn manhattan_distance_to(self, rhs: Self) -> T::Unsigned
            where
                T: Integer
            {
                T::Unsigned::ZERO $(+ self.$f.abs_diff(rhs.$f))+
            }

            /// Returns the manhattan distance to the specified axis-aligned bounding box.
            #[inline]
            #[must_use]
            pub fn manhattan_distance_to_aabb(self, min: Self, max: Self) -> T::Unsigned
            where
                T: Integer
            {
                T::Unsigned::ZERO $(
                    + min.$f.saturating_sub_0(self.$f)
                    + self.$f.saturating_sub_0(max.$f)
                )+
            }

            /// Returns the squared euclidean distance to the specified point.
            #[inline]
            #[must_use]
            pub fn squared_euclidean_distance_to(self, rhs: Self) -> T {
                T::ZERO $(+ self.$f.squared_diff(rhs.$f))+
            }

            /// Add the provided signed vector, wrapping on overflow.
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

            /// Returns the per-component minimum.
            #[inline]
            #[must_use]
            pub fn component_min(self, rhs: Self) -> Self
            where
                T: Ord,
            {
                Self{
                    $($f: self.$f.min(rhs.$f),)+
                }
            }


            /// Returns the per-component maximum.
            #[inline]
            #[must_use]
            pub fn component_max(self, rhs: Self) -> Self
            where
                T: Ord,
            {
                Self{
                    $($f: self.$f.max(rhs.$f),)+
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

        vec_impl!(@widen $s{$($f),+},
            u8 => [u16, u32, u64, u128, i16, i32, i64, i128, f32, f64],
            u16 => [u32, u64, u128, i32, i64, i128, f32, f64],
            u32 => [u64, u128, i64, i128, f64],
            u64 => [u128, i128],
            i8 => [i16, i32, i64, i128, f32, f64],
            i16 => [i32, i64, i128, f32, f64],
            i32 => [i64, i128, f64],
            i64 => [i128],
        );
    };
    (@widen $s:ident{$($f:ident),+}, $($from:ident => [$($to:ident),+],)+) => {$($(
        impl From<$s<$from>> for $s<$to> {
            #[inline(always)]
            fn from(value: $s<$from>) -> Self {
                value.cast()
            }
        }
    )+)+};
}

vec_impl! {2, (T, T) =>
    /// Struct representing a 2D vector or point.
    #[doc(alias("Vector", "Vector2", "Point", "Point2", "Point2D"))]
    pub struct Vec2{0 => x, 1 => y}
}

impl<T: Signed> Vec2<T> {
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
    pub const DIRECTIONS: [Self; 4] = [Self::UP, Self::RIGHT, Self::DOWN, Self::LEFT];

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

vec_impl! {3, (T, T, T) =>
    /// Struct representing a 3D vector or point.
    #[doc(alias("Vector3", "Point3", "Point3D"))]
    pub struct Vec3{0 => x, 1 => y, 2 => z}
}

vec_impl! {4, (T, T, T, T) =>
    /// Struct representing a 4D vector or point.
    #[doc(alias("Vector4", "Point4", "Point4D"))]
    pub struct Vec4{0 => x, 1 => y, 2 => z, 3 => w}
}

/// Enum representing the four cardinal directions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    #[default]
    Up = 0,
    Right,
    Down,
    Left,
}

impl Direction {
    /// Rotate this direction by the provided turn.
    #[inline]
    #[must_use]
    pub fn turn(self, turn: Turn) -> Self {
        Self::from((self as u8).wrapping_add_signed(turn as i8))
    }

    /// Rotate this direction 90 degrees anticlockwise.
    #[inline]
    #[must_use]
    pub fn turn_left(self) -> Self {
        self.turn(Turn::Left)
    }

    /// Rotate this direction 90 degrees clockwise.
    #[inline]
    #[must_use]
    pub fn turn_right(self) -> Self {
        self.turn(Turn::Right)
    }
}

impl From<u8> for Direction {
    #[inline]
    fn from(value: u8) -> Self {
        match value % 4 {
            0 => Direction::Up,
            1 => Direction::Right,
            2 => Direction::Down,
            3 => Direction::Left,
            _ => unreachable!(),
        }
    }
}

impl Not for Direction {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        Self::from((self as u8).wrapping_add(2))
    }
}

impl<T: Signed> From<Direction> for Vec2<T> {
    #[inline]
    fn from(value: Direction) -> Self {
        Vec2::DIRECTIONS[value as usize]
    }
}

impl<T: Signed> Add<Direction> for Vec2<T> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Direction) -> Self::Output {
        self + Vec2::from(rhs)
    }
}

impl<T: Signed> Sub<Direction> for Vec2<T> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Direction) -> Self::Output {
        self - Vec2::from(rhs)
    }
}

/// Enum representing possible turns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum Turn {
    #[doc(alias("Anticlockwise"))]
    Left = -1,
    #[doc(alias("Straight"))]
    #[default]
    None = 0,
    #[doc(alias("Clockwise"))]
    Right = 1,
}
