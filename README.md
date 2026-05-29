<div align="center">

# BlazingGoldbach

**High-Performance Goldbach Conjecture Solver**

[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Performance](https://img.shields.io/badge/100_digit-~5ms-green.svg)]()

Solve `p + q = N` for even numbers from single digits to **thousands of digits** — with adaptive sieves, parallel BPSW, and sub-5ms benchmarks at 100 digits.

</div>

---

## How It Works

BlazingGoldbach finds prime pairs `(p, q)` such that `p + q = N` for any even `N >= 6` using a multi-stage pipeline:

```
Input N (even, >= 6)
        │
        ▼
  Quick Small-Prime Check (primes up to 73)
        │ miss
        ▼
  Adaptive Double Sieve (segmented, parallel)
        │ surviving candidates
        ▼
  Parallel BPSW Primality Testing (rayon)
        │ first prime found
        ▼
  Return (p, q = N - p)
```

**Key algorithms:**
- **Adaptive double sieve** — segmented sieve with dynamic segment sizing based on survival density
- **Early elimination** — checks small primes before launching the full sieve
- **Parallel racing** — BPSW tests surviving candidates across all CPU cores
- **Modulo optimization** — efficient `N mod r` for candidate elimination

---

## Performance

| Input Size | Typical Time | BPSW Attempts | Sieve Elimination |
|:-----------|:-------------|:--------------|:------------------|
| 100 digits | 1–5 ms | 20–80 | 97–99.2% |
| 200 digits | 10–50 ms | — | >98% |
| Primorial(50) | seconds | — | — |

Solution `p_min` is typically small (2–4 digits for 100-digit inputs), which the early-check exploits.

---

## Quick Start

```bash
git clone https://github.com/telleroutlook/BlazingGoldbach.git
cd BlazingGoldbach
cargo build --release
```

```bash
# Solve for a specific number of digits
./target/release/goldbach 100

# Solve for a random 200-digit even number
./target/release/goldbach --random 200

# Solve extreme cases: primorial (product of first k primes)
./target/release/extreme primorial 50

# Solve extreme cases: 2 × product of primes in a range
./target/release/extreme range 1000000 1001000

# Run benchmarks
cargo run --release --bin bench
```

---

## Binaries

| Binary | Purpose |
|:-------|:--------|
| `goldbach` | Main solver for any even number |
| `extreme` | Specialized solver for primorial and product-even numbers |
| `bench` | Benchmark runner with configurable scenarios |
| `sample` | Sample runner for testing and validation |

---

## Configuration

| Parameter | Default | Description |
|:----------|:--------|:------------|
| `sieve_judge_bound` | 100,000 | Segmented sieve upper bound |
| `sieve_screen_bound` | 100,000 | Double sieve screening bound |
| `segment_init` | 4,096 | Initial segment size |
| `segment_growth` | 2× | Segment growth factor |
| `serial_head` | 2 | Serial candidates before parallel |
| `mr_rounds` | 25 | Miller-Rabin rounds for BPSW |

---

## Repository Structure

```
BlazingGoldbach/
├── src/
│   ├── lib.rs          # Library exports
│   ├── solver.rs       # Core solving algorithm
│   ├── sieve.rs        # Segmented sieve implementations
│   ├── primes.rs       # Prime generation utilities
│   ├── config.rs       # Configuration parameters
│   ├── construct.rs    # Number generation (random, primorial, range)
│   ├── modulo.rs       # Efficient modulo computation
│   └── bin/            # Binary executables
├── docs/               # Documentation
├── tools/              # Benchmark runner
└── results.csv         # Benchmark results data
```

---

## License

Apache License 2.0
