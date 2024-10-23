#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "unsafe"), forbid(unsafe_code))]

utils::year!(2017 => year2017, ${
    1 => day01::Day01<'_>,
});
