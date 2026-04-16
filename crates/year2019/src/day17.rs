use crate::intcode::features::Day09Features;
use crate::intcode::{Event, Interpreter};
use std::io::Write;
use std::ops::ControlFlow;
use utils::array::ArrayVec;
use utils::geometry::Direction;
use utils::grid;
use utils::prelude::*;

/// Interpreting machine code to find a path, then compressing it.
#[derive(Clone, Debug)]
pub struct Day17 {
    interpreter: Interpreter,
    scaffold: Vec<bool>,
    rows: usize,
    cols: usize,
    robot_index: usize,
    robot_dir: Direction,
}

impl Day17 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let interpreter = Interpreter::parse(input, 1)?;

        let mut camera = interpreter.clone();
        let mut output = Vec::with_capacity(4096);
        loop {
            match camera.run::<Day09Features>() {
                Event::Halt => break,
                Event::Input => {
                    return Err(InputError::new(
                        input,
                        0,
                        "expected camera program to halt without requesting input",
                    ));
                }
                Event::Output(value)
                    if let Ok(byte) = u8::try_from(value)
                        && matches!(byte, b'\n' | b'.' | b'#' | b'^' | b'>' | b'v' | b'<') =>
                {
                    output.push(byte);
                }
                Event::Output(_) => {
                    return Err(InputError::new(input, 0, "expected valid camera output"));
                }
            }
        }
        while output.pop_if(|x| *x == b'\n').is_some() {}
        if output.is_empty() {
            return Err(InputError::new(
                input,
                0,
                "expected non-empty camera output",
            ));
        }

        let mut robot = None;
        let (rows, cols, scaffold) = grid::parse(
            std::str::from_utf8(output.as_slice()).unwrap(),
            1,
            false,
            |b| b == b'#',
            |b| matches!(b, b'.' | b'#'),
            |i, b| {
                let dir = match b {
                    b'^' => Direction::Up,
                    b'>' => Direction::Right,
                    b'v' => Direction::Down,
                    b'<' => Direction::Left,
                    _ => unreachable!("output already validated"),
                };
                if robot.replace((i, dir)).is_some() {
                    return Err("duplicate robot");
                }
                Ok(true)
            },
        )?;
        let Some((robot_index, robot_dir)) = robot else {
            return Err(InputError::new(input, 0, "expected one robot"));
        };

        Ok(Self {
            interpreter,
            scaffold,
            rows,
            cols,
            robot_index,
            robot_dir,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let mut part1 = 0;
        for y in 0..self.rows - 2 {
            for x in 0..self.cols - 2 {
                let index = (y + 1) * self.cols + (x + 1);
                if self.scaffold[index - self.cols]
                    && self.scaffold[index - 1]
                    && self.scaffold[index]
                    && self.scaffold[index + 1]
                    && self.scaffold[index + self.cols]
                {
                    part1 += (x * y) as u32;
                }
            }
        }
        part1
    }

    #[must_use]
    pub fn part2(&self) -> i64 {
        let mut path = Vec::with_capacity(256);
        let offsets = [-(self.cols as isize), 1, self.cols as isize, -1];
        let (mut index, mut dir) = (self.robot_index, self.robot_dir);
        loop {
            let left = dir.turn_left();
            let right = dir.turn_right();
            let (turn, next_dir) = match (
                self.scaffold[index.wrapping_add_signed(offsets[left as usize])],
                self.scaffold[index.wrapping_add_signed(offsets[right as usize])],
            ) {
                (true, false) => ('L', left),
                (false, true) => ('R', right),
                (false, false) => break,
                (true, true) => panic!("no solution found: unexpected t-junction"),
            };
            dir = next_dir;

            let offset = offsets[dir as usize];
            index = index.wrapping_add_signed(offset);

            let mut distance = 1;
            while let next = index.wrapping_add_signed(offset)
                && self.scaffold[next]
            {
                index = next;
                distance += 1;
            }

            write!(&mut path, "{turn},{distance},").unwrap();
        }

        let mut main = ArrayVec::new();
        let mut routines = [None; 3];
        if Self::compress(&path, &mut main, &mut routines).is_continue() {
            panic!("no solution found: failed to compress path into three routines");
        }

        let mut interpreter = self.interpreter.clone();
        interpreter.mem[0] = 2;

        for line in std::iter::once(main.as_slice())
            .chain(routines.into_iter().flatten())
            .map(|line| line.strip_suffix(b",").unwrap_or(line))
            .chain(std::iter::once(b"n".as_slice()))
        {
            for &b in line {
                interpreter.push_input(i64::from(b));
            }
            interpreter.push_input(i64::from(b'\n'));
        }

        loop {
            let output = interpreter.expect_output::<Day09Features>();
            if output > 255 {
                return output;
            }
        }
    }

    #[inline]
    fn compress<'a>(
        path: &'a [u8],
        main: &mut ArrayVec<u8, 20>,
        routines: &mut [Option<&'a [u8]>; 3],
    ) -> ControlFlow<()> {
        if path.is_empty() {
            return ControlFlow::Break(());
        }

        for r in 0..3 {
            if main.push(b'A' + r as u8).is_err() {
                return ControlFlow::Continue(());
            };
            main.push(b',')
                .expect("pushing 2nd byte into even length array should never fail");

            if let Some(routine) = routines[r] {
                if path.starts_with(routine) {
                    Self::compress(&path[routine.len()..], main, routines)?;
                }
            } else {
                for (i, _) in path
                    .iter()
                    .enumerate()
                    .filter(|&(_, &b)| b == b',')
                    .skip(1)
                    .step_by(2)
                    .take_while(|&(i, _)| i < 21)
                {
                    let (routine, remaining) = path.split_at(i + 1);
                    routines[r] = Some(routine);
                    Self::compress(remaining, main, routines)?;
                    routines[r] = None;
                }
            }

            let (_, _) = (main.pop(), main.pop());
        }

        ControlFlow::Continue(())
    }
}

examples!(Day17 -> (u32, i64) []);
