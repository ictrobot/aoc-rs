use std::env::args;
use std::hint::black_box;
use std::io::Write;
use std::time::{Duration, Instant};
use utils::multiversion_test;

#[expect(clippy::print_stdout)]
fn main() {
    let mut block_sizes = args()
        .skip(1)
        .filter(|x| x != "--bench")
        .map(|x| x.parse())
        .collect::<Result<Vec<usize>, _>>()
        .expect("invalid block size(s)");
    if block_sizes.is_empty() {
        block_sizes.extend([16, 64, 256, 1024, 4096, 16384]);
    }
    block_sizes.sort_unstable();
    let max_block_size = block_sizes.last().unwrap();

    println!("               │ MD5 throughput (MiB/s) by block size (B)");
    print!("              ");
    for &length in &block_sizes {
        print!(" │ {length:<6}");
    }
    println!();
    print!("───────────────");
    for _ in &block_sizes {
        print!("┼────────");
    }
    println!();

    multiversion_test! {
        use {utils::simd::*, utils::md5::*};

        {
            print!("{SIMD_BACKEND:9}  {:3}", U32Vector::LANES);

            #[expect(clippy::cast_possible_truncation)]
            let input = (0..(max_block_size * U32Vector::LANES))
                .map(|x| x as u8)
                .collect::<Vec<_>>();

            for &block_size in &block_sizes {
                let input = &input[..block_size * U32Vector::LANES];

                let start = Instant::now();
                let mut iterations = 0;
                let bytes_sec = loop {
                    for _ in 0..1000 {
                        let _ = black_box(hash(black_box(input)));
                        iterations += 1;
                    }

                    let elapsed = start.elapsed();
                    if elapsed > Duration::from_secs(1) {
                        #[expect(clippy::cast_precision_loss)]
                        break (iterations * input.len()) as f64 / elapsed.as_secs_f64();
                    }
                };

                print!(" │ {:6.0}", (bytes_sec / 1024.0 / 1024.0).round());
                let _ = std::io::stdout().flush();
            }

            println!()
        }
    }
}
