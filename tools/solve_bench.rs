// Benchmark runner for Goldbach solver.
// Run with: cargo run --release --bin bench
// (Criterion version commented out pending network access for dependency download)

use std::time::Instant;
use rug::Integer;
use blazing_goldbach::{Config, solve};
use blazing_goldbach::construct::{primorial, random_even};

fn main() {
    let mut rng_state: u64 = 42;
    let mut rng = || {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        rng_state
    };

    let cases: Vec<(&str, Integer, Config)> = vec![
        ("random_20d", "12345678901234567890".parse().unwrap(), Config::default()),
        ("random_50d", {
            let (n, _) = random_even(50, &mut rng);
            n
        }, Config::default()),
        ("random_100d", {
            let (n, _) = random_even(100, &mut rng);
            n
        }, Config::default()),
        ("primorial_20", {
            let (n, _) = primorial(20);
            n
        }, Config::for_primorial(71)),
        ("primorial_30", {
            let (n, _) = primorial(30);
            n
        }, Config::for_primorial(113)),
    ];

    println!("{:<20} {:>10} {:>10} {:>12} {:>10} {:>8}",
        "Case", "Digits", "Attempts", "Time(ms)", "Sieve%", "p_min_d");
    println!("{}", "-".repeat(72));

    for (name, n, cfg) in &cases {
        let digits = n.to_string().len();
        let start = Instant::now();
        let result = solve(n, cfg);
        let elapsed = start.elapsed();
        let ms = elapsed.as_secs_f64() * 1000.0;

        match result {
            Some(r) => {
                let sieve_pct = if r.candidates_in_segment > 0 {
                    format!("{:.1}%", r.sieve_elimination_rate() * 100.0)
                } else {
                    "N/A".to_string()
                };
                println!("{:<20} {:>10} {:>10} {:>12.3} {:>10} {:>8}",
                    name, digits, r.attempts, ms, sieve_pct, r.p.to_string().len());
            }
            None => {
                println!("{:<20} {:>10} {:>10} {:>12.3} {:>10} {:>8}",
                    name, digits, "-", ms, "-", "FAIL");
            }
        }
    }
}
