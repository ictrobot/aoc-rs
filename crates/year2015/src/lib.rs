#![doc = include_str!("../README.md")]

utils::year!(2015 => year2015, ${
    1 => day01::Day01,
    2 => day02::Day02,
    3 => day03::Day03,
    4 => day04::Day04<'_>,
    5 => day05::Day05<'_>,
    6 => day06::Day06,
    7 => day07::Day07,
    8 => day08::Day08<'_>,
    9 => day09::Day09,
});
