use std::collections::HashMap;
use std::ops::ControlFlow;
use utils::prelude::*;

/// Finding swapped logic gates in an adder circuit.
#[derive(Clone, Debug)]
pub struct Day24 {
    wires: Vec<Wire>,
    wire_names: Vec<[u8; 3]>,
    x_initial: u64,
    y_initial: u64,
    z_indexes: Vec<usize>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Wire {
    X(usize),
    Y(usize),
    And(usize, usize),
    Or(usize, usize),
    Xor(usize, usize),
}

impl Day24 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let Some((initial_str, gate_str)) = input.split_once("\n\n") else {
            return Err(InputError::new(input, 0, "expected inputs and gates"));
        };

        let mut wires = Vec::new();
        let mut wire_names = Vec::new();
        let mut indexes = HashMap::new();
        let mut x_initial = 0;
        let mut y_initial = 0;
        let mut input_bits = 64;

        let mut next = (b'x', 0);
        for item in parser::byte_range(b'x'..=b'y')
            .then(parser::byte_range(b'0'..=b'9'))
            .then(parser::byte_range(b'0'..=b'9'))
            .with_suffix(": ")
            .then(parser::byte_range(b'0'..=b'1'))
            .with_suffix(parser::eol())
            .parse_iterator(initial_str)
        {
            let (wire, b) = item?;
            let n = ((wire.1 - b'0') * 10 + (wire.2 - b'0')) as usize;

            if (wire.0, n) != next {
                if next.0 == b'x' && wire == (b'y', b'0', b'0') {
                    input_bits = next.1;
                } else {
                    return Err(InputError::new(input, 0, "unexpected initial value"));
                }
            }

            if wire.0 == b'x' {
                x_initial |= u64::from(b == b'1') << n;
                wires.push(Wire::X(n));
            } else {
                y_initial |= u64::from(b == b'1') << n;
                wires.push(Wire::Y(n));
            }
            wire_names.push(wire.into());
            indexes.insert(wire.into(), wires.len() - 1);

            if n == input_bits - 1 {
                next = (b'?', 0);
            } else {
                next = (wire.0, n + 1);
            }
        }

        let mut z_indexes = vec![usize::MAX; input_bits + 1];
        let wire = parser::byte().repeat_n::<3, _>(parser::noop());
        for item in wire
            .then(parser::literal_map!(
                " AND " => Wire::And as fn(usize, usize) -> Wire,
                " OR " => Wire::Or,
                " XOR " => Wire::Xor,
            ))
            .then(wire.with_suffix(" -> "))
            .then(wire)
            .with_suffix(parser::eol())
            .parse_iterator(gate_str)
        {
            let (in1, gate, in2, out) = item?;

            let mut index_of = |n| {
                *indexes.entry(n).or_insert_with(|| {
                    wires.push(Wire::X(usize::MAX)); // Placeholder
                    wire_names.push(n);
                    wires.len() - 1
                })
            };

            let in1_index = index_of(in1);
            let in2_index = index_of(in2);
            let out_index = index_of(out);

            if wires[out_index] != Wire::X(usize::MAX) {
                return Err(InputError::new(input, 0, "duplicate wire definition"));
            }
            if out[0] == b'z' {
                let index = ((out[1] - b'0') * 10 + (out[2] - b'0')) as usize;
                if index < z_indexes.len() {
                    z_indexes[index] = out_index;
                } else {
                    return Err(InputError::new(input, 0, "too many z outputs"));
                }
            }

            wires[out_index] = gate(in1_index, in2_index);
        }

        if wires.contains(&Wire::X(usize::MAX)) {
            return Err(InputError::new(input, 0, "undefined wire"));
        }
        if z_indexes.contains(&usize::MAX) {
            return Err(InputError::new(input, 0, "undefined z output"));
        }

