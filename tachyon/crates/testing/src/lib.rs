//! Tachyon Testing Infrastructure
//!
//! This crate provides comprehensive testing infrastructure for the Tachyon project,
//! including unit tests, integration tests, fuzzing tests, and performance benchmarks.
//!
//! ## Modules
//!
//! - [`unit`] - Unit test modules for individual components
//! - [`integration`] - Integration test modules for end-to-end workflows
//! - [`fuzz`] - Fuzzing test modules for security and robustness
//! - [`benchmarks`] - Performance benchmark modules



// Public module exports
pub mod unit;
pub mod integration;
pub mod fuzz;
pub mod benchmarks;

// Common test utilities and helpers
pub mod common;

// Re-export common testing dependencies for convenience
pub use proptest;
pub use mockall;
pub use serial_test;
pub use tokio_test;
pub use wiremock;

/// Test configuration constants
pub mod config {
    /// Default test timeout in seconds
    pub const DEFAULT_TEST_TIMEOUT: u64 = 30;

    /// Database test container port
    pub const DB_TEST_PORT: u16 = 5432;

    /// Number of fuzzing iterations for short runs
    pub const SHORT_FUZZ_ITERATIONS: usize = 10_000;

    /// Number of fuzzing iterations for long runs
    pub const LONG_FUZZ_ITERATIONS: usize = 1_000_000;

    /// Benchmark sample size
    pub const BENCHMARK_SAMPLE_SIZE: usize = 100;

    /// Benchmark warmup time in milliseconds
    pub const BENCHMARK_WARMUP_MS: u64 = 1000;

    /// Benchmark measurement time in milliseconds
    pub const BENCHMARK_MEASUREMENT_MS: u64 = 5000;
}

/// Test result type for custom test assertions
pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

/// Common test setup trait for test fixtures
pub trait TestSetup {
    /// Setup the test fixture
    fn setup() -> Self;

    /// Teardown the test fixture
    fn teardown(self);
}
