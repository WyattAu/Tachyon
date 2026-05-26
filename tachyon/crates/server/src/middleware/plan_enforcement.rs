//! Plan enforcement middleware for multi-tenant billing.
//!
//! Checks plan limits before allowing mutations. Returns 402 if the
//! organization's current usage exceeds their plan tier.

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use tachyon_database::DatabasePool;
use tracing::warn;

#[derive(Debug, Clone)]
pub struct PlanLimits {
    pub max_docs: Option<i64>,
    pub max_members: Option<i64>,
    pub max_storage_bytes: Option<i64>,
}

impl PlanLimits {
    pub fn for_plan(plan: &str) -> Self {
        match plan {
            "free" => Self {
                max_docs: Some(100),
                max_members: Some(3),
                max_storage_bytes: Some(50_000_000),
            },
            "pro" => Self {
                max_docs: Some(10_000),
                max_members: Some(25),
                max_storage_bytes: Some(5_000_000_000),
            },
            "team" => Self {
                max_docs: Some(100_000),
                max_members: Some(100),
                max_storage_bytes: Some(50_000_000_000),
            },
            _ => Self {
                max_docs: None,
                max_members: None,
                max_storage_bytes: None,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanLimitExceededResponse {
    pub error: String,
    pub message: String,
    pub limit: Option<i64>,
    pub current: Option<i64>,
    pub upgrade_url: String,
}

#[derive(Debug, Clone)]
pub struct PlanEnforcementState {
    pub pool: DatabasePool,
}

pub async fn check_plan_limit(
    State(state): State<PlanEnforcementState>,
    metric: &'static str,
    request: Request,
    next: Next,
) -> Response {
    let org_id = extract_org_id(&request);

    let Some(org_id) = org_id else {
        return (
            StatusCode::BAD_REQUEST,
            Json(PlanLimitExceededResponse {
                error: "missing_org_id".to_string(),
                message: "Organization ID is required for plan enforcement".to_string(),
                limit: None,
                current: None,
                upgrade_url: "/billing/plans".to_string(),
            }),
        )
            .into_response();
    };

    let plan = match fetch_org_plan(&state.pool, &org_id).await {
        Ok(p) => p,
        Err(e) => {
            warn!(
                "Plan enforcement: failed to fetch plan for org {}: {}",
                org_id, e
            );
            return next.run(request).await;
        }
    };

    let limits = PlanLimits::for_plan(&plan);
    let limit_value = match metric {
        "docs" => limits.max_docs,
        "members" => limits.max_members,
        "storage" => limits.max_storage_bytes,
        _ => None,
    };

    let Some(limit_value) = limit_value else {
        return next.run(request).await;
    };

    let current = match fetch_current_usage(&state.pool, &org_id, metric).await {
        Ok(v) => v,
        Err(e) => {
            warn!(
                "Plan enforcement: failed to fetch usage for org {}: {}",
                org_id, e
            );
            return next.run(request).await;
        }
    };

    if current >= limit_value {
        return (
            StatusCode::PAYMENT_REQUIRED,
            Json(PlanLimitExceededResponse {
                error: "plan_limit_exceeded".to_string(),
                message: format!(
                    "Your plan's {} limit has been reached ({}). Please upgrade.",
                    metric, limit_value
                ),
                limit: Some(limit_value),
                current: Some(current),
                upgrade_url: "/billing/plans".to_string(),
            }),
        )
            .into_response();
    }

    next.run(request).await
}

fn extract_org_id(request: &Request) -> Option<String> {
    request
        .headers()
        .get("x-org-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

async fn fetch_org_plan(pool: &DatabasePool, org_id: &str) -> Result<String, String> {
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| format!("pool acquire: {}", e))?;
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT plan FROM subscriptions WHERE organization_id = $1 AND status = 'active' ORDER BY created_at DESC LIMIT 1",
    )
    .bind(org_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| format!("query: {}", e))?;
    Ok(row.map(|(p,)| p).unwrap_or_else(|| "free".to_string()))
}

async fn fetch_current_usage(
    pool: &DatabasePool,
    org_id: &str,
    metric: &str,
) -> Result<i64, String> {
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| format!("pool acquire: {}", e))?;
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT SUM(value) FROM usage_records WHERE org_id = $1 AND metric_type = $2 AND recorded_at >= NOW() - INTERVAL '30 days'",
    )
    .bind(org_id)
    .bind(metric)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| format!("query: {}", e))?;
    Ok(row.map(|(v,)| v).unwrap_or(0))
}
