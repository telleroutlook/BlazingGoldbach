use rug::{Assign, Integer};
use crate::config::ModuloStrategy;

/// Compute N mod r for each small prime r, using the configured strategy.
pub fn compute_remainders(n: &Integer, small_primes: &[u64], strategy: &ModuloStrategy) -> Vec<u64> {
    match strategy {
        ModuloStrategy::Bignum => compute_bignum(n, small_primes),
        ModuloStrategy::Incremental { factors } => compute_incremental(n, small_primes, factors),
    }
}

/// Direct bignum modular arithmetic (original path).
fn compute_bignum(n: &Integer, small_primes: &[u64]) -> Vec<u64> {
    if let Some(nval) = n.to_u64() {
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
    }
}

/// Incremental modular arithmetic using known factors.
///
/// For each small prime r:
/// - If r divides one of the factors (i.e., r is in the factor list), N mod r = 0.
/// - Otherwise, N mod r = (product of factors) mod r, computed entirely in u64.
fn compute_incremental(_n: &Integer, small_primes: &[u64], factors: &[u64]) -> Vec<u64> {
    use std::collections::HashSet;
    let factor_set: HashSet<u64> = factors.iter().copied().collect();

    small_primes
        .iter()
        .map(|&r| {
            if factor_set.contains(&r) {
                0
            } else {
                // Compute product of factors mod r in u64
                let mut acc: u64 = 1;
                for &f in factors {
                    acc = (acc as u128 * f as u128 % r as u128) as u64;
                }
                acc
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primes::primes_up_to;

    #[test]
    fn test_incremental_matches_bignum() {
        // N = 2 * 3 * 5 * 7 * 11 * 13 = 30030
        let factors = vec![2, 3, 5, 7, 11, 13];
        let n = Integer::from(factors.iter().product::<u64>());
        let small_primes = primes_up_to(1000);

        let rem_bignum = compute_bignum(&n, &small_primes);
        let rem_incr = compute_incremental(&n, &small_primes, &factors);

        assert_eq!(rem_bignum, rem_incr, "incremental must match bignum");
    }

    #[test]
    fn test_incremental_primorial_large() {
        // Test with a primorial that overflows u64: first 20 primes
        let factors = primes_up_to(71); // 20th prime
        let mut n = Integer::from(1u64);
        for &f in &factors {
            n = Integer::from(&n * f);
        }
        let small_primes = primes_up_to(500);

        let rem_bignum = compute_bignum(&n, &small_primes);
        let rem_incr = compute_incremental(&n, &small_primes, &factors);

        assert_eq!(rem_bignum, rem_incr, "incremental must match bignum for large primorial");
    }
}
