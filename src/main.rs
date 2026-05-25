mod primes;
mod sieve;
mod solver;

use std::env;
use std::time::Instant;
use rug::Integer;
use solver::solve;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: blazing-goldbach <even_number>");
        eprintln!("  Finds primes p, q such that p + q = N (Goldbach partition)");
        eprintln!("\nExamples:");
        eprintln!("  blazing-goldbach 100");
        eprintln!("  blazing-goldbach 12345678901234567890");
        eprintln!("  blazing-goldbach --random 1000   (random 1000-digit even number)");
        std::process::exit(1);
    }

    let n = if args[1] == "--random" {
        let digits: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(100);
        generate_random_even(digits)
    } else {
        match args[1].parse::<Integer>() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Error: '{}' is not a valid integer", args[1]);
                std::process::exit(1);
            }
        }
    };

    if n < 6u32 {
        eprintln!("Error: N must be >= 6");
        std::process::exit(1);
    }
    if !n.is_even() {
        eprintln!("Error: N must be even");
        std::process::exit(1);
    }

    let digit_count = n.to_string().len();
    println!("N = {} ({} digits)", n, digit_count);

    let start = Instant::now();
    match solve(&n) {
        Some(result) => {
            let elapsed = start.elapsed();
            println!("p = {}", result.p);
            println!("q = {}", result.q);
            println!("Verification: {} + {} = {}", result.p, result.q, n);
            println!("BPSW attempts: {}", result.attempts);
            println!("Time: {:.3} ms", elapsed.as_secs_f64() * 1000.0);
        }
        None => {
            eprintln!("No Goldbach partition found (this should not happen for N >= 6)");
        }
    }
}

fn generate_random_even(digits: usize) -> Integer {
    use std::time::{SystemTime, UNIX_EPOCH};
    // Simple seeded random for demo purposes
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    let mut state = seed;

    let mut s = String::with_capacity(digits);
    // First digit: 1-9
    state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    s.push(char::from_digit(((state >> 33) % 9 + 1) as u32, 10).unwrap());

    for _ in 1..digits {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        s.push(char::from_digit(((state >> 33) % 10) as u32, 10).unwrap());
    }

    // Ensure last digit is even
    let last_even = ['0', '2', '4', '6', '8'];
    state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    let last_char = last_even[((state >> 33) % 5) as usize];
    s.pop();
    s.push(last_char);

    s.parse::<Integer>().unwrap()
}
