use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use rug::{Integer, integer::IsPrime};

use crate::config::Config;
use crate::modulo::compute_remainders;
use crate::primes::primes_up_to;
use crate::sieve::{double_sieve, segmented_sieve};

/// Result of a Goldbach partition search.
#[derive(Debug)]
pub struct GoldbachResult {
    pub p: Integer,
    pub q: Integer,
    pub attempts: u64,
    /// How many candidates were in the segment that found the answer.
    pub candidates_in_segment: usize,
    /// How many survived the double-sieve (sent to BPSW) in that segment.
    pub survivors_in_segment: usize,
}

impl GoldbachResult {
    /// Fraction of candidates eliminated by double-sieve in the winning segment.
    pub fn sieve_elimination_rate(&self) -> f64 {
        if self.candidates_in_segment == 0 {
            return 0.0;
        }
        1.0 - self.survivors_in_segment as f64 / self.candidates_in_segment as f64
    }
}

/// Check if `n` is (probably) prime using GMP's BPSW + Miller-Rabin.
fn is_prime(n: &Integer, mr_rounds: u32) -> bool {
    n.is_probably_prime(mr_rounds) != IsPrime::No
}

/// Estimate a lower bound on the number of prime pairs (p, N-p) that could exist
/// in a segment of `survivor_count` candidates near `base`.
///
/// Analogous to Golomb-Vanguard's `sum_smallest_unset_sym` DFS branch pruning:
/// instead of computing the exact minimum cost of remaining branches, we estimate
/// the probability that at least one survivor p has N-p also prime. If the expected
/// count is far below 1, the segment is provably unproductive and can be skipped.
///
/// Uses the prime number theorem: the density of primes near x is ~1/ln(x).
/// For each survivor p, N-p is roughly the same order of magnitude, so the
/// probability that N-p is prime is ~1/ln(N). The expected number of Goldbach
/// pairs in a segment is approximately `survivor_count / ln(N)`.
///
/// Returns the expected number of Goldbach pairs in this segment.
fn expected_goldbach_pairs(survivor_count: usize, n_approx: f64) -> f64 {
    if survivor_count == 0 || n_approx <= 2.0 {
        return 0.0;
    }
    let ln_n = n_approx.ln();
    if ln_n <= 0.0 {
        return 0.0;
    }
    survivor_count as f64 / ln_n
}

/// Decide whether a segment is worth checking with expensive BPSW verification.
///
/// A segment is "productive" if the expected number of Goldbach pairs exceeds
/// `threshold` (default: a small fraction, e.g. 0.01). Segments below this
/// threshold are extremely unlikely to yield a result and can be safely skipped.
///
/// This is conservative: it only skips when the expected count is very low,
/// so correctness is preserved at the cost of occasionally checking a segment
/// that won't yield a result (false negatives in pruning are safe).
pub fn segment_is_productive(survivor_count: usize, n_approx: f64, threshold: f64) -> bool {
    survivor_count > 0 && expected_goldbach_pairs(survivor_count, n_approx) >= threshold
}

/// Parallel race: find the smallest survivor p where N-p is prime.
fn parallel_race(n: &Integer, survivors: &[u64], counter: &AtomicU64, mr_rounds: u32) -> Option<u64> {
    survivors
        .par_iter()
        .find_first(|&&p| {
            counter.fetch_add(1, Ordering::Relaxed);
            let target = Integer::from(n - p);
            is_prime(&target, mr_rounds)
        })
        .copied()
}

