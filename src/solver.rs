use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use rug::{Assign, Integer, integer::IsPrime};

use crate::primes::primes_up_to;
use crate::sieve::{double_sieve, segmented_sieve};

/// Result of a Goldbach partition search.
#[derive(Debug)]
pub struct GoldbachResult {
    pub p: Integer,
    pub q: Integer,
    pub attempts: u64,
}

/// Check if `n` is (probably) prime using GMP's BPSW + Miller-Rabin.
fn is_prime(n: &Integer) -> bool {
    n.is_probably_prime(25) != IsPrime::No
}

/// Parallel race: find any survivor p where N-p is prime.
/// `counter` tracks actual BPSW invocations (accurate despite find_any short-circuit).
fn parallel_race(n: &Integer, survivors: &[u64], counter: &AtomicU64) -> Option<u64> {
    survivors
        .par_iter()
        .find_any(|&&p| {
            counter.fetch_add(1, Ordering::Relaxed);
            let target = Integer::from(n - p);
            is_prime(&target)
        })
        .copied()
}

/// Solve Goldbach partition for even integer N >= 6.
pub fn solve(n: &Integer) -> Option<GoldbachResult> {
    assert!(*n >= 6u32, "N must be >= 6");
    assert!(n.is_even(), "N must be even");

    // Step 0: try p=2
    let mut attempts: u64 = 1;
    let n_minus_2 = Integer::from(n - 2u32);
    if is_prime(&n_minus_2) {
        return Some(GoldbachResult {
            p: Integer::from(2u32),
            q: n_minus_2,
            attempts,
        });
    }

    // Step 1: serial check first ~20 odd primes before the sieve.
    // This handles small N (where N-p can equal a sieve prime) and
    // covers the common case where p_min is tiny.
    let early_primes = [3u64, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73];
    for &p in &early_primes {
        attempts += 1;
        let target = Integer::from(n - p);
        if is_prime(&target) {
            return Some(GoldbachResult {
                p: Integer::from(p),
                q: target,
                attempts,
            });
        }
    }

    // Sieve boundary R
    let r_bound: u64 = 100_000;
    let small_primes = primes_up_to(r_bound);

    // Precompute N mod r for each small prime
    let rem: Vec<u64> = if let Some(nval) = n.to_u64() {
        small_primes.iter().map(|&sp| nval % sp).collect()
    } else {
        let mut sp_int = Integer::new();
        let mut remainder = Integer::new();
        small_primes
            .iter()
            .map(|&sp| {
                sp_int.assign(sp);
                remainder.assign(n % &sp_int);
                remainder.to_u64().unwrap()
            })
            .collect()
    };

    let n_u64 = n.to_u64();

    let mut base: u64 = 79; // start after the early primes we already checked
    let mut b: usize = 4096;

    loop {
        let is_p_prime = segmented_sieve(base, b, &small_primes);
        let nmp_comp = double_sieve(base, b, &small_primes, &rem);

        let mut survivors: Vec<u64> = Vec::new();
        for i in 0..b {
            let p = base + i as u64;
            if is_p_prime[i] == 1 && nmp_comp[i] == 0 {
                if let Some(nval) = n_u64 {
                    if p >= nval {
                        continue;
                    }
                }
                survivors.push(p);
            }
        }

        if !survivors.is_empty() {
            let serial_count = survivors.len().min(2);
            for &p in &survivors[..serial_count] {
                attempts += 1;
                let target = Integer::from(n - p);
                if is_prime(&target) && is_prime(&Integer::from(p)) {
                    return Some(GoldbachResult {
                        p: Integer::from(p),
                        q: target,
                        attempts,
                    });
                }
            }

            if survivors.len() > serial_count {
                let rest = &survivors[serial_count..];
                let counter = AtomicU64::new(0);
                if let Some(p) = parallel_race(n, rest, &counter) {
                    attempts += counter.into_inner();
                    let p_int = Integer::from(p);
                    assert!(is_prime(&p_int), "sieve incorrectly marked composite {p} as prime");
                    let q = Integer::from(n - &p_int);
                    return Some(GoldbachResult {
                        p: p_int,
                        q,
                        attempts,
                    });
                }
                attempts += counter.into_inner();
            }
        }

        base += b as u64;
        b *= 2;
        // Cap b so segmented sieve stays correct (primes ≤ R must cover √(base+b))
        let r_sq = r_bound * r_bound;
        b = b.min(r_sq.saturating_sub(base) as usize);
        if b == 0 {
            break None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_goldbach() {
        let test_cases: Vec<u32> = vec![6, 8, 10, 100, 1000, 10000];
        for n_val in test_cases {
            let n = Integer::from(n_val);
            let result = solve(&n).unwrap_or_else(|| panic!("Failed for N={}", n_val));
            assert_eq!(
                Integer::from(&result.p + &result.q),
                n,
                "p+q != N for N={}",
                n_val
            );
            assert!(is_prime(&result.p), "p={} is not prime", result.p);
            assert!(is_prime(&result.q), "q={} is not prime", result.q);
        }
    }
}