        Ok(Self {
            wires,
            wire_names,
            x_initial,
            y_initial,
            z_indexes,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        let mut z = 0;
        let mut cache = vec![None; self.wires.len()];
        for (i, &index) in self.z_indexes.iter().enumerate() {
            z |= u64::from(Self::evaluate(
                index,
                &self.wires,
                self.x_initial,
                self.y_initial,
                &mut cache,
            )) << i;
        }
        z
    }

    fn evaluate(index: usize, wires: &[Wire], x: u64, y: u64, cache: &mut [Option<bool>]) -> bool {
        if let Some(c) = cache[index] {
            return c;
        }
        let v = match wires[index] {
            Wire::X(n) => return x & (1 << n) != 0,
            Wire::Y(n) => return y & (1 << n) != 0,
            Wire::And(a, b) => {
                Self::evaluate(a, wires, x, y, cache) && Self::evaluate(b, wires, x, y, cache)
            }
            Wire::Or(a, b) => {
                Self::evaluate(a, wires, x, y, cache) || Self::evaluate(b, wires, x, y, cache)
            }
            Wire::Xor(a, b) => {
                Self::evaluate(a, wires, x, y, cache) ^ Self::evaluate(b, wires, x, y, cache)
            }
        };
        cache[index] = Some(v);
        v
    }

    #[must_use]
    pub fn part2(&self) -> String {
        let mut test_cases = Vec::new();
        for i in 0..self.z_indexes.len() - 1 {
            test_cases.push((i, 1u64 << i, 0u64));
            test_cases.push((i, 0u64, 1u64 << i));
            test_cases.push((i + 1, 1u64 << i, 1u64 << i));
            test_cases.push((i + 1, (1u64 << (i + 1)) - 1, 1u64));
            test_cases.push((i + 1, 1u64, (1u64 << (i + 1)) - 1));
        }

        let mut wires = self.wires.clone();
        if self.find_swaps(&test_cases, &mut wires, 0).is_continue() {
            panic!("failed to find working combination");
        }

        let mut changes = Vec::new();
        for (i, (&wire, &orig)) in wires.iter().zip(&self.wires).enumerate() {
            if wire != orig {
                changes.push(self.wire_names[i]);
            }
        }
        assert_eq!(changes.len(), 8, "found incorrect number of changes");

        changes.sort_unstable();
        changes.into_iter().fold(String::new(), |mut acc, name| {
            if !acc.is_empty() {
                acc.push(',');
            }
            acc.push(name[0] as char);
            acc.push(name[1] as char);
            acc.push(name[2] as char);
            acc
        })
    }

    fn find_swaps(
        &self,
        test_cases: &[(usize, u64, u64)],
        wires: &mut [Wire],
        assume: usize,
    ) -> ControlFlow<()> {
        let mut cache = vec![None; wires.len()];
        let mut used = vec![false; wires.len()];
        for &(n, x, y) in test_cases {
            cache.fill(None);
            let sum = x + y;
            for i in 0..n {
                let b = Self::evaluate(self.z_indexes[i], wires, x, y, &mut cache);
                if ((sum >> i) & 1 != 0) != b {
                    // Previous swap broke even earlier bit
                    return ControlFlow::Continue(());
                }
            }

            for i in 0..wires.len() {
                used[i] = cache[i].is_some();
            }

            let b = Self::evaluate(self.z_indexes[n], wires, x, y, &mut cache);
            if ((sum >> n) & 1 != 0) == b {
                continue;
            }
            if n < assume {
                // Previous swap didn't fix the bit it was trying to fix
                return ControlFlow::Continue(());
            }

            // Found bit with a wrong gate

            // Gates which were used for the first time this bit
            let candidates1 = cache
                .iter()
                .zip(&used)
                .enumerate()
                .filter(|&(_, (&c, &u))| c.is_some() && !u)
                .map(|(i, _)| i)
                .collect::<Vec<_>>();
            // Gates which weren't used previously and only contain current bits
            let mask = (1 << (n + 1)) - 1;
            let candidates2 = Self::used_bits(wires)
                .iter()
                .zip(&used)
                .enumerate()
                .filter(|(_, (&b, &u))| b != 0 && b & mask == b && !u)
                .map(|(i, _)| i)
                .collect::<Vec<_>>();

            // Try swapping each combination of candidates
            for &c1 in &candidates1 {
                for &c2 in &candidates2 {
                    if c1 == c2 {
                        continue;
                    }

                    (wires[c1], wires[c2]) = (wires[c2], wires[c1]);

                    // Check swap didn't create a loop
                    if !self.loops(wires) {
                        // Check swap fixed this test case before recursively calling and checking
                        // all cases from the start
                        cache.fill(None);
                        let b = Self::evaluate(self.z_indexes[n], wires, x, y, &mut cache);
                        if ((sum >> n) & 1 != 0) == b
                            && self.find_swaps(test_cases, wires, n + 1).is_break()
                        {
                            // This and future swaps work, found working combination
                            return ControlFlow::Break(());
                        }
                    }

                    // Failed, swap back and try next combination
                    (wires[c1], wires[c2]) = (wires[c2], wires[c1]);
                }
            }

            // No combinations worked, previous swap must be wrong
            return ControlFlow::Continue(());
        }

        ControlFlow::Break(())
    }

    fn used_bits(wires: &[Wire]) -> Vec<u64> {
        fn eval(index: usize, wires: &[Wire], used_bits: &mut [u64]) -> u64 {
            if used_bits[index] != 0 {
                return used_bits[index];
            }
            let v = match wires[index] {
                Wire::X(n) => return 1 << n,
                Wire::Y(n) => return 1 << n,
                Wire::And(a, b) | Wire::Or(a, b) | Wire::Xor(a, b) => {
                    eval(a, wires, used_bits) | eval(b, wires, used_bits)
                }
            };
            used_bits[index] = v;
            v
        }

        let mut used = vec![0; wires.len()];
        for i in 0..wires.len() {
            eval(i, wires, &mut used);
        }
        used
    }

    fn loops(&self, wires: &[Wire]) -> bool {
        fn eval(index: usize, wires: &[Wire], checked: &mut [bool], depth: usize) -> bool {
            if checked[index] {
                return false;
            }
            if depth > wires.len() {
                return true;
            }
            match wires[index] {
                Wire::X(_) | Wire::Y(_) => {}
                Wire::And(a, b) | Wire::Or(a, b) | Wire::Xor(a, b) => {
                    if eval(a, wires, checked, depth + 1) || eval(b, wires, checked, depth + 1) {
                        return true;
                    }
                }
            }
            checked[index] = true;
            false
        }

        let mut checked = vec![false; wires.len()];
        for i in 0..wires.len() {
            if eval(i, wires, &mut checked, 0) {
                return true;
            }
        }
        false
    }
}

examples!(Day24 -> (u64, &'static str) []);
