//! Traits for using numbers as generic data types.
//!
//! Machine-dependent integer types aren't unsupported.

use std::fmt::Debug;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};

/// Trait implemented by the primitive number types, combining common supertraits.
pub trait Number:
    Copy
    + Debug
    + Default
    + Add<Output = Self>
    + AddAssign
    + Div<Output = Self>
    + DivAssign
    + Mul<Output = Self>
    + MulAssign
    + Rem<Output = Self>
    + RemAssign
    + Sub<Output = Self>
    + SubAssign
{
    const ZERO: Self;
    const ONE: Self;
}

macro_rules! number_impl {
    ($zero:literal, $one:literal => $($t:ty),+) => {$(
        impl Number for $t {
            const ZERO: Self = $zero;
            const ONE: Self = $one;
        }
    )+};
}
number_impl! {0, 1 => u8, u16, u32, u64, u128}
number_impl! {0, 1 => i8, i16, i32, i64, i128}
number_impl! {0.0, 1.0 => f32, f64}

/// Trait implemented by the primitive integer types.
pub trait Integer: Number {
    fn checked_add(self, rhs: Self) -> Option<Self>;
    fn checked_sub(self, rhs: Self) -> Option<Self>;
    fn checked_mul(self, rhs: Self) -> Option<Self>;
}

macro_rules! integer_impl {
    ($($t:ty),+) => {$(
        impl Integer for $t {
            fn checked_add(self, rhs: Self) -> Option<Self> {
                self.checked_add(rhs)
            }
            fn checked_sub(self, rhs: Self) -> Option<Self> {
                self.checked_sub(rhs)
            }
            fn checked_mul(self, rhs: Self) -> Option<Self> {
                self.checked_mul(rhs)
            }
        }
    )+};
}
integer_impl! {u8, u16, u32, u64, u128}
integer_impl! {i8, i16, i32, i64, i128}

/// Trait implemented by the primitive unsigned integer types.
pub trait Unsigned: Integer + From<u8> {}

macro_rules! unsigned_impl {
    ($($t:ty),+) => {$(
        impl Unsigned for $t {}
    )+};
}
unsigned_impl! {u8, u16, u32, u64, u128}

/// Trait implemented by the primitive signed integer types.
pub trait Signed: Integer + From<i8> {
    const MINUS_ONE: Self;
}

macro_rules! signed_impl {
    ($($t:ty),+) => {$(
        impl Signed for $t {
            const MINUS_ONE: Self = -1;
        }
    )+};
}
signed_impl! {i8, i16, i32, i64, i128}
