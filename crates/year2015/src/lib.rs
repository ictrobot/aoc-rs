#![doc = include_str!("../README.md")]

utils::year!(2015 => year2015, ${
    1 => day01::Day01,
    2 => day02::Day02,
    3 => day03::Day03,
    4 => day04::Day04,
});
