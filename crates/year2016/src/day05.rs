use std::ops::DerefMut;
use std::sync::Mutex;
use utils::md5;
use utils::prelude::*;

/// Finding MD5 hashes, part two.
///
/// Very similar to [2015 Day 4](../year2015/struct.Day04.html), but with more complex logic to
/// assemble the answer from the matching hashes.
///
/// See [`md5::find_hash_with_appended_count()`].
#[derive(Clone, Debug)]
pub struct Day05<'a> {
    input: &'a str,
}

impl<'a> Day05<'a> {
    pub fn new(input: &'a str, _: InputType) -> Result<Self, InputError> {
        Ok(Self { input })
    }

    #[must_use]
    pub fn part1(&self) -> String {
        let mutex = Mutex::new(Vec::new());

        md5::find_hash_with_appended_count(self.input, |i, [a, ..]| {
            if a & 0xFFFF_F000 != 0 {
                return false;
            }

            let character = match (a & 0x00000F00) >> 8 {
                n @ 0..=9 => b'0' + n as u8,
                n @ 10..=15 => b'a' + (n - 10) as u8,
                _ => unreachable!(),
            };

            let mut guard = mutex.lock().unwrap();
            guard.push((i, character));

            guard.len() >= 8
        });

        let mut vec = mutex.into_inner().unwrap();
        vec.sort_unstable();
        vec.iter().take(8).map(|&(_, b)| b as char).collect()
    }

    #[must_use]
    pub fn part2(&self) -> String {
        let mutex = Mutex::new(([0u8; 8], [0u64; 8]));

        md5::find_hash_with_appended_count(self.input, |i, [a, ..]| {
            if a & 0xFFFF_F800 != 0 {
                return false;
            }

            let position = ((a & 0x0000_0F00) >> 8) as usize;
            let character = match (a & 0x0000_00F0) >> 4 {
                n @ 0..=9 => b'0' + n as u8,
                n @ 10..=15 => b'a' + (n - 10) as u8,
                _ => unreachable!(),
            };

            let mut guard = mutex.lock().unwrap();
            let (password, counts) = guard.deref_mut();

            if password[position] == 0 || i < counts[position] {
                password[position] = character;
                counts[position] = i;
            }

            password.iter().all(|&x| x > 0)
        });

        let (password, ..) = mutex.into_inner().unwrap();
        String::from_utf8(password.to_vec()).unwrap()
    }
}

examples!(Day05<'_> -> (&'static str, &'static str) [
    {input: "abc", part1: "18f47a30", part2: "05ace8e3"},
]);
