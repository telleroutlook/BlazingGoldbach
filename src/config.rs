/// Configuration for the Goldbach partition solver.
///
/// All parameters that were previously hardcoded in solver.rs are now
/// configurable here. `Config::default()` reproduces the original behavior.
#[derive(Debug, Clone)]
pub struct Config {
    /// Primes up to this bound are used by `segmented_sieve` for primality
    /// testing. Must be >= sqrt(expected p_min upper bound).
    /// Invariant: `sieve_judge_bound >= sqrt(max possible base + b)`.
    pub sieve_judge_bound: u64,
    /// Primes up to this bound are used by `double_sieve` for eliminating
    /// composite N-p candidates. Can be much larger than judge_bound.
    /// More primes here means fewer BPSW calls but more sieve work.
    pub sieve_screen_bound: u64,
    /// Segment sieve start position (default 79, skipping early primes already
    /// checked serially).
    pub base_start: u64,
    /// Initial segment length.
    pub segment_init: usize,
    /// Segment length growth factor per iteration.
    pub segment_growth: usize,
    /// Number of survivors to check serially before entering parallel race.
    pub serial_head: usize,
    /// BPSW Miller-Rabin rounds.
    pub mr_rounds: u32,
    /// Strategy for computing N mod r remainders.
    pub modulo_strategy: ModuloStrategy,
}

/// Strategy for computing N mod r for each small prime r.
#[derive(Debug, Clone)]
pub enum ModuloStrategy {
    /// Direct bignum division for each r (works for any N).
    Bignum,
    /// Incremental modular arithmetic using known factor decomposition.
    /// For each small prime r not in the factor list, compute product of
    /// factors mod r via u64 arithmetic — no bignum division needed.
    Incremental { factors: Vec<u64> },
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sieve_judge_bound: 100_000,
            sieve_screen_bound: 100_000,
            base_start: 79,
            segment_init: 4096,
            segment_growth: 2,
            serial_head: 2,
            mr_rounds: 25,
            modulo_strategy: ModuloStrategy::Bignum,
        }
    }
}

impl Config {
    /// Config tuned for primorial-constructed N with max factor p_k.
    /// Sets sieve_screen_bound well above p_k so double_sieve has enough
    /// primes in (p_k, R] to eliminate candidates.
    pub fn for_primorial(max_factor: u64) -> Self {
        let sieve_screen_bound = (max_factor * 4).max(10_000_000);
        Self {
            sieve_screen_bound,
            modulo_strategy: ModuloStrategy::Incremental {
                factors: crate::primes::primes_up_to(max_factor),
            },
            ..Self::default()
        }
    }
}
