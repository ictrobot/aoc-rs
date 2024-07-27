//! Implementation of [`multiversion!`](crate::multiversion) macro.

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
/// Two invocations are supported: the first match arm which matches a dynamic dispatch function,
/// optionally followed by extra private helper functions, or the second match arm without a dynamic
/// dispatch function for library functions. Dynamic dispatch is controlled using a `dyn_dispatch`
/// "attribute", which should include the targets to generate versions for in preference order.
///
/// Both invocations start with a use statement which can be used for wildcard imports of
/// multiversioned libraries. Each path will be expanded to include the version's name - e.g. the
/// `avx2` version will expand `use {utils::simd::*};` into `use {utils::simd::avx2::*};`. Other
/// imports should be outside the macro invocation.
///
/// Supported function syntax is limited as this is a declarative macro. Dynamic dispatch functions
/// are the most limited, supporting only basic arguments and optionally a return type. Library
/// functions may have complex definitions including generics, up to the maximum supported number of
/// token trees per function definition.
///
/// Versions enabling additional features require the `unsafe` crate feature to be enabled, as
/// features are enabled using the
/// [`target_feature`](https://doc.rust-lang.org/reference/attributes/codegen.html#the-target_feature-attribute)
/// attribute, which requires functions to be marked as unsafe. A safe fallback version will always
/// be generated.
///
/// See [`crate::md5`] as an example.
#[macro_export]
macro_rules! multiversion {
    // One dynamic dispatch function, optionally with extra helper functions.
    (
        use {$($($path:ident::)+*),*};

        #[dyn_dispatch($target:ident,$($extra_targets:ident),+)]
        $(#[$m:meta])* $v:vis fn $name:ident($($arg_name:ident: $arg_type:ty),*) $(-> $ret:ty)? $body:block

        $($tail:tt)*
    ) => {
        /// [`multiversion!`] dynamic dispatch implementations.
        mod $name {
            #[allow(unused_imports)]
            use $crate::multiversion; // Needed for rustdoc links

            $crate::multiversion!{@mod [$target $($extra_targets)+],
                use {$($($path::)+*),*};
                $(#[$m])* $v fn $name($($arg_name: $arg_type),*) $(-> $ret)? $body
                $($tail)*
            }
        }

        /// [`multiversion!`] dynamic dispatch function.
        #[inline]
        $v fn $name($($arg_name: $arg_type),*) $(-> $ret)? {
            $crate::multiversion!{@dispatch [$target $($extra_targets)+],
                $name($($arg_name),*)
            }
        }
    };

    // Library-only definition, without a dynamic dispatch function.
    (
        use {$($($path:ident::)+*),*};

        $($tail:tt)+
    ) => {
        /// [`multiversion!`] scalar implementation.
        // for x in 1..LANES {} loops become no-ops in scalar mode
        #[allow(clippy::reversed_empty_ranges)]
        pub mod scalar {
            #[allow(unused_imports, clippy::wildcard_imports)]
            use {super::*, $($($path::)+scalar::*),*};

            $($tail)*
        }

        /// [`multiversion!`] avx2 implementation.
        #[cfg(all(feature="unsafe", any(target_arch = "x86", target_arch = "x86_64")))]
        #[allow(clippy::missing_safety_doc)]
        pub mod avx2 {
            #[allow(unused_imports, clippy::wildcard_imports)]
            use {super::*, $($($path::)+avx2::*),*};

            $crate::multiversion!{@helper target_feature(enable = "avx2") $($tail)*}
        }
    };

    // These arms work around macro_rules limitations where repetitions must have the
    // same structure as the input, which prevent using $(mod $extra_target { ... })+
    (@mod [], $($tail:tt)*) => {};
    (@mod [scalar $($extra_targets:ident)*],
        use {$($($path:ident::)+*),*};
        $(#[$m:meta])* $v:vis fn $name:ident($($arg_name:ident: $arg_type:ty),*) $(-> $ret:ty)? $body:block
        $($tail:tt)*
    ) => {
        /// [`multiversion!`] scalar implementation.
        // for x in 1..LANES {} loops become no-ops in scalar mode
        #[allow(clippy::reversed_empty_ranges)]
        mod scalar {
            #[allow(unused_imports, clippy::wildcard_imports)]
            use {super::super::*, $($($path::)+scalar::*),*};

            $(#[$m])* pub fn $name($($arg_name: $arg_type),*) $(-> $ret)? $body

            $($tail)*
        }
        pub use scalar::$name as scalar;

        // This should be the last target, so ignore extra targets. Enforced in @dispatch
    };
    (@mod [avx2 $($extra_targets:ident)*],
        use {$($($path:ident::)+*),*};
        $(#[$m:meta])* $v:vis fn $name:ident($($arg_name:ident: $arg_type:ty),*) $(-> $ret:ty)? $body:block
        $($tail:tt)*
    ) => {
        /// [`multiversion!`] avx2 implementation.
        #[cfg(all(feature="unsafe", any(target_arch = "x86", target_arch = "x86_64")))]
        #[allow(clippy::missing_safety_doc)]
        mod avx2 {
            #[allow(unused_imports, clippy::wildcard_imports)]
            use {super::super::*, $($($path::)+avx2::*),*};

            #[target_feature(enable = "avx2")]
            $(#[$m])* pub unsafe fn $name($($arg_name: $arg_type),*) $(-> $ret)? $body

            $crate::multiversion!{@helper target_feature(enable = "avx2") $($tail)*}
        }
        #[cfg(all(feature="unsafe", any(target_arch = "x86", target_arch = "x86_64")))]
        pub use avx2::$name as avx2;

        $crate::multiversion!{@mod [$($extra_targets)*],
            use {$($($path::)+*),*};
            $(#[$m])* $v fn $name($($arg_name: $arg_type),*) $(-> $ret)? $body
            $($tail)*
        }
    };

    // Similar workaround for the dynamic dispatch function
    (@dispatch [], $($tail:tt)*) => {};
    (@dispatch [scalar $($extra_targets:ident)*],
        $name:ident($($arg_name:ident),*)
    ) => {
        return $name::scalar($($arg_name),*);

        $( compile_error!(concat!("scalar should be the final dyn_dispatch target, move ", stringify!($extra_targets), " before")) )*
    };
    (@dispatch [avx2 $($extra_targets:ident)*],
        $name:ident($($arg_name:ident),*)
    ) => {
        #[cfg(all(feature = "unsafe", any(target_arch = "x86", target_arch = "x86_64")))]
        if is_x86_feature_detected!("avx2") {
            return unsafe { $name::avx2($($arg_name),*) }
        }

        $crate::multiversion!{@dispatch [$($extra_targets)*],
            $name($($arg_name),*)
        }
    };

    // Allow helper functions to have complex definitions up to the max number of supported token
    // trees. This structure is needed as `$($t:tt)+ $b:block` is ambiguous
    (@helper $t:meta) => {};
    // Replace #[inline(always)] which is incompatible with target_feature with normal #[inline]
    (@helper $t:meta #[inline(always)] $($tail:tt)*) => {$crate::multiversion!{@helper $t #[inline] $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $t10:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $t10 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $t10:tt $t11:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $t10 $t11 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $t10:tt $t11:tt $t12:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $t10 $t11 $t12 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $t10:tt $t11:tt $t12:tt $t13:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $t10 $t11 $t12 $t13 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $t10:tt $t11:tt $t12:tt $t13:tt $t14:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $t10 $t11 $t12 $t13 $t14 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $t10:tt $t11:tt $t12:tt $t13:tt $t14:tt $t15:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $t10 $t11 $t12 $t13 $t14 $t15 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $t10:tt $t11:tt $t12:tt $t13:tt $t14:tt $t15:tt $t16:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $t10 $t11 $t12 $t13 $t14 $t15 $t16 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $t10:tt $t11:tt $t12:tt $t13:tt $t14:tt $t15:tt $t16:tt $t17:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $t10 $t11 $t12 $t13 $t14 $t15 $t16 $t17 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $t10:tt $t11:tt $t12:tt $t13:tt $t14:tt $t15:tt $t16:tt $t17:tt $t18:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $t10 $t11 $t12 $t13 $t14 $t15 $t16 $t17 $t18 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $t10:tt $t11:tt $t12:tt $t13:tt $t14:tt $t15:tt $t16:tt $t17:tt $t18:tt $t19:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $t10 $t11 $t12 $t13 $t14 $t15 $t16 $t17 $t18 $t19 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $t10:tt $t11:tt $t12:tt $t13:tt $t14:tt $t15:tt $t16:tt $t17:tt $t18:tt $t19:tt $t20:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $t10 $t11 $t12 $t13 $t14 $t15 $t16 $t17 $t18 $t19 $t20 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $t10:tt $t11:tt $t12:tt $t13:tt $t14:tt $t15:tt $t16:tt $t17:tt $t18:tt $t19:tt $t20:tt $t21:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $t10 $t11 $t12 $t13 $t14 $t15 $t16 $t17 $t18 $t19 $t20 $t21 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $t10:tt $t11:tt $t12:tt $t13:tt $t14:tt $t15:tt $t16:tt $t17:tt $t18:tt $t19:tt $t20:tt $t21:tt $t22:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $t10 $t11 $t12 $t13 $t14 $t15 $t16 $t17 $t18 $t19 $t20 $t21 $t22 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $t10:tt $t11:tt $t12:tt $t13:tt $t14:tt $t15:tt $t16:tt $t17:tt $t18:tt $t19:tt $t20:tt $t21:tt $t22:tt $t23:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $t10 $t11 $t12 $t13 $t14 $t15 $t16 $t17 $t18 $t19 $t20 $t21 $t22 $t23 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $t10:tt $t11:tt $t12:tt $t13:tt $t14:tt $t15:tt $t16:tt $t17:tt $t18:tt $t19:tt $t20:tt $t21:tt $t22:tt $t23:tt $t24:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $t10 $t11 $t12 $t13 $t14 $t15 $t16 $t17 $t18 $t19 $t20 $t21 $t22 $t23 $t24 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $t10:tt $t11:tt $t12:tt $t13:tt $t14:tt $t15:tt $t16:tt $t17:tt $t18:tt $t19:tt $t20:tt $t21:tt $t22:tt $t23:tt $t24:tt $t25:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $t10 $t11 $t12 $t13 $t14 $t15 $t16 $t17 $t18 $t19 $t20 $t21 $t22 $t23 $t24 $t25 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $t10:tt $t11:tt $t12:tt $t13:tt $t14:tt $t15:tt $t16:tt $t17:tt $t18:tt $t19:tt $t20:tt $t21:tt $t22:tt $t23:tt $t24:tt $t25:tt $t26:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $t10 $t11 $t12 $t13 $t14 $t15 $t16 $t17 $t18 $t19 $t20 $t21 $t22 $t23 $t24 $t25 $t26 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $t10:tt $t11:tt $t12:tt $t13:tt $t14:tt $t15:tt $t16:tt $t17:tt $t18:tt $t19:tt $t20:tt $t21:tt $t22:tt $t23:tt $t24:tt $t25:tt $t26:tt $t27:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $t10 $t11 $t12 $t13 $t14 $t15 $t16 $t17 $t18 $t19 $t20 $t21 $t22 $t23 $t24 $t25 $t26 $t27 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $t10:tt $t11:tt $t12:tt $t13:tt $t14:tt $t15:tt $t16:tt $t17:tt $t18:tt $t19:tt $t20:tt $t21:tt $t22:tt $t23:tt $t24:tt $t25:tt $t26:tt $t27:tt $t28:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $t10 $t11 $t12 $t13 $t14 $t15 $t16 $t17 $t18 $t19 $t20 $t21 $t22 $t23 $t24 $t25 $t26 $t27 $t28 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $t10:tt $t11:tt $t12:tt $t13:tt $t14:tt $t15:tt $t16:tt $t17:tt $t18:tt $t19:tt $t20:tt $t21:tt $t22:tt $t23:tt $t24:tt $t25:tt $t26:tt $t27:tt $t28:tt $t29:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $t10 $t11 $t12 $t13 $t14 $t15 $t16 $t17 $t18 $t19 $t20 $t21 $t22 $t23 $t24 $t25 $t26 $t27 $t28 $t29 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $t10:tt $t11:tt $t12:tt $t13:tt $t14:tt $t15:tt $t16:tt $t17:tt $t18:tt $t19:tt $t20:tt $t21:tt $t22:tt $t23:tt $t24:tt $t25:tt $t26:tt $t27:tt $t28:tt $t29:tt $t30:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $t10 $t11 $t12 $t13 $t14 $t15 $t16 $t17 $t18 $t19 $t20 $t21 $t22 $t23 $t24 $t25 $t26 $t27 $t28 $t29 $t30 $b $crate::multiversion!{@helper $t $($tail)*}};
    (@helper $t:meta $(#[$m:meta])* $v:vis fn $n:ident $t0:tt $t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt $t7:tt $t8:tt $t9:tt $t10:tt $t11:tt $t12:tt $t13:tt $t14:tt $t15:tt $t16:tt $t17:tt $t18:tt $t19:tt $t20:tt $t21:tt $t22:tt $t23:tt $t24:tt $t25:tt $t26:tt $t27:tt $t28:tt $t29:tt $t30:tt $t31:tt $b:block $($tail:tt)*) => {$(#[$m])* #[$t] $v unsafe fn $n $t0 $t1 $t2 $t3 $t4 $t5 $t6 $t7 $t8 $t9 $t10 $t11 $t12 $t13 $t14 $t15 $t16 $t17 $t18 $t19 $t20 $t21 $t22 $t23 $t24 $t25 $t26 $t27 $t28 $t29 $t30 $t31 $b $crate::multiversion!{@helper $t $($tail)*}};
}

/// Helper for testing [`multiversion!`] library functions.
///
/// `#[target_feature(...)]` isn't applied to the test functions as the feature-specific code should
/// be elsewhere, inside a [`multiversion!`] macro.
#[cfg(test)]
#[allow(clippy::module_name_repetitions)] // Once exported name is utils::multiversion_test
#[macro_export]
macro_rules! multiversion_test {
    (
        use {$($($path:ident::)+*),*};

        #[test]
        $(#[$m:meta])* $v:vis fn multiversion() $body:block
    ) => {
        #[test]
        // for x in 1..LANES {} loops become no-ops in scalar mode
        #[allow(clippy::reversed_empty_ranges)]
        $(#[$m])*
        fn scalar() {
            #[allow(unused_imports, clippy::wildcard_imports)]
            use {$($($path::)+scalar::*),*};

            $body
        }

        #[test]
        #[cfg(all(feature="unsafe", any(target_arch = "x86", target_arch = "x86_64")))]
        $(#[$m])*
        fn avx2() {
            #[allow(unused_imports, clippy::wildcard_imports)]
            use {$($($path::)+avx2::*),*};

            if !std::arch::is_x86_feature_detected!("avx2") {
                use std::io::{stdout, Write};
                let _ = writeln!(&mut stdout(), "warning: skipping test in {}::avx2 due to missing avx2 support", module_path!());
                return;
            }

            unsafe { $body }
        }
    };
}
