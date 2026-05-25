use std::env;
use std::time::Instant;
use blazing_goldbach::{Config, solve};
use blazing_goldbach::construct::random_even;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: sample <digits> [count] [seed]");
        eprintln!("  Generate <count> random even numbers of <digits> digits,");
        eprintln!("  solve each, and output CSV to stdout.");
        eprintln!("\nExample: sample 100 50 42 > results.csv");
        std::process::exit(1);
    }

    let digits: usize = args[1].parse().unwrap_or(100);
    let count: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(10);
    let seed: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(42);

    let mut rng_state = seed;
    let mut rng = || {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        rng_state
    };

    println!("digits,n_prefix,p_min,p_min_digits,attempts,sieve_elim_pct,time_ms");
    let cfg = Config::default();

    for i in 0..count {
        let (n, _) = random_even(digits, &mut rng);
        let n_str = n.to_string();
        let n_prefix = &n_str[..20.min(n_str.len())];

        let start = Instant::now();
        match solve(&n, &cfg) {
            Some(result) => {
                let elapsed = start.elapsed();
                let ms = elapsed.as_secs_f64() * 1000.0;
                let sieve_pct = result.sieve_elimination_rate() * 100.0;
                println!(
                    "{},{},{},{},{},{:.1},{:.3}",
                    digits,
                    n_prefix,
                    result.p,
                    result.p.to_string().len(),
                    result.attempts,
                    sieve_pct,
                    ms
                );
            }
            None => {
                eprintln!("Sample {}/{}: FAILED for N starting with {}", i + 1, count, n_prefix);
            }
        }

        if (i + 1) % 10 == 0 {
            eprintln!("Progress: {}/{}", i + 1, count);
        }
    }
}
