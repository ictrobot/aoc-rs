//! Traits for using numbers as generic data types.
//!
//! Machine-dependent integer types aren't unsupported.

use std::fmt::Debug;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

/// Trait implemented by the primitive number types, combining common supertraits.
pub trait Number:
    Copy
    + Debug
    + Default
    + PartialEq
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

    #[must_use]
    fn abs(self) -> Self;
}

/// Trait implemented by the primitive signed integer and floating point types.
pub trait Signed: Number + Neg<Output = Self> + From<i8> {
    const MINUS_ONE: Self;
}

/// Trait implemented by the primitive integer types.
pub trait Integer:
    Number
    + Not<Output = Self>
    + BitAnd<Output = Self>
    + BitAndAssign
    + BitOr<Output = Self>
    + BitOrAssign
    + BitXor<Output = Self>
    + BitXorAssign
    + Shl<Output = Self>
    + Shl<u32, Output = Self>
    + ShlAssign
    + ShlAssign<u32>
    + Shr<Output = Self>
    + Shr<u32, Output = Self>
    + ShrAssign
    + ShrAssign<u32>
{
    #[must_use]
    fn checked_add(self, rhs: Self) -> Option<Self>;
    #[must_use]
    fn checked_sub(self, rhs: Self) -> Option<Self>;
    #[must_use]
    fn checked_mul(self, rhs: Self) -> Option<Self>;
    #[must_use]
    fn trailing_ones(self) -> u32;
    #[must_use]
    fn trailing_zeros(self) -> u32;
}

/// Trait implemented by the primitive unsigned integer types.
pub trait UnsignedInteger: Integer + From<u8> {}

/// Trait implemented by the primitive signed integer types.
pub trait SignedInteger: Integer + Signed {}

macro_rules! number_impl {
    (uint => $($t:ty),+) => {
        $(impl Number for $t {
            const ZERO: Self = 0;
            const ONE: Self = 1;

            #[inline]
            fn abs(self) -> Self {
                self // no-op for unsigned integers
            }
        })+

        number_impl! {@common integer => $($t),+}

        $(impl UnsignedInteger for $t {})+
    };
    (int => $($t:ty),+) => {
        $(impl Number for $t {
            const ZERO: Self = 0;
            const ONE: Self = 1;

            #[inline]
            fn abs(self) -> Self {
                self.abs()
            }
        })+

        number_impl! {@common integer => $($t),+}
        number_impl! {@common signed => $($t),+}

        $(impl SignedInteger for $t {})+
    };
    (float => $($t:ty),+) => {
        $(impl Number for $t {
            const ZERO: Self = 0.0;
            const ONE: Self = 1.0;

            #[inline]
            fn abs(self) -> Self {
                self.abs()
            }
        })+

        number_impl! {@common signed => $($t),+}
    };
    (@common signed => $($t:ty),+) => {
        $(impl Signed for $t {
            const MINUS_ONE: Self = -Self::ONE;
        })+
    };
    (@common integer => $($t:ty),+) => {
        $(impl Integer for $t {
            #[inline]
            fn checked_add(self, rhs: Self) -> Option<Self> {
                self.checked_add(rhs)
            }
            #[inline]
            fn checked_sub(self, rhs: Self) -> Option<Self> {
                self.checked_sub(rhs)
            }
            #[inline]
            fn checked_mul(self, rhs: Self) -> Option<Self> {
                self.checked_mul(rhs)
            }
            #[inline]
            fn trailing_ones(self) -> u32 {
                self.trailing_ones()
            }
            #[inline]
            fn trailing_zeros(self) -> u32 {
                self.trailing_zeros()
            }
        })+
    };
}
number_impl! {uint => u8, u16, u32, u64, u128}
number_impl! {int => i8, i16, i32, i64, i128}
number_impl! {float => f32, f64}