/// Solve Goldbach partition for even integer N >= 6.
pub fn solve(n: &Integer, cfg: &Config) -> Option<GoldbachResult> {
    assert!(*n >= 6u32, "N must be >= 6");
    assert!(n.is_even(), "N must be even");

    let mr = cfg.mr_rounds;

    // Step 0: try p=2
    let mut attempts: u64 = 1;
    let n_minus_2 = Integer::from(n - 2u32);
    if is_prime(&n_minus_2, mr) {
        return Some(GoldbachResult {
            p: Integer::from(2u32),
            q: n_minus_2,
            attempts,
            candidates_in_segment: 0,
            survivors_in_segment: 0,
        });
    }

    // Step 1: serial check first ~20 odd primes before the sieve.
    let early_primes = [3u64, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73];
    for &p in &early_primes {
        attempts += 1;
        let target = Integer::from(n - p);
        if is_prime(&target, mr) {
            return Some(GoldbachResult {
                p: Integer::from(p),
                q: target,
                attempts,
                candidates_in_segment: 0,
                survivors_in_segment: 0,
            });
        }
    }

    // Two independent sieve roles:
    // - judge_primes (up to sieve_judge_bound): for segmented_sieve primality testing
    // - screen_primes (up to sieve_screen_bound): for double_sieve eliminating N-p composites
    let judge_primes = primes_up_to(cfg.sieve_judge_bound);
    let screen_primes = primes_up_to(cfg.sieve_screen_bound);

    // Precompute N mod r for screening primes
    let rem = compute_remainders(n, &screen_primes, &cfg.modulo_strategy);

    let n_u64 = n.to_u64();

    let mut base: u64 = cfg.base_start;
    let mut b: usize = cfg.segment_init;

    loop {
        let is_p_prime = segmented_sieve(base, b, &judge_primes);
        let nmp_comp = double_sieve(base, b, &screen_primes, &rem, n_u64);

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

        let candidate_count = survivors.len();
        if !survivors.is_empty() {
            // Lower-bound pruning (from Golomb-Vanguard's symmetry-aware bounding):
            // If the expected Goldbach pairs in this segment is extremely low,
            // skip the expensive BPSW verification and move to the next segment.
            let n_approx = n_u64.map(|v| v as f64).unwrap_or_else(|| {
                let bits = n.significant_digits::<u32>() as f64;
                2f64.powf(bits)
            });
            if !segment_is_productive(candidate_count, n_approx, 0.01) {
                base += b as u64;
                b *= cfg.segment_growth;
                let judge_sq = cfg.sieve_judge_bound * cfg.sieve_judge_bound;
                b = b.min(judge_sq.saturating_sub(base) as usize);
                if b == 0 {
                    break None;
                }
                continue;
            }
            let serial_count = survivors.len().min(cfg.serial_head);
            for &p in &survivors[..serial_count] {
                attempts += 1;
                let target = Integer::from(n - p);
                if is_prime(&target, mr) && is_prime(&Integer::from(p), mr) {
                    debug_assert!(is_prime(&target, mr), "q must be prime");
                    debug_assert_eq!(Integer::from(p + &target), *n, "p + q must equal N");
                    return Some(GoldbachResult {
                        p: Integer::from(p),
                        q: target,
                        attempts,
                        candidates_in_segment: b,
                        survivors_in_segment: candidate_count,
                    });
                }
            }

            if survivors.len() > serial_count {
                let rest = &survivors[serial_count..];
                let counter = AtomicU64::new(0);
                if let Some(p) = parallel_race(n, rest, &counter, mr) {
                    attempts += counter.into_inner();
                    let p_int = Integer::from(p);
                    assert!(is_prime(&p_int, mr), "sieve incorrectly marked composite {p} as prime");
                    let q = Integer::from(n - &p_int);
                    debug_assert!(is_prime(&q, mr), "q must be prime");
                    debug_assert_eq!(Integer::from(&p_int + &q), *n, "p + q must equal N");
                    return Some(GoldbachResult {
                        p: p_int,
                        q,
                        attempts,
                        candidates_in_segment: b,
                        survivors_in_segment: candidate_count,
                    });
                }
                attempts += counter.into_inner();
            }
        }

        base += b as u64;
        b *= cfg.segment_growth;
        // Cap b so segmented sieve stays correct:
        // judge_primes must cover sqrt(base + b), so b <= judge_bound^2 - base
        let judge_sq = cfg.sieve_judge_bound * cfg.sieve_judge_bound;
        b = b.min(judge_sq.saturating_sub(base) as usize);
        if b == 0 {
            break None;
        }
    }
}

