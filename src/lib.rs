pub mod config;
pub mod construct;
pub mod modulo;
pub mod primes;
pub mod sieve;
pub mod solver;

pub use config::{Config, ModuloStrategy};
pub use solver::{GoldbachResult, solve};
