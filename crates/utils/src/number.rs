//! Traits for using numbers as generic data types.
//!
//! Machine-dependent integer types aren't unsupported.

use std::fmt::Debug;
use std::iter::{Product, Sum};
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
    + PartialOrd
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
    + Sum<Self>
    + for<'a> Sum<&'a Self>
    + Product<Self>
    + for<'a> Product<&'a Self>
{
    const ZERO: Self;
    const ONE: Self;

    #[must_use]
    fn abs(self) -> Self;

    #[must_use]
    fn rem_euclid(self, rhs: Self) -> Self;
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

            #[inline]
            fn rem_euclid(self, rhs: Self) -> Self {
                self.rem_euclid(rhs)
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

            #[inline]
            fn rem_euclid(self, rhs: Self) -> Self {
                self.rem_euclid(rhs)
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

            #[inline]
            fn rem_euclid(self, rhs: Self) -> Self {
                self.rem_euclid(rhs)
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

/// Checks if the provided unsigned integer `n` is a prime number.
///
/// # Examples
/// ```
/// # use utils::number::is_prime;
/// assert_eq!(is_prime(7901u32), true);
/// assert_eq!(is_prime(2147483647u32), true);
/// assert_eq!(is_prime(4294967291u32), true);
/// assert_eq!(is_prime(6u32), false);
/// assert_eq!(is_prime(123u32), false);
/// ```
#[inline]
pub fn is_prime<T: UnsignedInteger>(n: T) -> bool {
    if n <= T::ONE {
        return false;
    }
    if n == T::from(2) || n == T::from(3) {
        return true;
    }
    if n % T::from(2) == T::ZERO || n % T::from(3) == T::ZERO {
        return false;
    }

    let mut i = T::from(5);
    while let Some(square) = i.checked_mul(i) {
        if square > n {
            break;
        }

        if n % i == T::ZERO || n % (i + T::from(2)) == T::ZERO {
            return false;
        }

        if let Some(next) = i.checked_add(T::from(6)) {
            i = next;
        } else {
            break;
        }
    }

    true
}

/// Computes the greatest common divisor (GCD) using the extended Euclidean algorithm.
///
/// Returns a tuple `(gcd, x, y)` where `x`, `y` are the coefficients of Bézout's identity:
/// ```text
/// ax + by = gcd(a, b)
/// ```
///
/// # Examples
/// ```
/// # use utils::number::egcd;
/// assert_eq!(egcd(252, 105), (21, -2, 5));
/// assert_eq!((252 * -2) + (105 * 5), 21);
/// ```
#[inline]
pub fn egcd<T: SignedInteger>(mut a: T, mut b: T) -> (T, T, T) {
    let (mut x0, mut x1, mut y0, mut y1) = (T::ONE, T::ZERO, T::ZERO, T::ONE);

    while b != T::ZERO {
        let q = a / b;
        (a, b) = (b, a % b);
        (x0, x1) = (x1, x0 - q * x1);
        (y0, y1) = (y1, y0 - q * y1);
    }

    (a, x0, y0)
}

/// Computes the modular inverse of `a` modulo `b` if it exists.
///
/// # Examples
/// ```
/// # use utils::number::mod_inverse;
/// assert_eq!(mod_inverse(3, 5), Some(2));
/// assert_eq!((3 * 2) % 5, 1);
///
/// assert_eq!(mod_inverse(10, 23), Some(7));
/// assert_eq!((10 * 7) % 23, 1);
///
/// assert_eq!(mod_inverse(2, 8), None);
/// ```
#[inline]
pub fn mod_inverse<T: SignedInteger>(a: T, b: T) -> Option<T> {
    let (gcd, x, _) = egcd(a, b);
    if gcd == T::ONE {
        Some(x.rem_euclid(b))
    } else {
        None
    }
}

/// Solves a system of simultaneous congruences using the Chinese Remainder Theorem.
///
/// This function finds the smallest non-negative integer `x` where `x % modulus = residue` for each
/// provided (residue, modulus) pair.
///
/// # Examples
/// ```
/// # use utils::number::chinese_remainder;
/// assert_eq!(chinese_remainder([1, 2, 3], [5, 7, 11]), Some(366));
/// assert_eq!(366 % 5, 1);
/// assert_eq!(366 % 7, 2);
/// assert_eq!(366 % 11, 3);
/// ```
#[inline]
pub fn chinese_remainder<T: SignedInteger>(
    residues: impl IntoIterator<Item = T>,
    moduli: impl IntoIterator<Item = T, IntoIter: Clone>,
) -> Option<T> {
    let moduli = moduli.into_iter();
    let product = moduli.clone().product();

    let mut sum = T::ZERO;
    for (residue, modulus) in residues.into_iter().zip(moduli) {
        let p = product / modulus;
        sum += residue * mod_inverse(p, modulus)? * p;
    }

    Some(sum.rem_euclid(product))
}
