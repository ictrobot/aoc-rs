//! Implementation of [`multiversion!`](crate::multiversion!) macro.

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::sync::OnceLock;

/// Macro to generate multiversioned functions.
///
/// This macro generates multiple versions of the provided functions, each optimized for different
/// hardware, as well as dynamic dispatch functions which select the best version to use at runtime
/// based on feature support.
///
/// This allows using instruction set extensions such as
/// [AVX2](https://en.wikipedia.org/wiki/Advanced_Vector_Extensions#Advanced_Vector_Extensions_2)
/// if the binary is run on a compatible processor, without enabling it at compile time, which would
/// create a binary which can't be run on processors without AVX2 support.
///
/// The first three rules are supported:
///
/// 1. The first rule matches a dynamic dispatch function, optionally followed by extra private
///    helper functions. Dynamic dispatch is controlled at runtime by supplying a path to a
///    [`LazyLock`](std::sync::LazyLock) containing a [`Version`] value in the `dyn_dispatch`
///    attribute.
///
/// 2. The second rule matches library functions without a dynamic dispatch function.
///
/// 3. The third rule takes a name of a library function generated using the second rule and
///    expands to a [`LazyLock`](std::sync::LazyLock) evaluating to the fastest version, unless a
///    global override is set using [`Version::set_override`]. The return value is intended to be
///    stored in a `static` item for use in a `dyn_dispatch` attribute.
///
/// The first two rules start with a use statement which can be used for wildcard imports of
/// multiversioned libraries. Each path will be expanded to include the version's name - e.g. the
/// `avx2` version will expand `use {utils::simd::*};` into `use {utils::simd::avx2::*};`. Other
/// imports should be outside the macro invocation.
///
/// Supported function syntax is limited as this is a declarative macro. Dynamic dispatch functions
/// are the most limited, supporting only basic arguments and optionally a return type.
///
/// Versions enabling additional features require the `unsafe` crate feature to be enabled, as
/// features are enabled using the
/// [`target_feature`](https://doc.rust-lang.org/reference/attributes/codegen.html#the-target_feature-attribute)
/// attribute, which requires callers to use `unsafe` blocks. A safe fallback version will always
/// be generated.
///
/// See [`crate::md5`] as an example. [`multiversion_test!`](crate::multiversion_test!) can be used
/// for testing multiversioned libraries.
#[macro_export]
macro_rules! multiversion {
    // One dynamic dispatch function, optionally with extra helper functions.
    (
        use {$($($path:ident::)+*),*};

        #[dyn_dispatch = $dispatch:path]
        $(#[$m:meta])* $v:vis fn $name:ident($($arg_name:ident: $arg_type:ty),*$(,)?) $(-> $ret:ty)? $body:block

        $($tail:tt)*
    ) => {
        /// [`multiversion!`] dynamic dispatch implementations.
        mod $name {
            #[allow(clippy::allow_attributes, unused_imports)]
            use {super::*, $crate::multiversion}; // multiversion import needed for rustdoc links

            $crate::multiversion!{
                use {$($($path::)+*),*};
                $(#[$m])* pub fn $name($($arg_name: $arg_type),*) $(-> $ret)? $body
                $($tail)*
            }
        }

        /// [`multiversion!`] dynamic dispatch function.
        #[inline]
        $v fn $name($($arg_name: $arg_type),*) $(-> $ret)? {
            use $crate::multiversion::Version::*;

            match *$dispatch {
                Scalar => $name::scalar::$name($($arg_name),*),
                Array128 => $name::array128::$name($($arg_name),*),
                Array256 => $name::array256::$name($($arg_name),*),
                #[cfg(feature = "all-simd")]
                Array4096 => $name::array4096::$name($($arg_name),*),
                #[cfg(all(feature = "unsafe", any(target_arch = "x86", target_arch = "x86_64")))]
                AVX2 => unsafe { $name::avx2::$name($($arg_name),*) },
                #[cfg(all(feature = "unsafe", feature = "all-simd", any(target_arch = "x86", target_arch = "x86_64")))]
                AVX2x2 => unsafe { $name::avx2x2::$name($($arg_name),*) },
                #[cfg(all(feature = "unsafe", feature = "all-simd", any(target_arch = "x86", target_arch = "x86_64")))]
                AVX2x4 => unsafe { $name::avx2x4::$name($($arg_name),*) },
                #[cfg(all(feature = "unsafe", feature = "all-simd", any(target_arch = "x86", target_arch = "x86_64")))]
                AVX2x8 => unsafe { $name::avx2x8::$name($($arg_name),*) },
                #[cfg(all(feature = "unsafe", target_arch = "aarch64"))]
                Neon => unsafe { $name::neon::$name($($arg_name),*) },
                #[cfg(all(feature = "unsafe", feature = "all-simd", target_arch = "aarch64"))]
                Neonx2 => unsafe { $name::neonx2::$name($($arg_name),*) },
                #[cfg(all(feature = "unsafe", feature = "all-simd", target_arch = "aarch64"))]
                Neonx4 => unsafe { $name::neonx4::$name($($arg_name),*) },
                #[cfg(all(feature = "unsafe", feature = "all-simd", target_arch = "aarch64"))]
                Neonx8 => unsafe { $name::neonx8::$name($($arg_name),*) },
            }
        }
    };

    // Library-only definition, without a dynamic dispatch function.
    (
        use {$($($path:ident::)+*),*};

        $($tail:tt)+
    ) => {
        /// [`multiversion!`] scalar implementation.
        pub mod scalar {
            #![allow(
                clippy::reversed_empty_ranges,
                clippy::range_plus_one,
                clippy::modulo_one,
                clippy::trivially_copy_pass_by_ref
            )]

            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {super::*, $($($path::)+scalar::*),*};

            $($tail)*
        }

        /// [`multiversion!`] array128 implementation.
        pub mod array128 {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {super::*, $($($path::)+array128::*),*};

            $($tail)*
        }

        /// [`multiversion!`] array256 implementation.
        pub mod array256 {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {super::*, $($($path::)+array256::*),*};

            $($tail)*
        }

        /// [`multiversion!`] array4096 implementation.
        #[cfg(feature="all-simd")]
        pub mod array4096 {
            #![allow(clippy::large_types_passed_by_value)]

            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {super::*, $($($path::)+array4096::*),*};

            $($tail)*
        }

        /// [`multiversion!`] avx2 implementation.
        #[cfg(all(feature = "unsafe", any(target_arch = "x86", target_arch = "x86_64")))]
        pub mod avx2 {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {super::*, $($($path::)+avx2::*),*};

            $crate::multiversion!{@enable target_feature(enable = "avx2") $($tail)*}
        }

        /// [`multiversion!`] avx2x2 implementation.
        #[cfg(all(feature = "unsafe", feature = "all-simd", any(target_arch = "x86", target_arch = "x86_64")))]
        pub mod avx2x2 {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {super::*, $($($path::)+avx2x2::*),*};

            $crate::multiversion!{@enable target_feature(enable = "avx2") $($tail)*}
        }

        /// [`multiversion!`] avx2x4 implementation.
        #[cfg(all(feature = "unsafe", feature = "all-simd", any(target_arch = "x86", target_arch = "x86_64")))]
        pub mod avx2x4 {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {super::*, $($($path::)+avx2x4::*),*};

            $crate::multiversion!{@enable target_feature(enable = "avx2") $($tail)*}
        }

        /// [`multiversion!`] avx2x8 implementation.
        #[cfg(all(feature = "unsafe", feature = "all-simd", any(target_arch = "x86", target_arch = "x86_64")))]
        pub mod avx2x8 {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {super::*, $($($path::)+avx2x8::*),*};

            $crate::multiversion!{@enable target_feature(enable = "avx2") $($tail)*}
        }

        /// [`multiversion!`] neon implementation.
        #[cfg(all(feature = "unsafe", target_arch = "aarch64"))]
        pub mod neon {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {super::*, $($($path::)+neon::*),*};

            $crate::multiversion!{@enable target_feature(enable = "neon") $($tail)*}
        }

        /// [`multiversion!`] neonx2 implementation.
        #[cfg(all(feature = "unsafe", feature = "all-simd", target_arch = "aarch64"))]
        pub mod neonx2 {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {super::*, $($($path::)+neonx2::*),*};

            $crate::multiversion!{@enable target_feature(enable = "neon") $($tail)*}
        }

        /// [`multiversion!`] neonx4 implementation.
        #[cfg(all(feature = "unsafe", feature = "all-simd", target_arch = "aarch64"))]
        pub mod neonx4 {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {super::*, $($($path::)+neonx4::*),*};

            $crate::multiversion!{@enable target_feature(enable = "neon") $($tail)*}
        }

        /// [`multiversion!`] neonx8 implementation.
        #[cfg(all(feature = "unsafe", feature = "all-simd", target_arch = "aarch64"))]
        pub mod neonx8 {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {super::*, $($($path::)+neonx8::*),*};

            $crate::multiversion!{@enable target_feature(enable = "neon") $($tail)*}
        }
    };

    // Microbenchmark for dynamic dispatch
    (fastest($name:ident())) => {
        ::std::sync::LazyLock::new(#[cfg_attr(target_family = "wasm", expect(unreachable_code))] || {
            use $crate::multiversion::Version::*;

            // Instant::now() isn't implemented in WebAssembly, so hardcode implementations
            #[cfg(all(target_family = "wasm", target_feature = "simd128"))]
            return Array256;
            #[cfg(all(target_family = "wasm"))]
            return Array128;

            if let Some(version) = $crate::multiversion::Version::get_override() {
                return version;
            }

            $crate::multiversion::VERSIONS
                .iter()
                .map(|&x| {
                    let start = ::std::time::Instant::now();
                    ::std::hint::black_box(match x {
                        Scalar => scalar::$name(),
                        Array128 => array128::$name(),
                        Array256 => array256::$name(),
                        #[cfg(feature = "all-simd")]
                        Array4096 => array4096::$name(),
                        #[cfg(all(feature = "unsafe", any(target_arch = "x86", target_arch = "x86_64")))]
                        AVX2 => unsafe { avx2::$name() },
                        #[cfg(all(feature = "unsafe", feature = "all-simd", any(target_arch = "x86", target_arch = "x86_64")))]
                        AVX2x2 => unsafe { avx2x2::$name() },
                        #[cfg(all(feature = "unsafe", feature = "all-simd", any(target_arch = "x86", target_arch = "x86_64")))]
                        AVX2x4 => unsafe { avx2x4::$name() },
                        #[cfg(all(feature = "unsafe", feature = "all-simd", any(target_arch = "x86", target_arch = "x86_64")))]
                        AVX2x8 => unsafe { avx2x8::$name() },
                        #[cfg(all(feature = "unsafe", target_arch = "aarch64"))]
                        Neon => unsafe { neon::$name() },
                        #[cfg(all(feature = "unsafe", feature = "all-simd", target_arch = "aarch64"))]
                        Neonx2 => unsafe { neonx2::$name() },
                        #[cfg(all(feature = "unsafe", feature = "all-simd", target_arch = "aarch64"))]
                        Neonx4 => unsafe { neonx4::$name() },
                        #[cfg(all(feature = "unsafe", feature = "all-simd", target_arch = "aarch64"))]
                        Neonx8 => unsafe { neonx8::$name() },
                    });
                    (start.elapsed(), x)
                })
                // .inspect(|x| { dbg!(x); })
                .min_by_key(|x| x.0)
                .unwrap()
                .1
        })
    };

    // Helper rules to add target_feature to helper functions
    (@enable $t:meta) => {};
    (@enable $t:meta
        $(#[$m:meta])* $v:vis const $n:ident: $ty:ty = $e:expr; $($tail:tt)*
    ) => {
        $(#[$m])* $v const $n: $ty = $e;
        $crate::multiversion!{@enable $t $($tail)*}
    };
    // Replace #[inline(always)] which is incompatible with target_feature with normal #[inline]
    (@enable $t:meta
        #[inline(always)] $($tail:tt)*
    ) => {
        $crate::multiversion!{@enable $t
            #[inline] $($tail)*
        }
    };
    (@enable $t:meta
        $(#[$m:meta])* $v:vis fn $($tail:tt)*
    ) => {
        $crate::multiversion!{@one_item $t
            $(#[$m])*
            #[$t]
            #[allow(clippy::allow_attributes, clippy::missing_safety_doc)]
            $v fn $($tail)*
        }
    };

    // Helper rule to pop the first item out of the tt sequence
    (@one_item $t:meta $item:item $($tail:tt)*) => {
        $item
        $crate::multiversion!{@enable $t $($tail)*}
    };
}

/// Helper for testing and benchmarking [`multiversion!`] library functions.
///
/// The first rule is for testing and creates individual
/// [`#[test]`](https://doc.rust-lang.org/reference/attributes/testing.html) functions for each
/// implementation.
///
/// The second rule is more general, duplicating the same expression for each implementation and
/// is useful for benchmarking.
///
/// `#[target_feature(...)]` isn't applied to the test code as the feature-specific code should
/// be elsewhere, inside a [`multiversion!`] macro.
#[macro_export]
macro_rules! multiversion_test {
    (
        use {$($($path:ident::)+*),*};

        #[test]
        $(#[$m:meta])* $v:vis fn multiversion() $body:block
    ) => {
        #[test]
        $(#[$m])*
        fn scalar() {
            #![allow(
                clippy::reversed_empty_ranges,
                clippy::range_plus_one,
                clippy::modulo_one,
                clippy::trivially_copy_pass_by_ref
            )]

            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {$($($path::)+scalar::*),*};

            $body
        }

        #[test]
        $(#[$m])*
        fn array128() {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {$($($path::)+array128::*),*};

            $body
        }

        #[test]
        $(#[$m])*
        fn array256() {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {$($($path::)+array256::*),*};

            $body
        }

        #[test]
        #[cfg(feature = "all-simd")]
        $(#[$m])*
        fn array4096() {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {$($($path::)+array4096::*),*};

            $body
        }

        #[test]
        #[cfg(all(feature = "unsafe", any(target_arch = "x86", target_arch = "x86_64")))]
        $(#[$m])*
        fn avx2() {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {$($($path::)+avx2::*),*};

            if !$crate::multiversion::Version::AVX2.supported() {
                use std::io::{stdout, Write};
                let _ = writeln!(&mut stdout(), "warning: skipping test in {}::avx2 due to missing avx2 support", module_path!());
                return;
            }

            unsafe { $body }
        }

        #[test]
        #[cfg(all(feature = "unsafe", feature = "all-simd", any(target_arch = "x86", target_arch = "x86_64")))]
        $(#[$m])*
        fn avx2x2() {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {$($($path::)+avx2x2::*),*};

            if !$crate::multiversion::Version::AVX2x2.supported() {
                use std::io::{stdout, Write};
                let _ = writeln!(&mut stdout(), "warning: skipping test in {}::avx2x2 due to missing avx2 support", module_path!());
                return;
            }

            unsafe { $body }
        }

        #[test]
        #[cfg(all(feature = "unsafe", feature = "all-simd", any(target_arch = "x86", target_arch = "x86_64")))]
        $(#[$m])*
        fn avx2x4() {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {$($($path::)+avx2x4::*),*};

            if !$crate::multiversion::Version::AVX2x4.supported() {
                use std::io::{stdout, Write};
                let _ = writeln!(&mut stdout(), "warning: skipping test in {}::avx2x4 due to missing avx2 support", module_path!());
                return;
            }

            unsafe { $body }
        }

        #[test]
        #[cfg(all(feature = "unsafe", feature = "all-simd", any(target_arch = "x86", target_arch = "x86_64")))]
        $(#[$m])*
        fn avx2x8() {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {$($($path::)+avx2x8::*),*};

            if !$crate::multiversion::Version::AVX2x8.supported() {
                use std::io::{stdout, Write};
                let _ = writeln!(&mut stdout(), "warning: skipping test in {}::avx2x8 due to missing avx2 support", module_path!());
                return;
            }

            unsafe { $body }
        }

        #[test]
        #[cfg(all(feature = "unsafe", target_arch = "aarch64"))]
        $(#[$m])*
        fn neon() {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {$($($path::)+neon::*),*};

            unsafe { $body }
        }

        #[test]
        #[cfg(all(feature = "unsafe", feature = "all-simd", target_arch = "aarch64"))]
        $(#[$m])*
        fn neonx2() {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {$($($path::)+neonx2::*),*};

            unsafe { $body }
        }

        #[test]
        #[cfg(all(feature = "unsafe", feature = "all-simd", target_arch = "aarch64"))]
        $(#[$m])*
        fn neonx4() {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {$($($path::)+neonx4::*),*};

            unsafe { $body }
        }

        #[test]
        #[cfg(all(feature = "unsafe", feature = "all-simd", target_arch = "aarch64"))]
        $(#[$m])*
        fn neonx8() {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {$($($path::)+neonx8::*),*};

            unsafe { $body }
        }
    };

    (
        use {$($($path:ident::)+*),*};

        { $($tail:tt)+ }
    ) => {
        #[allow(
            clippy::reversed_empty_ranges,
            clippy::range_plus_one,
            clippy::modulo_one,
            clippy::trivially_copy_pass_by_ref
        )]
        {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {$($($path::)+scalar::*),*};

            $crate::multiversion_test!(@expr { $($tail)+ });
        }

        {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {$($($path::)+array128::*),*};

            $crate::multiversion_test!(@expr { $($tail)+ });
        }

        {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {$($($path::)+array256::*),*};

            $crate::multiversion_test!(@expr { $($tail)+ });
        }

        #[cfg(feature = "all-simd")]
        #[allow(clippy::large_types_passed_by_value)]
        {
            #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
            use {$($($path::)+array4096::*),*};

            $crate::multiversion_test!(@expr { $($tail)+ });
        }

        #[cfg(all(feature = "unsafe", any(target_arch = "x86", target_arch = "x86_64")))]
        if $crate::multiversion::Version::AVX2.supported() {
            unsafe {
                #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
                use {$($($path::)+avx2::*),*};

                $crate::multiversion_test!(@expr { $($tail)+ });
            }

            #[cfg(feature = "all-simd")]
            unsafe {
                #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
                use {$($($path::)+avx2x2::*),*};

                $crate::multiversion_test!(@expr { $($tail)+ });
            }

            #[cfg(feature = "all-simd")]
            unsafe {
                #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
                use {$($($path::)+avx2x4::*),*};

                $crate::multiversion_test!(@expr { $($tail)+ });
            }

            #[cfg(feature = "all-simd")]
            unsafe {
                #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
                use {$($($path::)+avx2x8::*),*};

                $crate::multiversion_test!(@expr { $($tail)+ });
            }
        }

        #[cfg(all(feature = "unsafe", target_arch = "aarch64"))]
        {
            unsafe {
                #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
                use {$($($path::)+neon::*),*};

                $crate::multiversion_test!(@expr { $($tail)+ });
            }

            #[cfg(feature = "all-simd")]
            unsafe {
                #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
                use {$($($path::)+neonx2::*),*};

                $crate::multiversion_test!(@expr { $($tail)+ });
            }

            #[cfg(feature = "all-simd")]
            unsafe {
                #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
                use {$($($path::)+neonx4::*),*};

                $crate::multiversion_test!(@expr { $($tail)+ });
            }

            #[cfg(feature = "all-simd")]
            unsafe {
                #[allow(clippy::allow_attributes, unused_imports, clippy::wildcard_imports)]
                use {$($($path::)+neonx8::*),*};

                $crate::multiversion_test!(@expr { $($tail)+ });
            }
        }
    };
    (@expr $e:expr) => { $e }
}

macro_rules! versions_impl {
    ($($
        (#[$m:meta])*
        $name:ident $(if $supported:expr)?,
    )+) => {
        /// Versions generated by [`multiversion!`](crate::multiversion!) on this platform.
        #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
        pub enum Version {
            #[default] // First item is the default
            $(
                $(#[$m])*
                $(#[doc = concat!("Requires `", stringify!($supported), "`.")])?
                $name,
            )+
        }

        impl Version {
            /// Check if this version is supported at runtime.
            #[must_use]
            pub fn supported(self) -> bool {
                match self {
                    $($(#[$m])* Version::$name => true $(&& $supported)?,)+
                }
            }
        }

        impl FromStr for Version {
            type Err = UnknownVersion;

            /// Implementation is case-insensitive.
            fn from_str(x: &str) -> Result<Self, Self::Err> {
                $(
                    $(#[$m])*
                    if stringify!($name).eq_ignore_ascii_case(x) { return Ok(Version::$name); }
                )+
                Err(UnknownVersion(x.to_string()))
            }
        }

        /// Runtime generated list of supported versions.
        pub static VERSIONS: std::sync::LazyLock<Vec<Version>> = std::sync::LazyLock::new(|| {
            let mut vec = vec![$($(#[$m])* Version::$name,)+];
            vec.retain(|i| i.supported());
            vec
        });
    };
}
versions_impl! {
    Scalar,
    Array128,
    Array256,
    #[cfg(feature = "all-simd")]
    Array4096,
    #[cfg(all(feature = "unsafe", any(target_arch = "x86", target_arch = "x86_64")))]
    AVX2 if std::arch::is_x86_feature_detected!("avx2"),
    #[cfg(all(feature = "unsafe", feature = "all-simd", any(target_arch = "x86", target_arch = "x86_64")))]
    AVX2x2 if std::arch::is_x86_feature_detected!("avx2"),
    #[cfg(all(feature = "unsafe", feature = "all-simd", any(target_arch = "x86", target_arch = "x86_64")))]
    AVX2x4 if std::arch::is_x86_feature_detected!("avx2"),
    #[cfg(all(feature = "unsafe", feature = "all-simd", any(target_arch = "x86", target_arch = "x86_64")))]
    AVX2x8 if std::arch::is_x86_feature_detected!("avx2"),
    #[cfg(all(feature = "unsafe", target_arch = "aarch64"))]
    Neon,
    #[cfg(all(feature = "unsafe", feature = "all-simd", target_arch = "aarch64"))]
    Neonx2,
    #[cfg(all(feature = "unsafe", feature = "all-simd", target_arch = "aarch64"))]
    Neonx4,
    #[cfg(all(feature = "unsafe", feature = "all-simd", target_arch = "aarch64"))]
    Neonx8,
}

static OVERRIDE: OnceLock<Option<Version>> = OnceLock::new();

impl Version {
    /// Get the global version override, if any.
    ///
    /// This function will return [`Some`] containing the version provided to
    /// [`Version::set_override`] if it was previously called, or [`None`] otherwise.
    ///
    /// All dynamic dispatch implementations should respect this value.
    pub fn get_override() -> Option<Version> {
        *OVERRIDE.get_or_init(|| None)
    }

    /// Set the global version override.
    ///
    /// # Panics
    ///
    /// This function will panic if any of the following conditions are met:
    ///
    /// - The provided version is not [supported](Self::supported).
    ///
    /// - This function is called more than once.
    ///
    /// - This function is called after calling [`Version::get_override`] (or any multiversioned
    ///   dynamic dispatch function that respects the global override).
    pub fn set_override(version: Version) {
        assert!(version.supported(), "{version:?} is not supported!");

        if OVERRIDE.set(Some(version)).is_err() {
            // Value returned in Err() is the value passed to set, not the existing value
            if Self::get_override().is_none() {
                panic!("Version::set_override must be called before get_override");
            } else {
                panic!("Version::set_override called more than once");
            }
        }
    }
}

/// Error type returned when trying to convert an invalid string to a [`Version`].
#[derive(Debug)]
pub struct UnknownVersion(String);

impl Display for UnknownVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "unknown function version: {:#}", self.0)
    }
}

impl Error for UnknownVersion {}
