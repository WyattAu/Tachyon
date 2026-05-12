//! Custom assertion helpers for testing
//!
//! Provides specialized assertion macros and functions for Tachyon testing.
//!
//! Note: `assert_ok` and `assert_err` macros are defined in `test_utils.rs`
//! to avoid duplicate macro definitions. Use `assert_some`, `assert_none`,
//! `assert_eq_unordered`, `assert_contains`, `assert_not_contains`,
//! `assert_in_range`, and `assert_timeout` from this module.

/// Assert that an option is Some and return the inner value
#[macro_export]
macro_rules! assert_some {
    ($expr:expr) => {
        match $expr {
            Some(val) => val,
            None => panic!("Expected Some, got None"),
        }
    };
    ($expr:expr, $msg:expr) => {
        match $expr {
            Some(val) => val,
            None => panic!("{}", $msg),
        }
    };
}

/// Assert that an option is None
#[macro_export]
macro_rules! assert_none {
    ($expr:expr) => {
        match $expr {
            Some(val) => panic!("Expected None, got Some: {:?}", val),
            None => (),
        }
    };
    ($expr:expr, $msg:expr) => {
        match $expr {
            Some(val) => panic!("{}: {:?}", $msg, val),
            None => (),
        }
    };
}

/// Assert that two collections are equal regardless of order
#[macro_export]
macro_rules! assert_eq_unordered {
    ($left:expr, $right:expr) => {{
        use std::collections::HashSet;
        let left_set: HashSet<_> = $left.into_iter().collect();
        let right_set: HashSet<_> = $right.into_iter().collect();
        assert_eq!(left_set, right_set);
    }};
}

/// Assert that a string contains a substring
#[macro_export]
macro_rules! assert_contains {
    ($haystack:expr, $needle:expr) => {{
        let haystack: &str = &$haystack;
        let needle: &str = &$needle;
        assert!(
            haystack.contains(needle),
            "String {:?} does not contain {:?}",
            haystack,
            needle
        );
    }};
}

/// Assert that a string does not contain a substring
#[macro_export]
macro_rules! assert_not_contains {
    ($haystack:expr, $needle:expr) => {{
        let haystack: &str = &$haystack;
        let needle: &str = &$needle;
        assert!(
            !haystack.contains(needle),
            "String {:?} should not contain {:?}",
            haystack,
            needle
        );
    }};
}

/// Assert that a value is within a range
#[macro_export]
macro_rules! assert_in_range {
    ($value:expr, $min:expr, $max:expr) => {
        assert!(
            $value >= $min && $value <= $max,
            "Value {} is not in range [{}, {}]",
            $value,
            $min,
            $max
        )
    };
}

/// Assert that a future completes within a timeout
#[macro_export]
macro_rules! assert_timeout {
    ($future:expr, $duration:expr) => {{
        use tokio::time::{Duration, error::Elapsed, timeout};
        match timeout($duration, $future).await {
            Ok(result) => result,
            Err(Elapsed { .. }) => panic!("Operation timed out after {:?}", $duration),
        }
    }};
}

/// Utility function to compare floating point numbers with tolerance
pub fn assert_almost_eq(a: f64, b: f64, epsilon: f64) {
    let diff = (a - b).abs();
    assert!(
        diff <= epsilon,
        "Values {} and {} differ by {} (epsilon: {})",
        a,
        b,
        diff,
        epsilon
    );
}

/// Utility function to assert collection length
pub fn assert_len<T>(collection: &[T], expected_len: usize) {
    assert_eq!(
        collection.len(),
        expected_len,
        "Expected length {}, got {}",
        expected_len,
        collection.len()
    );
}
