use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct RequestMetrics {
    pub total_requests: AtomicU64,
    pub successful_requests: AtomicU64,
    pub failed_requests: AtomicU64,
    pub total_request_duration_ms: AtomicU64,
}

impl RequestMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a request with per-route labels.
    ///
    /// `route` should already be normalized (dynamic ID segments replaced,
    /// see `normalize_route` in `request_tracing.rs`) to keep label
    /// cardinality bounded. `status` is bucketed to its class (2xx/4xx/5xx).
    pub fn record_request(&self, duration_ms: u64, status: u16, method: &str, route: &str) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.total_request_duration_ms
            .fetch_add(duration_ms, Ordering::Relaxed);
        if status < 400 {
            self.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }

        // Mirror into the `metrics` facade so /metrics/prometheus is populated.
        let status_class: &'static str = match status {
            200..=299 => "2xx",
            300..=399 => "3xx",
            400..=499 => "4xx",
            _ => "5xx",
        };
        metrics::counter!("tachyon_requests_total", "method" => method.to_string(), "route" => route.to_string(), "status" => status_class).increment(1);
        metrics::histogram!("tachyon_request_duration_seconds", "method" => method.to_string(), "route" => route.to_string()).record(duration_ms as f64 / 1000.0);
        if status >= 400 {
            metrics::counter!("tachyon_requests_failed", "route" => route.to_string(), "status" => status_class).increment(1);
        }
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        let total = self.total_requests.load(Ordering::Relaxed);
        MetricsSnapshot {
            total_requests: total,
            successful_requests: self.successful_requests.load(Ordering::Relaxed),
            failed_requests: self.failed_requests.load(Ordering::Relaxed),
            avg_duration_ms: self
                .total_request_duration_ms
                .load(Ordering::Relaxed)
                .checked_div(total)
                .unwrap_or(0),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_duration_ms: u64,
}
