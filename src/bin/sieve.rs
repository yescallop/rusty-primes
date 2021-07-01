use std::time::Instant;

use rusty_primes::sieve::*;

fn main() {
    let start = Instant::now();

    let primes = SegmentedEratosthenes::prime_pi(u32::MAX as usize);

    println!("{} ({:?})", primes, start.elapsed());
}
