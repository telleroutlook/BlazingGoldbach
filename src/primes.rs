use std::vec::Vec;

/// Sieve of Eratosthenes: return all primes up to `limit`.
pub fn primes_up_to(limit: u64) -> Vec<u64> {
    if limit < 2 {
        return Vec::new();
    }
    let mut is_prime = vec![true; (limit + 1) as usize];
    is_prime[0] = false;
    is_prime[1] = false;
    let mut i = 2u64;
    while i * i <= limit {
        if is_prime[i as usize] {
            let mut j = i * i;
            while j <= limit {
                is_prime[j as usize] = false;
                j += i;
            }
        }
        i += 1;
    }
    (2..=limit).filter(|&i| is_prime[i as usize]).collect()
}
