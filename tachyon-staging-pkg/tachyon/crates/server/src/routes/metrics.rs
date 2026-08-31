use axum::extract::State;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
pub struct MetricsState {
    pub metrics: Arc<crate::middleware::metrics::RequestMetrics>,
    pub start_time: Instant,
}

pub async fn prometheus_metrics(State(state): State<MetricsState>) -> String {
    let snapshot = state.metrics.snapshot();

    format!(
        r#"# HELP tachyon_requests_total Total number of HTTP requests
# TYPE tachyon_requests_total counter
tachyon_requests_total {}

# HELP tachyon_requests_successful Total successful requests
# TYPE tachyon_requests_successful counter
tachyon_requests_successful {}

# HELP tachyon_requests_failed Total failed requests
# TYPE tachyon_requests_failed counter
tachyon_requests_failed {}

# HELP tachyon_request_duration_avg_ms Average request duration in milliseconds
# TYPE tachyon_request_duration_avg_ms gauge
tachyon_request_duration_avg_ms {}

# HELP tachyon_uptime_seconds Server uptime in seconds
# TYPE tachyon_uptime_seconds gauge
tachyon_uptime_seconds {}

# HELP tachyon_version Server version
# TYPE tachyon_version gauge
tachyon_version_info{{version="{}"}} 1
"#,
        snapshot.total_requests,
        snapshot.successful_requests,
        snapshot.failed_requests,
        snapshot.avg_duration_ms,
        state.start_time.elapsed().as_secs(),
        env!("CARGO_PKG_VERSION"),
    )
}
