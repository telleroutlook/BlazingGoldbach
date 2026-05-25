use std::env;
use std::time::Instant;
use blazing_goldbach::{Config, solve};
use blazing_goldbach::construct::{primorial, product_even};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  extreme primorial <k>           Product of first k primes");
        eprintln!("  extreme range <lo> <hi>         2 * product of primes in [lo, hi]");
        std::process::exit(1);
    }

    let (n, factors) = match args[1].as_str() {
        "primorial" => {
            let k: usize = args.get(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(50);
            let (n, f) = primorial(k);
            println!("Primorial P_{} ({} digits)", k, n.to_string().len());
            (n, f)
        }
        "range" => {
            let lo: u64 = args.get(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(1_000_000);
            let hi: u64 = args.get(3)
                .and_then(|s| s.parse().ok())
                .unwrap_or(lo + 1000);
            let (n, f) = product_even(lo, hi);
            println!("2 * product of primes in [{}, {}] ({} digits)", lo, hi, n.to_string().len());
            (n, f)
        }
        _ => {
            eprintln!("Unknown mode: {}. Use 'primorial' or 'range'.", args[1]);
            std::process::exit(1);
        }
    };

    assert!(n.is_even(), "Constructed N must be even");
    assert!(n >= 6, "Constructed N must be >= 6");

    let max_factor = *factors.iter().max().unwrap_or(&2);
    let cfg = Config::for_primorial(max_factor);
    println!("Config: judge_bound={}, screen_bound={}, strategy={}",
        cfg.sieve_judge_bound,
        cfg.sieve_screen_bound,
        match &cfg.modulo_strategy {
            blazing_goldbach::ModuloStrategy::Bignum => "Bignum".to_string(),
            blazing_goldbach::ModuloStrategy::Incremental { factors: f } =>
                format!("Incremental({} factors)", f.len()),
        }
    );

    let start = Instant::now();
    match solve(&n, &cfg) {
        Some(result) => {
            let elapsed = start.elapsed();
            println!("p = {}", result.p);
            println!("q = {}", result.q);
            println!("p_min digits: {}", result.p.to_string().len());
            println!("Verification: {} + {} = {}", result.p, result.q, n);
            println!("BPSW attempts: {}", result.attempts);
            if result.candidates_in_segment > 0 {
                println!(
                    "Sieve: {}/{} survivors ({:.1}% eliminated)",
                    result.survivors_in_segment,
                    result.candidates_in_segment,
                    result.sieve_elimination_rate() * 100.0
                );
            }
            println!("Time: {:.3} ms", elapsed.as_secs_f64() * 1000.0);
        }
        None => {
            eprintln!("No Goldbach partition found within search range");
        }
    }
}
