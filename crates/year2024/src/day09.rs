use std::collections::VecDeque;
use utils::prelude::*;

/// Implementing disk defragmentation.
#[derive(Clone, Debug)]
pub struct Day09<'a> {
    input: &'a str,
    total_length: u32,
}

impl<'a> Day09<'a> {
    pub fn new(input: &'a str, _: InputType) -> Result<Self, InputError> {
        let mut pos = 0;
        for b in input.bytes() {
            if b.is_ascii_digit() {
                pos += (b - b'0') as u32;
            } else {
                return Err(InputError::new(input, b as char, "expected digit"));
            }
        }
        Ok(Self {
            input,
            total_length: pos,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        let mut free_iterator = self.free_space_iter();
        let mut file_iterator = self.rev_file_iter();

        let mut checksum = 0;
        let (mut free_pos, mut free_len) = (0, 0);
        let (mut file_pos, mut file_len, mut file_id) = (0, 0, 0);
        loop {
            if free_len == 0 {
                let Some(next) = free_iterator.next() else {
                    break;
                };
                (free_pos, free_len) = next;
            }
            if file_len == 0 {
                let Some(next) = file_iterator.next() else {
                    break;
                };
                (file_pos, file_len, file_id) = next;
            }

            if free_pos > file_pos {
                break;
            }

            let len = free_len.min(file_len);
            checksum += Self::file_checksum(free_pos, len, file_id);
            free_pos += len;
            free_len -= len;
            file_len -= len;
        }

        // Handle the remaining file_len that wasn't moved into a free space, if any
        if file_len > 0 {
            checksum += Self::file_checksum(file_pos, file_len, file_id);
        }

        // Handle remaining complete files, if any
        for (file_pos, file_len, file_id) in file_iterator {
            checksum += Self::file_checksum(file_pos, file_len, file_id);
        }

        checksum
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        // Each VecDeque stores a lazily populated sorted list of free space positions that are i+1
        // long. Elements are always popped from the start of the deque, and newly parsed free
        // spaces are always pushed to the end of the deque.
        //
        // When a file is moved and partially consumes a free space, the remainder has to be
        // inserted into the correct index to maintain the sort order, which may be in the middle of
        // the deque. However, this happens relatively infrequently in the input (<400 times),
        // and in my testing using a deque is faster than using a BTreeSet or BinaryHeap.
        let mut free_positions: [_; 9] =
            std::array::from_fn(|_| VecDeque::with_capacity(self.input.len() / 10));
        for (pos, len) in self.free_space_iter() {
            free_positions[len as usize - 1].push_back(pos);
        }

        let mut checksum = 0;
        let mut file_iterator = self.rev_file_iter();
        for (file_pos, file_len, file_id) in &mut file_iterator {
            let (mut next_pos, mut free_len) = (file_pos, 0);

            // Check for free spaces at least as long as the file before the file's position
            for len in file_len..=9 {
                if let Some(&pos) = free_positions[len as usize - 1].front() {
                    if pos < next_pos {
                        (next_pos, free_len) = (pos, len);
                    }
                }
            }

            checksum += Self::file_checksum(next_pos, file_len, file_id);

            if next_pos < file_pos {
                // Remove now used free space, and add remaining space if any
                free_positions[free_len as usize - 1].pop_front();
                if free_len > file_len {
                    let (pos, len) = (next_pos + file_len, free_len - file_len);
                    match free_positions[len as usize - 1].binary_search(&pos) {
                        Ok(_) => unreachable!(),
                        Err(i) => free_positions[len as usize - 1].insert(i, pos),
                    }
                }
            } else if file_len == 1 {
                // If there is no space prior to the current file to move a 1-length file, then
                // either there is no free space remaining or it is all after the remaining files
                break;
            }
        }

        // Handle remaining files, if any
        for (file_pos, file_len, file_id) in file_iterator {
            checksum += Self::file_checksum(file_pos, file_len, file_id);
        }

        checksum
    }

    fn free_space_iter(&self) -> impl Iterator<Item = (u32, u32)> + use<'_> {
        struct FreeSpaceIterator<'a> {
            input: &'a [u8],
            pos: u32,
        }

        impl Iterator for FreeSpaceIterator<'_> {
            type Item = (u32, u32);

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                while self.input.len() >= 2 {
                    let file_len = (self.input[0] - b'0') as u32;
                    let free_len = (self.input[1] - b'0') as u32;
                    let free_pos = self.pos + file_len;

                    self.input = &self.input[2..];
                    self.pos += file_len + free_len;

                    if free_len > 0 {
                        return Some((free_pos, free_len));
                    }
                }
                None
            }
        }

        FreeSpaceIterator {
            input: self.input.as_bytes(),
            pos: 0,
        }
    }

    fn rev_file_iter(&self) -> impl Iterator<Item = (u32, u32, u32)> + use<'_> {
        struct ReverseFileIterator<'a> {
            input: &'a [u8],
            pos: u32,
            id: u32,
        }

        impl Iterator for ReverseFileIterator<'_> {
            type Item = (u32, u32, u32);

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                while !self.input.is_empty() {
                    if self.input.len() % 2 == 0 {
                        self.pos -= (self.input[self.input.len() - 1] - b'0') as u32;
                        self.input = &self.input[..self.input.len() - 1];
                    }

                    let file_len = (self.input[self.input.len() - 1] - b'0') as u32;
                    self.pos -= file_len;
                    self.id -= 1;
                    self.input = &self.input[..self.input.len() - 1];

                    if file_len > 0 {
                        return Some((self.pos, file_len, self.id));
                    }
                }
                None
            }
        }

        ReverseFileIterator {
            input: self.input.as_bytes(),
            pos: self.total_length,
            id: self.input.len().div_ceil(2) as u32,
        }
    }

    #[inline]
    fn file_checksum(pos: u32, len: u32, id: u32) -> u64 {
        let pos_sum = len * (2 * pos + len - 1) / 2;
        pos_sum as u64 * id as u64
    }
}

examples!(Day09<'_> -> (u64, u64) [
    {input: "2333133121414131402", part1: 1928, part2: 2858},
]);
