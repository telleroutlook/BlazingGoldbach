/// Segmented sieve of Eratosthenes: mark primes in [base, base+B).
/// Returns a bitvec (as Vec<u8>) where `buf[i] == 1` means `base+i` is prime.
/// `small_primes` should contain all primes up to sqrt(base+B).
pub fn segmented_sieve(base: u64, b: usize, small_primes: &[u64]) -> Vec<u8> {
    let mut buf = vec![1u8; b];
    for &r in small_primes {
        if r * r > base + b as u64 {
            break;
        }
        // Find first multiple of r >= base
        let start = if base == 0 {
            r * r
        } else {
            let rem = base % r;
            if rem == 0 { base } else { base + (r - rem) }
        };
        let start = if start < r * r { r * r } else { start };
        if start >= base + b as u64 {
            continue;
        }
        let offset = (start - base) as usize;
        let mut j = offset;
        while j < b {
            buf[j] = 0;
            j += r as usize;
        }
    }
    // 0 and 1 are not prime
    if base == 0 {
        if b > 0 { buf[0] = 0; }
        if b > 1 { buf[1] = 0; }
    }
    buf
}

/// Double-sieve: for each small prime r, mark positions where (N-p) % r == 0.
/// `rem[r]` = N mod r (precomputed once). Returns a bitvec where 1 = N-p is divisible by some small prime.
pub fn double_sieve(base: u64, b: usize, small_primes: &[u64], rem: &[u64]) -> Vec<u8> {
    let mut nmp_comp = vec![0u8; b];
    for (idx, &r) in small_primes.iter().enumerate() {
        let nr = rem[idx];
        // N-p ≡ 0 (mod r)  <=>  p ≡ N (mod r)
        // Find first i in [0, B) where (base + i) mod r == N mod r
        let base_mod = base % r;
        let target = nr % r;
        let offset = if target >= base_mod {
            (target - base_mod) as usize
        } else {
            (target + r - base_mod) as usize
        };
        let mut i = offset;
        while i < b {
            nmp_comp[i] = 1;
            i += r as usize;
        }
    }
    nmp_comp
}