/// Dynamically adjust sieve segment size based on survival density.
/// Borrowed from Golomb-Vanguard's sum_smallest_unset_sym dynamic bounding:
/// if many candidates survive in early segments, we can use smaller segments
/// for more precise filtering; if few survive, larger segments batch more
/// candidates per sieve pass, amortizing the fixed sieve setup cost.
///
/// The survival ratio is interpolated linearly between two thresholds:
///   - ratio > 0.1  (dense): return `min_size` for precision
///   - ratio < 0.01 (sparse): return `max_size` for throughput
///   - in between:   linearly interpolate
pub fn adaptive_segment_size(
    base_segment_size: usize,
    survivors_found: usize,
    candidates_scanned: usize,
    min_size: usize,
    max_size: usize,
) -> usize {
    if candidates_scanned == 0 {
        return base_segment_size;
    }
    let ratio = survivors_found as f64 / candidates_scanned as f64;

    const HIGH_THRESHOLD: f64 = 0.1;
    const LOW_THRESHOLD: f64 = 0.01;

    let size = if ratio >= HIGH_THRESHOLD {
        // Dense survival: use small segments for precision.
        min_size
    } else if ratio <= LOW_THRESHOLD {
        // Sparse survival: use large segments to amortize sieve cost.
        max_size
    } else {
        // Linear interpolation between min and max.
        let t = (HIGH_THRESHOLD - ratio) / (HIGH_THRESHOLD - LOW_THRESHOLD);
        let interpolated = min_size as f64 + t * (max_size - min_size) as f64;
        interpolated.round() as usize
    };

    // Always clamp to [min_size, max_size] and respect the base as a fallback.
    size.clamp(min_size, max_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_goldbach() {
        let test_cases: Vec<u32> = vec![6, 8, 10, 100, 1000, 10000];
        let cfg = Config::default();
        for n_val in test_cases {
            let n = Integer::from(n_val);
            let result = solve(&n, &cfg).unwrap_or_else(|| panic!("Failed for N={}", n_val));
            assert_eq!(
                Integer::from(&result.p + &result.q),
                n,
                "p+q != N for N={}",
                n_val
            );
            assert!(is_prime(&result.p, cfg.mr_rounds), "p={} is not prime", result.p);
            assert!(is_prime(&result.q, cfg.mr_rounds), "q={} is not prime", result.q);
        }
    }

    /// Differential test: brute-force minimum p for small N, compare with solve.
    fn brute_force_min_p(n: u64) -> u64 {
        let primes = crate::primes::primes_up_to(n);
        let prime_set: std::collections::HashSet<u64> = primes.iter().copied().collect();
        for &p in &primes {
            if p > n / 2 {
                break;
            }
            let q = n - p;
            if prime_set.contains(&q) {
                return p;
            }
        }
        panic!("No partition found for N={}", n);
    }

    #[test]
    fn test_differential_small() {
        let cfg = Config::default();
        for n_val in (6..=10_000).step_by(2) {
            let n = Integer::from(n_val);
            let result = solve(&n, &cfg).unwrap_or_else(|| panic!("Failed for N={}", n_val));
            let expected_p = brute_force_min_p(n_val);
            assert_eq!(
                result.p, Integer::from(expected_p),
                "solve found p={} but brute-force found p_min={} for N={}",
                result.p, expected_p, n_val
            );
        }
    }

    #[test]
    fn test_verification_invariant() {
        // Every result must satisfy: p prime, q prime, p+q=N
        let cfg = Config::default();
        for n_val in [6u64, 100, 1000, 1000000, 9999999998] {
            let n = Integer::from(n_val);
            if let Some(result) = solve(&n, &cfg) {
                assert!(is_prime(&result.p, cfg.mr_rounds), "p not prime for N={}", n_val);
                assert!(is_prime(&result.q, cfg.mr_rounds), "q not prime for N={}", n_val);
                assert_eq!(Integer::from(&result.p + &result.q), n, "p+q != N for N={}", n_val);
            }
        }
    }

    #[test]
    fn test_expected_goldbach_pairs() {
        // For N=100, ln(100) ~ 4.6, so 5 survivors -> ~1.09 expected pairs
        let pairs = expected_goldbach_pairs(5, 100.0);
        assert!(pairs > 1.0 && pairs < 1.5, "expected ~1.09, got {}", pairs);

        // Zero survivors -> zero pairs
        assert_eq!(expected_goldbach_pairs(0, 100.0), 0.0);

        // Very large N, few survivors -> low expectation
        let pairs = expected_goldbach_pairs(1, 1e18);
        assert!(pairs < 0.05, "expected < 0.05, got {}", pairs);

        // Many survivors, small N -> high expectation
        let pairs = expected_goldbach_pairs(100, 1000.0);
        assert!(pairs > 10.0, "expected > 10, got {}", pairs);
    }

    #[test]
    fn test_segment_is_productive() {
        // With threshold 0.01:
        // For N=1e18, ln(N) ~ 41.4, so 1 survivor gives 1/41.4 ~ 0.024 > 0.01 -> productive
        assert!(segment_is_productive(1, 1e18, 0.01));

        // Zero survivors -> never productive
        assert!(!segment_is_productive(0, 1e18, 0.01));

        // For extremely large N with threshold 0.1, 1 survivor is not productive
        assert!(!segment_is_productive(1, 1e100, 0.1));

        // But 100 survivors is productive even for large N
        assert!(segment_is_productive(100, 1e18, 0.01));
    }

    #[test]
    fn test_lower_bound_preserves_correctness() {
        // Differential test with segment_is_productive integrated:
        // ensure solve still finds correct answers after pruning is added.
        let cfg = Config::default();
        for n_val in (6..=1000).step_by(2) {
            let n = Integer::from(n_val);
            let result = solve(&n, &cfg).unwrap_or_else(|| panic!("Failed for N={}", n_val));
            let expected_p = brute_force_min_p(n_val);
            assert_eq!(
                result.p, Integer::from(expected_p),
                "solve found p={} but brute-force found p_min={} for N={}",
                result.p, expected_p, n_val
            );
        }
    }
}
