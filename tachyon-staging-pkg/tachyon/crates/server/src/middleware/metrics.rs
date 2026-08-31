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

    pub fn record_request(&self, duration_ms: u64, status: u16) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.total_request_duration_ms
            .fetch_add(duration_ms, Ordering::Relaxed);
        if status < 400 {
            self.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }

        // Mirror into the `metrics` facade so /metrics/prometheus is populated.
        metrics::counter!("tachyon_requests_total").increment(1);
        metrics::histogram!("tachyon_request_duration_seconds")
            .record(duration_ms as f64 / 1000.0);
        if status >= 400 {
            metrics::counter!("tachyon_requests_failed").increment(1);
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
