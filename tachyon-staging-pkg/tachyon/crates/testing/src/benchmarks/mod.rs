// Performance benchmark modules
//
// Each benchmark is a standalone criterion binary defined in Cargo.toml
// under [[bench]]. They are not library modules.
//
// Run: cargo bench --bench <name>

// This file exists to satisfy Rust's module system for the benchmarks/ directory.
// No public re-exports are needed since each benchmark is its own binary target.
