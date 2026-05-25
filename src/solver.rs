use rayon::prelude::*;
use rug::{Integer, integer::IsPrime};

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
/// GMP >= 6.2.0 uses Baillie-PSW internally; reps=25 gives negligible error probability.
fn is_prime(n: &Integer) -> bool {
    n.is_probably_prime(25) != IsPrime::No
}

/// Parallel race: find any survivor p where N-p is prime.
/// Uses rayon::find_any for cooperative cancellation on first hit.
fn parallel_race(n: &Integer, survivors: &[u64]) -> Option<u64> {
    survivors
        .par_iter()
        .find_any(|&&p| {
            let target = Integer::from(n - p);
            is_prime(&target)
        })
        .copied()
}

/// Solve Goldbach partition for even integer N >= 6.
///
/// Algorithm (from goldbach_strategy_optimized_v3.md):
///   1. Try p=2 first (special case).
///   2. Generate small primes up to R (sieve boundary).
///   3. Precompute N mod r for each small prime r.
///   4. Adaptive-segment double-sieve: sieve p AND N-p simultaneously.
///   5. For survivors: serial-probe first 1-2, then parallel race with rayon.
pub fn solve(n: &Integer) -> Option<GoldbachResult> {
    assert!(n >= 6u32, "N must be >= 6");
    assert!(n.is_even(), "N must be even");

    // Step 0: try p=2
    let n_minus_2 = Integer::from(n - 2u32);
    if is_prime(&n_minus_2) {
        return Some(GoldbachResult {
            p: Integer::from(2u32),
            q: n_minus_2,
            attempts: 1,
        });
    }

    // Sieve boundary R: aggressively sieve small primes up to 100_000
    let r: u64 = 100_000;
    let small_primes = primes_up_to(r);

    // Precompute N mod r for each small prime (the only big-integer operations in sieve phase)
    let rem: Vec<u64> = small_primes
        .iter()
        .map(|&sp| {
            let modulo = n.rem_u64(sp);
            modulo
        })
        .collect();

    let mut base: u64 = 3;
    let mut b: usize = 4096; // start small — p_min is usually tiny
    let mut total_attempts: u64 = 0;

    loop {
        // 1) Segmented sieve: mark primes in [base, base+B)
        let is_p_prime = segmented_sieve(base, b, &small_primes);

        // 2) Double sieve: mark where N-p is composite (divisible by small primes)
        let nmp_comp = double_sieve(base, b, &small_primes, &rem);

        // 3) Collect survivors: both p is prime AND N-p passes small-prime filter
        let mut survivors: Vec<u64> = Vec::new();
        for i in 0..b {
            if is_p_prime[i] == 1 && nmp_comp[i] == 0 {
                survivors.push(base + i as u64);
            }
        }

        if !survivors.is_empty() {
            // Serial probe first 1-2 candidates (common case: hits immediately)
            let serial_count = survivors.len().min(2);
            for &p in &survivors[..serial_count] {
                total_attempts += 1;
                let target = Integer::from(n - p);
                if is_prime(&target) {
                    return Some(GoldbachResult {
                        p: Integer::from(p),
                        q: target,
                        attempts: total_attempts,
                    });
                }
            }

            // Rare case: first few missed, parallel race the rest
            if survivors.len() > serial_count {
                let rest = &survivors[serial_count..];
                total_attempts += rest.len() as u64;
                if let Some(p) = parallel_race(n, rest) {
                    return Some(GoldbachResult {
                        p: Integer::from(p),
                        q: Integer::from(n - p),
                        attempts: total_attempts,
                    });
                }
            }
        }

        // Expand interval and continue (extremely rare for large N)
        base += b as u64;
        b *= 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_goldbach() {
        let test_cases = vec![
            (6u32, true),
            (8u32, true),
            (10u32, true),
            (100u32, true),
            (1000u32, true),
            (10000u32, true),
        ];
        for (n, should_find) in test_cases {
            let n = Integer::from(n);
            let result = solve(&n);
            assert_eq!(result.is_some(), should_find, "Failed for N={}", n);
            if let Some(r) = result {
                assert_eq!(&r.p + &r.q, n, "p+q != N");
                assert!(is_prime(&r.p), "p={} is not prime", r.p);
                assert!(is_prime(&r.q), "q={} is not prime", r.q);
            }
        }
    }
}
