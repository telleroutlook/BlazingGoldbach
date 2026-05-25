use rug::Integer;
use crate::primes::primes_up_to;

/// Compute the primorial P_k: product of the first k primes.
/// Returns (N, factor_list) where N is even (includes 2).
pub fn primorial(k: usize) -> (Integer, Vec<u64>) {
    assert!(k >= 1, "k must be >= 1");
    // Generate more than enough primes to get k of them
    let estimate = if k < 10 { 30 } else { (k as f64 * ((k as f64).ln() + 1.5)) as u64 };
    let all_primes = primes_up_to(estimate.max(30));
    let factors: Vec<u64> = all_primes.into_iter().take(k).collect();
    let mut n = Integer::from(1u64);
    for &f in &factors {
        n = Integer::from(&n * f);
    }
    (n, factors)
}

/// Construct 2 * product of all primes in [lo, hi].
/// Returns (N, factor_list including 2).
pub fn product_even(lo: u64, hi: u64) -> (Integer, Vec<u64>) {
    assert!(lo >= 2 && hi >= lo, "invalid range");
    let primes = primes_up_to(hi);
    let selected: Vec<u64> = primes.into_iter().filter(|&p| p >= lo).collect();
    let mut n = Integer::from(2u64);
    for &p in &selected {
        n = Integer::from(&n * p);
    }
    let mut factors = vec![2u64];
    factors.extend(selected);
    (n, factors)
}

/// Generate a random even number with the given number of digits.
/// Returns (N, empty factor list) — no factorization info available.
pub fn random_even(digits: usize, rng: &mut impl FnMut() -> u64) -> (Integer, Vec<u64>) {
    let mut s = String::with_capacity(digits);
    // First digit: 1-9
    let d = (rng() % 9 + 1) as u32;
    s.push(char::from_digit(d, 10).unwrap());

    for _ in 1..digits {
        let d = (rng() % 10) as u32;
        s.push(char::from_digit(d, 10).unwrap());
    }

    // Ensure last digit is even
    let last_even = ['0', '2', '4', '6', '8'];
    let last = last_even[(rng() % 5) as usize];
    s.pop();
    s.push(last);

    (s.parse::<Integer>().unwrap(), Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primorial_small() {
        let (n, factors) = primorial(4);
        assert_eq!(n, Integer::from(2 * 3 * 5 * 7));
        assert_eq!(factors, vec![2, 3, 5, 7]);
    }

    #[test]
    fn test_primorial_is_even() {
        for k in 1..=10 {
            let (n, _) = primorial(k);
            assert!(n.is_even(), "primorial({}) must be even", k);
        }
    }

    #[test]
    fn test_product_even() {
        let (n, factors) = product_even(3, 11);
        // primes in [3, 11] are 3, 5, 7, 11
        assert_eq!(n, Integer::from(2u64 * 3 * 5 * 7 * 11));
        assert_eq!(factors[0], 2);
    }

    #[test]
    fn test_random_even() {
        let mut call_count = 0u64;
        let mut rng = || { call_count += 1; call_count };
        let (n, factors) = random_even(50, &mut rng);
        assert!(n.is_even());
        assert!(n.to_string().len() == 50);
        assert!(factors.is_empty());
    }
}
