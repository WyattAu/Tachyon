use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

const DEFAULT_THRESHOLD_MS: u64 = 100;
const MAX_QUERY_LOG_LENGTH: usize = 200;

#[derive(Debug)]
struct SlowQueryLoggerInner {
    threshold_ms: u64,
    slow_count: AtomicU64,
    total_query_time_ns: AtomicU64,
    total_queries: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct SlowQueryLogger {
    inner: Arc<SlowQueryLoggerInner>,
}

impl SlowQueryLogger {
    pub fn new(threshold_ms: u64) -> Self {
        Self {
            inner: Arc::new(SlowQueryLoggerInner {
                threshold_ms,
                slow_count: AtomicU64::new(0),
                total_query_time_ns: AtomicU64::new(0),
                total_queries: AtomicU64::new(0),
            }),
        }
    }

    pub fn from_env() -> Self {
        let threshold_ms = std::env::var("TACHYON_SLOW_QUERY_THRESHOLD_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_THRESHOLD_MS);
        Self::new(threshold_ms)
    }

    pub fn record_query(&self, query: &str, duration: Duration, location: Option<&str>) {
        let duration_ms = duration.as_millis() as u64;
        self.inner.total_queries.fetch_add(1, Ordering::Relaxed);
        self.inner
            .total_query_time_ns
            .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);

        if duration_ms >= self.inner.threshold_ms {
            self.inner.slow_count.fetch_add(1, Ordering::Relaxed);
            let display_query = if query.len() > MAX_QUERY_LOG_LENGTH {
                format!("{}...", &query[..MAX_QUERY_LOG_LENGTH])
            } else {
                query.to_string()
            };
            tracing::warn!(
                slow_query_duration_ms = duration_ms,
                slow_query_threshold_ms = self.inner.threshold_ms,
                slow_query_text = %display_query,
                slow_query_location = ?location,
                "Slow query detected"
            );
        }
    }

    pub fn slow_count(&self) -> u64 {
        self.inner.slow_count.load(Ordering::Relaxed)
    }

    pub fn total_queries(&self) -> u64 {
        self.inner.total_queries.load(Ordering::Relaxed)
    }

    pub fn average_query_time_ms(&self) -> f64 {
        let total = self.inner.total_queries.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            let total_ns = self.inner.total_query_time_ns.load(Ordering::Relaxed);
            (total_ns as f64) / (total as f64) / 1_000_000.0
        }
    }

    pub fn threshold_ms(&self) -> u64 {
        self.inner.threshold_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_default_threshold_from_env() {
        unsafe {
            std::env::remove_var("TACHYON_SLOW_QUERY_THRESHOLD_MS");
        }
        let logger = SlowQueryLogger::from_env();
        assert_eq!(logger.threshold_ms(), 100);
    }

    #[test]
    #[serial]
    fn test_custom_threshold_from_env() {
        unsafe {
            std::env::set_var("TACHYON_SLOW_QUERY_THRESHOLD_MS", "250");
        }
        let logger = SlowQueryLogger::from_env();
        assert_eq!(logger.threshold_ms(), 250);
        unsafe {
            std::env::remove_var("TACHYON_SLOW_QUERY_THRESHOLD_MS");
        }
    }

    #[test]
    fn test_record_query_below_threshold() {
        let logger = SlowQueryLogger::new(1000);
        logger.record_query("SELECT 1", Duration::from_millis(50), None);
        assert_eq!(logger.slow_count(), 0);
        assert_eq!(logger.total_queries(), 1);
    }

    #[test]
    fn test_record_query_at_threshold() {
        let logger = SlowQueryLogger::new(100);
        logger.record_query("SELECT 1", Duration::from_millis(100), None);
        assert_eq!(logger.slow_count(), 1);
        assert_eq!(logger.total_queries(), 1);
    }

    #[test]
    fn test_record_query_above_threshold() {
        let logger = SlowQueryLogger::new(100);
        logger.record_query(
            "SELECT * FROM users WHERE id = $1",
            Duration::from_millis(250),
            Some("user_repo::find"),
        );
        assert_eq!(logger.slow_count(), 1);
        assert_eq!(logger.total_queries(), 1);
    }

    #[test]
    fn test_average_query_time() {
        let logger = SlowQueryLogger::new(1000);
        logger.record_query("SELECT 1", Duration::from_millis(10), None);
        logger.record_query("SELECT 2", Duration::from_millis(30), None);
        assert_eq!(logger.total_queries(), 2);
        assert!((logger.average_query_time_ms() - 20.0).abs() < 0.1);
    }

    #[test]
    fn test_average_query_time_no_queries() {
        let logger = SlowQueryLogger::new(100);
        assert_eq!(logger.average_query_time_ms(), 0.0);
    }

    #[test]
    fn test_query_truncation_in_log() {
        let logger = SlowQueryLogger::new(0);
        let long_query = "x".repeat(300);
        logger.record_query(&long_query, Duration::from_millis(1), None);
        assert_eq!(logger.slow_count(), 1);
    }

    #[test]
    fn test_clone_shares_state() {
        let logger = SlowQueryLogger::new(100);
        let logger2 = logger.clone();
        logger.record_query("SELECT 1", Duration::from_millis(200), None);
        assert_eq!(logger2.slow_count(), 1);
        assert_eq!(logger2.total_queries(), 1);
        assert!((logger2.average_query_time_ms() - 200.0).abs() < 0.1);
    }

    #[test]
    #[serial]
    fn test_invalid_env_falls_back_to_default() {
        unsafe {
            std::env::set_var("TACHYON_SLOW_QUERY_THRESHOLD_MS", "not_a_number");
        }
        let logger = SlowQueryLogger::from_env();
        assert_eq!(logger.threshold_ms(), 100);
        unsafe {
            std::env::remove_var("TACHYON_SLOW_QUERY_THRESHOLD_MS");
        }
    }

    #[test]
    fn test_zero_threshold_logs_everything() {
        let logger = SlowQueryLogger::new(0);
        logger.record_query("SELECT 1", Duration::from_nanos(1), None);
        assert_eq!(logger.slow_count(), 1);
        assert_eq!(logger.total_queries(), 1);
    }
}
