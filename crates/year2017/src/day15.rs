use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use utils::multithreading;
use utils::number::mod_pow;
use utils::prelude::*;

/// Comparing numbers from two simple random number generators.
///
/// Uses a pool of worker threads to calculate batches of the random numbers.
#[derive(Clone, Debug)]
pub struct Day15 {
    batches: BTreeMap<u32, BatchResults>,
}

#[derive(Clone, Debug)]
struct BatchResults {
    part1_matches: u32,
    part2_a_values: Vec<u16>,
    part2_b_values: Vec<u16>,
}

const FACTOR_A: u64 = 16807;
const FACTOR_B: u64 = 48271;
const MODULUS: u64 = 2147483647;

const PART1_PAIRS: u32 = 40_000_000;
const PART2_PAIRS: u32 = 5_000_000;
const BATCH_SIZE: u32 = 250_000; // BATCH_SIZE must evenly divide PART1_PAIRS

impl Day15 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (start_a, start_b) = parser::u32()
            .with_prefix("Generator A starts with ")
            .with_suffix(parser::eol())
            .then(parser::u32().with_prefix("Generator B starts with "))
            .parse_complete(input)?;

        let mutex = Mutex::default();
        let next_index = AtomicU32::new(0);
        let part2_a_count = AtomicU32::new(0);
        let part2_b_count = AtomicU32::new(0);

        multithreading::worker_pool(|| {
            Self::values_worker(
                start_a as u64,
                start_b as u64,
                &mutex,
                &next_index,
                &part2_a_count,
                &part2_b_count,
            );
        });

        Ok(Self {
            batches: mutex.into_inner().unwrap(),
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.batches
            .range(0..PART1_PAIRS)
            .map(|(_, BatchResults { part1_matches, .. })| part1_matches)
            .sum()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let a_values = self
            .batches
            .values()
            .flat_map(|BatchResults { part2_a_values, .. }| part2_a_values.iter().copied());
        let b_values = self
            .batches
            .values()
            .flat_map(|BatchResults { part2_b_values, .. }| part2_b_values.iter().copied());

        a_values
            .zip(b_values)
            .take(PART2_PAIRS as usize)
            .filter(|&(a, b)| a == b)
            .count() as u32
    }

    fn values_worker(
        start_a: u64,
        start_b: u64,
        mutex: &Mutex<BTreeMap<u32, BatchResults>>,
        next_index: &AtomicU32,
        part2_a_count: &AtomicU32,
        part2_b_count: &AtomicU32,
    ) {
        loop {
            let start_index = next_index.fetch_add(BATCH_SIZE, Ordering::AcqRel);
            let part2_a_finished = part2_a_count.load(Ordering::Acquire) >= PART2_PAIRS;
            let part2_b_finished = part2_b_count.load(Ordering::Acquire) >= PART2_PAIRS;
            if start_index >= PART1_PAIRS && part2_a_finished && part2_b_finished {
                break;
            }

            let mut part1_matches = 0;
            let mut part2_a_values = Vec::with_capacity(if part2_a_finished { 0 } else { 65536 });
            let mut part2_b_values = Vec::with_capacity(if part2_b_finished { 0 } else { 32768 });

            let mut a = start_a * mod_pow(FACTOR_A, start_index as u64, MODULUS);
            let mut b = start_b * mod_pow(FACTOR_B, start_index as u64, MODULUS);
            for _ in 0..BATCH_SIZE {
                a = (a * FACTOR_A) % MODULUS;
                b = (b * FACTOR_B) % MODULUS;

                if a as u16 == b as u16 {
                    part1_matches += 1;
                }
                if !part2_a_finished && a.is_multiple_of(4) {
                    part2_a_values.push(a as u16);
                }
                if !part2_b_finished && b.is_multiple_of(8) {
                    part2_b_values.push(b as u16);
                }
            }

            part2_a_count.fetch_add(part2_a_values.len() as u32, Ordering::AcqRel);
            part2_b_count.fetch_add(part2_b_values.len() as u32, Ordering::AcqRel);

            let mut batches_guard = mutex.lock().unwrap();
            batches_guard.insert(
                start_index,
                BatchResults {
                    part1_matches,
                    part2_a_values,
                    part2_b_values,
                },
            );
        }
    }
}

examples!(Day15 -> (u32, u32) [
    {input: "Generator A starts with 65\nGenerator B starts with 8921", part1: 588, part2: 309},
]);
