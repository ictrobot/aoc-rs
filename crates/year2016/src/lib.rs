#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "unsafe"), forbid(unsafe_code))]

utils::year!(2016 => year2016, ${
    5 => day05::Day05<'_>,
    14 => day14::Day14<'_>,
});
