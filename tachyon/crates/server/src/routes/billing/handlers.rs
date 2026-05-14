use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use chrono::{DateTime, Utc};
use hmac::Mac;
use subtle::ConstantTimeEq;
use tachyon_database::error::DatabaseError;
use tracing::{info, warn};

use crate::truelayer::TrueLayerError;

use super::types::{
    BillingErrorResponse, BillingState, ChangePlanRequest, ChangePlanResponse,
    CreateMandateRequest, CreatePaymentRequest, CreateSubscriptionRequest, HmacSha256,
    InvoicesResponse, MandateResponse, MandateStatusResponse, PaymentResponse,
    PaymentStatusResponse, Plan, PlanDetails, PlanInfo, PlansResponse, ProrationResult,
    SubscriptionResponse, TransitionType, UsageMetrics, UsageResponse, WebhookPayload,
};

pub fn verify_webhook_signature(payload: &[u8], signature_header: &str, secret: &str) -> bool {
    let sig_part = signature_header
        .strip_prefix("v1=")
        .unwrap_or(signature_header);

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(payload);

    let expected = hex::encode(mac.finalize().into_bytes());

    expected.as_bytes().ct_eq(sig_part.as_bytes()).into()
}

// ============================================================================
// Helper functions
// ============================================================================

pub fn plan_price(plan: &str) -> u64 {
    match plan {
        "free" => 0,
        "pro" => 12_00,
        "team" => 29_00,
        _ => 0,
    }
}

pub fn plan_max_docs(plan: &str) -> usize {
    match plan {
        "free" => 100,
        "pro" => 10_000,
        "team" => 100_000,
        _ => usize::MAX,
    }
}

pub fn plan_max_members(plan: &str) -> usize {
    match plan {
        "free" => 1,
        "pro" => 5,
        "team" => 50,
        _ => usize::MAX,
    }
}

pub fn plan_features(plan: &str) -> Vec<String> {
    match plan {
        "free" => vec![
            "Basic editor".into(),
            "5GB storage".into(),
            "Community support".into(),
        ],
        "pro" => vec![
            "Advanced editor".into(),
            "50GB storage".into(),
            "SSG export".into(),
            "Plugin system".into(),
            "Email support".into(),
        ],
        "team" => vec![
            "Everything in Pro".into(),
            "Collaboration".into(),
            "Admin panel".into(),
            "Audit logs".into(),
            "Priority support".into(),
        ],
        _ => vec![],
    }
}

pub fn truelayer_error_response(e: TrueLayerError) -> (StatusCode, Json<BillingErrorResponse>) {
    match e {
        TrueLayerError::Disabled => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(BillingErrorResponse {
                code: "PAYMENTS_DISABLED".to_string(),
                message: "Payment processing is not enabled".to_string(),
            }),
        ),
        TrueLayerError::ApiError { status, message } => (
            StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY),
            Json(BillingErrorResponse {
                code: "TRUELAYER_API_ERROR".to_string(),
                message,
            }),
        ),
        TrueLayerError::AuthError(msg) => (
            StatusCode::UNAUTHORIZED,
            Json(BillingErrorResponse {
                code: "TRUELAYER_AUTH_ERROR".to_string(),
                message: msg,
            }),
        ),
        TrueLayerError::ConfigError(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(BillingErrorResponse {
                code: "TRUELAYER_CONFIG_ERROR".to_string(),
                message: msg,
            }),
        ),
        TrueLayerError::RequestError(err) => (
            StatusCode::BAD_GATEWAY,
            Json(BillingErrorResponse {
                code: "TRUELAYER_REQUEST_ERROR".to_string(),
                message: err.to_string(),
            }),
        ),
    }
}

pub fn validate_plan_name(plan: &str) -> Result<(), (StatusCode, Json<BillingErrorResponse>)> {
    const VALID_PLANS: &[&str] = &["free", "pro", "team", "enterprise"];
    if !VALID_PLANS.contains(&plan) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(BillingErrorResponse {
                code: "INVALID_PLAN".to_string(),
                message: format!(
                    "Invalid plan '{}'. Must be one of: free, pro, team, enterprise",
                    plan
                ),
            }),
        ));
    }
    Ok(())
}

pub fn validate_plan_transition(
    current: &str,
    new: &str,
) -> Result<TransitionType, (StatusCode, Json<BillingErrorResponse>)> {
    const PLAN_ORDER: &[&str] = &["free", "pro", "team", "enterprise"];
    let current_idx = PLAN_ORDER.iter().position(|&p| p == current).unwrap_or(0);
    let new_idx = PLAN_ORDER.iter().position(|&p| p == new).unwrap_or(0);

    if current == "enterprise" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(BillingErrorResponse {
                code: "ENTERPRISE_CHANGE_REQUIRES_ADMIN".to_string(),
                message: "Enterprise plan changes require admin approval".to_string(),
            }),
        ));
    }

    match new_idx.cmp(&current_idx) {
        std::cmp::Ordering::Greater => Ok(TransitionType::Upgrade),
        std::cmp::Ordering::Less => Ok(TransitionType::Downgrade),
        std::cmp::Ordering::Equal => Err((
            StatusCode::BAD_REQUEST,
            Json(BillingErrorResponse {
                code: "SAME_PLAN".to_string(),
                message: "Already on this plan".to_string(),
            }),
        )),
    }
}

pub fn plan_price_f64(plan: &str) -> f64 {
    plan_price(plan) as f64 / 100.0
}

pub fn calculate_proration(
    current_plan_price: f64,
    new_plan_price: f64,
    billing_start: DateTime<Utc>,
    billing_end: DateTime<Utc>,
    now: DateTime<Utc>,
) -> ProrationResult {
    let total_days = (billing_end - billing_start).num_days();
    let total_days = if total_days <= 0 {
        1.0
    } else {
        total_days as f64
    };
    let days_used = (now - billing_start).num_days();
    let days_used = if days_used < 0 { 0.0 } else { days_used as f64 };
    let days_remaining = (total_days - days_used).max(0.0);

    let current_daily = current_plan_price / total_days;
    let new_daily = new_plan_price / total_days;

    let credit = current_daily * days_remaining;
    let charge = new_daily * days_remaining;

    ProrationResult {
        prorated_amount: charge - credit,
        credit,
        charge,
        days_remaining: days_remaining as u32,
    }
}

// ============================================================================
// Route handlers
// ============================================================================

/// GET /api/v1/billing/plans — List available plans
#[utoipa::path(
    get,
    path = "/billing/plans",
    responses(
        (status = 200, description = "Available plans", body = PlansResponse),
    ),
    tag = "billing",
)]
pub async fn list_plans() -> Json<PlansResponse> {
    let plans = vec![
        PlanInfo {
            name: "free".to_string(),
            price_monthly_cents: Plan::Free.price_monthly(),
            max_documents: Plan::Free.max_documents(),
            max_members: Plan::Free.max_members(),
            features: Plan::Free
                .features()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        },
        PlanInfo {
            name: "pro".to_string(),
            price_monthly_cents: Plan::Pro.price_monthly(),
            max_documents: Plan::Pro.max_documents(),
            max_members: Plan::Pro.max_members(),
            features: Plan::Pro.features().iter().map(|s| s.to_string()).collect(),
        },
        PlanInfo {
            name: "team".to_string(),
            price_monthly_cents: Plan::Team.price_monthly(),
            max_documents: Plan::Team.max_documents(),
            max_members: Plan::Team.max_members(),
            features: Plan::Team
                .features()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        },
        PlanInfo {
            name: "enterprise".to_string(),
            price_monthly_cents: Plan::Enterprise.price_monthly(),
            max_documents: Plan::Enterprise.max_documents(),
            max_members: Plan::Enterprise.max_members(),
            features: Plan::Enterprise
                .features()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        },
    ];
    Json(PlansResponse { plans })
}

/// POST /api/v1/billing/subscriptions — Create a subscription
#[utoipa::path(
    post,
    path = "/billing/subscriptions",
    request_body(content = CreateSubscriptionRequest, description = "Subscription creation request"),
    responses(
        (status = 200, description = "Subscription created", body = SubscriptionResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "billing",
    security(("bearer_auth" = [])),
)]
pub async fn create_subscription(
    State(state): State<BillingState>,
    Json(req): Json<CreateSubscriptionRequest>,
) -> Result<Json<SubscriptionResponse>, (StatusCode, Json<BillingErrorResponse>)> {
    info!(
        "Creating {} subscription for org {}",
        req.plan, req.organization_id
    );

    let repo = tachyon_database::SubscriptionRepository::new(state.pool.clone());
    let sub = repo
        .create(tachyon_database::CreateSubscriptionRequest {
            organization_id: req.organization_id.clone(),
            plan: req.plan.clone(),
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BillingErrorResponse {
                    code: "DB_ERROR".to_string(),
                    message: e.to_string(),
                }),
            )
        })?;

    let plan_details = PlanDetails {
        name: sub.plan.clone(),
        price_monthly_cents: plan_price(&sub.plan),
        max_documents: plan_max_docs(&sub.plan),
        max_members: plan_max_members(&sub.plan),
        features: plan_features(&sub.plan),
    };

    Ok(Json(SubscriptionResponse {
        subscription: sub,
        plan_details,
    }))
}

/// GET /api/v1/billing/subscriptions/{org_id} — Get subscription
#[utoipa::path(
    get,
    path = "/billing/subscriptions/{org_id}",
    params(
        ("org_id" = String, Path, description = "Organization ID"),
    ),
    responses(
        (status = 200, description = "Subscription details", body = SubscriptionResponse),
        (status = 404, description = "No subscription found"),
    ),
    tag = "billing",
    security(("bearer_auth" = [])),
)]
pub async fn get_subscription(
    State(state): State<BillingState>,
    axum::extract::Path(org_id): axum::extract::Path<String>,
) -> Result<Json<SubscriptionResponse>, (StatusCode, Json<BillingErrorResponse>)> {
    let repo = tachyon_database::SubscriptionRepository::new(state.pool.clone());
    let sub = repo.get_by_org(&org_id).await.map_err(|e| {
        if matches!(e, DatabaseError::NotFound { .. }) {
            (
                StatusCode::NOT_FOUND,
                Json(BillingErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: "No subscription found".to_string(),
                }),
            )
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BillingErrorResponse {
                    code: "DB_ERROR".to_string(),
                    message: e.to_string(),
                }),
            )
        }
    })?;

    let plan_details = PlanDetails {
        name: sub.plan.clone(),
        price_monthly_cents: plan_price(&sub.plan),
        max_documents: plan_max_docs(&sub.plan),
        max_members: plan_max_members(&sub.plan),
        features: plan_features(&sub.plan),
    };

    Ok(Json(SubscriptionResponse {
        subscription: sub,
        plan_details,
    }))
}

/// GET /api/v1/billing/invoices/{org_id} — List invoices
#[utoipa::path(
    get,
    path = "/billing/invoices/{org_id}",
    params(
        ("org_id" = String, Path, description = "Organization ID"),
    ),
    responses(
        (status = 200, description = "Invoice list", body = InvoicesResponse),
    ),
    tag = "billing",
    security(("bearer_auth" = [])),
)]
pub async fn list_invoices(
    State(state): State<BillingState>,
    axum::extract::Path(org_id): axum::extract::Path<String>,
) -> Json<InvoicesResponse> {
    let repo = tachyon_database::InvoiceRepository::new(state.pool.clone());
    let invoices = repo.list_by_org(&org_id).await.unwrap_or_default();
    let total = invoices.len();
    Json(InvoicesResponse { invoices, total })
}

/// GET /api/v1/billing/usage/{org_id} — Get usage metrics (real implementation)
#[utoipa::path(
    get,
    path = "/billing/usage/{org_id}",
    params(
        ("org_id" = String, Path, description = "Organization ID"),
    ),
    responses(
        (status = 200, description = "Usage metrics", body = UsageResponse),
    ),
    tag = "billing",
    security(("bearer_auth" = [])),
)]
pub async fn get_usage(
    State(state): State<BillingState>,
    axum::extract::Path(org_id): axum::extract::Path<String>,
) -> Json<UsageResponse> {
    let now = Utc::now();
    let period_start = now - chrono::Duration::days(30);

    let pool = state.pool.inner();

    let documents_total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM documents WHERE organization_id = $1 AND deleted_at IS NULL",
    )
    .bind(&org_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let documents_created: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM documents WHERE organization_id = $1 AND deleted_at IS NULL AND created_at >= $2"
    )
    .bind(&org_id)
    .bind(period_start)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let storage_bytes: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(LENGTH(content::text)), 0) FROM documents WHERE organization_id = $1 AND deleted_at IS NULL"
    )
    .bind(&org_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let members_total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM organization_members WHERE organization_id = $1")
            .bind(&org_id)
            .fetch_one(pool)
            .await
            .unwrap_or(1);

    let plan = sqlx::query_scalar::<_, String>(
        "SELECT plan FROM subscriptions WHERE organization_id = $1 ORDER BY created_at DESC LIMIT 1"
    )
    .bind(&org_id)
    .fetch_one(pool)
    .await
    .ok()
    .map(|p| match p.as_str() {
        "pro" => Plan::Pro,
        "team" => Plan::Team,
        "enterprise" => Plan::Enterprise,
        _ => Plan::Free,
    })
    .unwrap_or(Plan::Free);

    Json(UsageResponse {
        usage: UsageMetrics {
            organization_id: org_id,
            period_start,
            period_end: now,
            documents_created: documents_created as usize,
            documents_total: documents_total as usize,
            members_total: members_total as usize,
            storage_bytes: storage_bytes as u64,
            plan,
        },
    })
}

/// POST /api/v1/billing/subscription/change-plan — Upgrade/downgrade plan
#[utoipa::path(
    post,
    path = "/billing/subscription/change-plan",
    request_body(content = ChangePlanRequest, description = "Plan change request"),
    responses(
        (status = 200, description = "Plan change result", body = ChangePlanResponse),
        (status = 400, description = "Invalid plan"),
        (status = 404, description = "No subscription found"),
        (status = 403, description = "Enterprise change requires admin"),
    ),
    tag = "billing",
    security(("bearer_auth" = [])),
)]
pub async fn change_plan(
    State(state): State<BillingState>,
    Json(req): Json<ChangePlanRequest>,
) -> Result<Json<ChangePlanResponse>, (StatusCode, Json<BillingErrorResponse>)> {
    validate_plan_name(&req.new_plan)?;

    let repo = tachyon_database::SubscriptionRepository::new(state.pool.clone());
    let sub = repo.get_by_org(&req.organization_id).await.map_err(|e| {
        if matches!(e, DatabaseError::NotFound { .. }) {
            (
                StatusCode::NOT_FOUND,
                Json(BillingErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: "No subscription found".to_string(),
                }),
            )
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BillingErrorResponse {
                    code: "DB_ERROR".to_string(),
                    message: e.to_string(),
                }),
            )
        }
    })?;

    let old_plan = sub.plan.clone();
    let transition = validate_plan_transition(&old_plan, &req.new_plan)?;

    let now = Utc::now();
    let proration = calculate_proration(
        plan_price_f64(&old_plan),
        plan_price_f64(&req.new_plan),
        sub.current_period_start,
        sub.current_period_end,
        now,
    );

    match transition {
        TransitionType::Upgrade => {
            let prorated_cents = (proration.prorated_amount * 100.0).round() as i64;

            if let Some(truelayer) = state.truelayer.as_ref() {
                let reference = format!("upgrade-{}-{}", sub.id, uuid::Uuid::new_v4());
                let payment_result = truelayer
                    .create_payment(
                        sub.payment_method_id.as_deref().unwrap_or(""),
                        prorated_cents as u64,
                        &reference,
                    )
                    .await
                    .map_err(truelayer_error_response)?;

                let updated = repo
                    .update(
                        &sub.id,
                        tachyon_database::UpdateSubscriptionRequest {
                            plan: Some(req.new_plan.clone()),
                            status: None,
                            cancel_at_period_end: None,
                            payment_method_id: None,
                        },
                    )
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(BillingErrorResponse {
                                code: "DB_ERROR".to_string(),
                                message: e.to_string(),
                            }),
                        )
                    })?;

                let invoice_repo = tachyon_database::InvoiceRepository::new(state.pool.clone());
                let _ = invoice_repo
                    .create(tachyon_database::CreateInvoiceRequest {
                        subscription_id: updated.id.clone(),
                        organization_id: req.organization_id.clone(),
                        amount_cents: prorated_cents,
                        currency: "GBP".to_string(),
                        description: format!("Upgrade from {} to {}", old_plan, req.new_plan),
                    })
                    .await;

                info!(
                    "Plan upgrade: {} -> {} for org {} (payment {})",
                    old_plan, req.new_plan, req.organization_id, payment_result.id
                );

                Ok(Json(ChangePlanResponse {
                    subscription_id: updated.id,
                    old_plan,
                    new_plan: req.new_plan,
                    status: "pending_payment".to_string(),
                    effective_at: now.to_rfc3339(),
                    prorated_amount: Some(proration.prorated_amount),
                    next_billing_date: Some(sub.current_period_end.to_rfc3339()),
                }))
            } else {
                let updated = repo
                    .update(
                        &sub.id,
                        tachyon_database::UpdateSubscriptionRequest {
                            plan: Some(req.new_plan.clone()),
                            status: None,
                            cancel_at_period_end: None,
                            payment_method_id: None,
                        },
                    )
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(BillingErrorResponse {
                                code: "DB_ERROR".to_string(),
                                message: e.to_string(),
                            }),
                        )
                    })?;

                info!(
                    "Plan upgrade (free): {} -> {} for org {}",
                    old_plan, req.new_plan, req.organization_id
                );

                Ok(Json(ChangePlanResponse {
                    subscription_id: updated.id,
                    old_plan,
                    new_plan: req.new_plan,
                    status: "immediate".to_string(),
                    effective_at: now.to_rfc3339(),
                    prorated_amount: Some(proration.prorated_amount),
                    next_billing_date: Some(sub.current_period_end.to_rfc3339()),
                }))
            }
        }
        TransitionType::Downgrade => {
            info!(
                "Plan downgrade scheduled: {} -> {} for org {} (effective at period end)",
                old_plan, req.new_plan, req.organization_id
            );

            Ok(Json(ChangePlanResponse {
                subscription_id: sub.id,
                old_plan,
                new_plan: req.new_plan,
                status: "scheduled".to_string(),
                effective_at: sub.current_period_end.to_rfc3339(),
                prorated_amount: Some(proration.prorated_amount),
                next_billing_date: Some(sub.current_period_end.to_rfc3339()),
            }))
        }
    }
}

/// POST /api/v1/billing/subscriptions/{org_id}/cancel — Cancel subscription
#[utoipa::path(
    post,
    path = "/billing/subscriptions/{org_id}/cancel",
    params(
        ("org_id" = String, Path, description = "Organization ID"),
    ),
    responses(
        (status = 200, description = "Subscription cancelled", body = serde_json::Value),
        (status = 404, description = "No subscription found"),
    ),
    tag = "billing",
    security(("bearer_auth" = [])),
)]
pub async fn cancel_subscription(
    State(state): State<BillingState>,
    axum::extract::Path(org_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<BillingErrorResponse>)> {
    let repo = tachyon_database::SubscriptionRepository::new(state.pool.clone());
    let sub = repo.get_by_org(&org_id).await.map_err(|e| {
        if matches!(e, DatabaseError::NotFound { .. }) {
            (
                StatusCode::NOT_FOUND,
                Json(BillingErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: "No subscription found".to_string(),
                }),
            )
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BillingErrorResponse {
                    code: "DB_ERROR".to_string(),
                    message: e.to_string(),
                }),
            )
        }
    })?;

    let updated = repo
        .update(
            &sub.id,
            tachyon_database::UpdateSubscriptionRequest {
                cancel_at_period_end: Some(true),
                plan: None,
                status: None,
                payment_method_id: None,
            },
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BillingErrorResponse {
                    code: "DB_ERROR".to_string(),
                    message: e.to_string(),
                }),
            )
        })?;

    Ok(Json(serde_json::json!({
        "success": true,
        "subscription": updated,
        "cancel_at_period_end": updated.cancel_at_period_end,
    })))
}

// ============================================================================
// TrueLayer payment routes
// ============================================================================

/// POST /api/v1/billing/mandates — Create a payment mandate
#[utoipa::path(
    post,
    path = "/billing/mandates",
    request_body(content = CreateMandateRequest, description = "Mandate creation request"),
    responses(
        (status = 200, description = "Mandate created", body = MandateResponse),
        (status = 503, description = "Payments not enabled"),
    ),
    tag = "billing",
    security(("bearer_auth" = [])),
)]
pub async fn create_mandate(
    State(state): State<BillingState>,
    Json(req): Json<CreateMandateRequest>,
) -> Result<Json<MandateResponse>, (StatusCode, Json<BillingErrorResponse>)> {
    let truelayer = state.truelayer.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(BillingErrorResponse {
                code: "PAYMENTS_DISABLED".to_string(),
                message: "Payment processing is not enabled".to_string(),
            }),
        )
    })?;

    let result = truelayer
        .create_payment_mandate(&req.organization_id, &req.return_url)
        .await
        .map_err(truelayer_error_response)?;

    let repo = tachyon_database::SubscriptionRepository::new(state.pool.clone());
    if let Ok(sub) = repo.get_by_org(&req.organization_id).await {
        let _ = repo
            .update(
                &sub.id,
                tachyon_database::UpdateSubscriptionRequest {
                    payment_method_id: Some(result.id.clone()),
                    plan: None,
                    status: None,
                    cancel_at_period_end: None,
                },
            )
            .await;
    }

    info!(
        "Created mandate {} for org {}",
        result.id, req.organization_id
    );

    Ok(Json(MandateResponse {
        mandate_id: result.id,
        authorization_url: result.authorization_url,
        status: result.status,
    }))
}

/// GET /api/v1/billing/mandates/{mandate_id} — Check mandate status
#[utoipa::path(
    get,
    path = "/billing/mandates/{mandate_id}",
    params(
        ("mandate_id" = String, Path, description = "Mandate ID"),
    ),
    responses(
        (status = 200, description = "Mandate status", body = MandateStatusResponse),
        (status = 503, description = "Payments not enabled"),
    ),
    tag = "billing",
    security(("bearer_auth" = [])),
)]
pub async fn get_mandate_status(
    State(state): State<BillingState>,
    axum::extract::Path(mandate_id): axum::extract::Path<String>,
) -> Result<Json<MandateStatusResponse>, (StatusCode, Json<BillingErrorResponse>)> {
    let truelayer = state.truelayer.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(BillingErrorResponse {
                code: "PAYMENTS_DISABLED".to_string(),
                message: "Payment processing is not enabled".to_string(),
            }),
        )
    })?;

    let result = truelayer
        .get_mandate_status(&mandate_id)
        .await
        .map_err(truelayer_error_response)?;

    Ok(Json(MandateStatusResponse {
        mandate_id: result.id,
        status: result.status,
    }))
}

/// POST /api/v1/billing/payments — Create a payment
#[utoipa::path(
    post,
    path = "/billing/payments",
    request_body(content = CreatePaymentRequest, description = "Payment creation request"),
    responses(
        (status = 200, description = "Payment created", body = PaymentResponse),
        (status = 503, description = "Payments not enabled"),
    ),
    tag = "billing",
    security(("bearer_auth" = [])),
)]
pub async fn create_payment(
    State(state): State<BillingState>,
    Json(req): Json<CreatePaymentRequest>,
) -> Result<Json<PaymentResponse>, (StatusCode, Json<BillingErrorResponse>)> {
    let truelayer = state.truelayer.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(BillingErrorResponse {
                code: "PAYMENTS_DISABLED".to_string(),
                message: "Payment processing is not enabled".to_string(),
            }),
        )
    })?;

    let reference = format!("payment-{}", uuid::Uuid::new_v4());
    let result = truelayer
        .create_payment(&req.mandate_id, req.amount_cents, &reference)
        .await
        .map_err(truelayer_error_response)?;

    let invoice_repo = tachyon_database::InvoiceRepository::new(state.pool.clone());
    let sub_repo = tachyon_database::SubscriptionRepository::new(state.pool.clone());

    let subscription_id = sub_repo
        .get_by_org(&req.organization_id)
        .await
        .map(|s| s.id)
        .unwrap_or_default();

    if let Ok(_invoice) = invoice_repo
        .create(tachyon_database::CreateInvoiceRequest {
            subscription_id: subscription_id.clone(),
            organization_id: req.organization_id.clone(),
            amount_cents: req.amount_cents as i64,
            currency: "GBP".to_string(),
            description: format!("Payment {}", result.id),
        })
        .await
    {
        info!("Created invoice for payment {}", result.id);
    }

    info!(
        "Created payment {} for org {} ({} pence)",
        result.id, req.organization_id, req.amount_cents
    );

    Ok(Json(PaymentResponse {
        payment_id: result.id,
        status: result.status,
        amount: result.amount_in_minor,
    }))
}

/// GET /api/v1/billing/payments/{payment_id} — Check payment status
#[utoipa::path(
    get,
    path = "/billing/payments/{payment_id}",
    params(
        ("payment_id" = String, Path, description = "Payment ID"),
    ),
    responses(
        (status = 200, description = "Payment status", body = PaymentStatusResponse),
        (status = 503, description = "Payments not enabled"),
    ),
    tag = "billing",
    security(("bearer_auth" = [])),
)]
pub async fn get_payment_status(
    State(state): State<BillingState>,
    axum::extract::Path(payment_id): axum::extract::Path<String>,
) -> Result<Json<PaymentStatusResponse>, (StatusCode, Json<BillingErrorResponse>)> {
    let truelayer = state.truelayer.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(BillingErrorResponse {
                code: "PAYMENTS_DISABLED".to_string(),
                message: "Payment processing is not enabled".to_string(),
            }),
        )
    })?;

    let result = truelayer
        .get_payment_status(&payment_id)
        .await
        .map_err(truelayer_error_response)?;

    Ok(Json(PaymentStatusResponse {
        payment_id: result.id,
        status: result.status,
    }))
}

/// POST /api/v1/billing/webhook — TrueLayer webhook handler
#[utoipa::path(
    post,
    path = "/billing/webhook",
    request_body(content = WebhookPayload, description = "TrueLayer webhook payload"),
    responses(
        (status = 200, description = "Webhook processed", body = serde_json::Value),
        (status = 400, description = "Invalid payload"),
        (status = 401, description = "Invalid signature"),
    ),
    tag = "billing",
)]
pub async fn webhook_handler(
    State(state): State<BillingState>,
    headers: HeaderMap,
    body: String,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<BillingErrorResponse>)> {
    if let Some(truelayer) = &state.truelayer {
        let secret = truelayer.webhook_secret();
        if !secret.is_empty() {
            let signature = headers
                .get("TrueLayer-Signature")
                .and_then(|v| v.to_str().ok());

            match signature {
                None => {
                    warn!("Webhook received without signature header");
                    return Err((
                        StatusCode::UNAUTHORIZED,
                        Json(BillingErrorResponse {
                            code: "MISSING_SIGNATURE".to_string(),
                            message: "Missing TrueLayer-Signature header".to_string(),
                        }),
                    ));
                }
                Some(sig) => {
                    if !verify_webhook_signature(body.as_bytes(), sig, secret) {
                        warn!("Webhook signature verification failed");
                        return Err((
                            StatusCode::UNAUTHORIZED,
                            Json(BillingErrorResponse {
                                code: "INVALID_SIGNATURE".to_string(),
                                message: "Webhook signature verification failed".to_string(),
                            }),
                        ));
                    }
                    info!("Webhook signature verified successfully");
                }
            }
        }
    }

    let payload: WebhookPayload = serde_json::from_str(&body).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(BillingErrorResponse {
                code: "INVALID_PAYLOAD".to_string(),
                message: format!("Invalid webhook payload: {}", e),
            }),
        )
    })?;

    info!(
        "Received TrueLayer webhook: {} ({})",
        payload.event_type, payload.event_id
    );

    match payload.event_type.as_str() {
        "mandate_approved" | "mandate_active" => {
            if let Some(mandate_id) = payload.body.get("id").and_then(|v| v.as_str()) {
                let repo = tachyon_database::SubscriptionRepository::new(state.pool.clone());
                let status = payload.event_type.as_str();
                if let Ok(sub) = repo.list_all().await {
                    for sub in sub {
                        if sub.payment_method_id.as_deref() == Some(mandate_id) {
                            let _ = repo
                                .update(
                                    &sub.id,
                                    tachyon_database::UpdateSubscriptionRequest {
                                        status: Some(status.to_string()),
                                        plan: None,
                                        cancel_at_period_end: None,
                                        payment_method_id: None,
                                    },
                                )
                                .await;
                            info!(
                                "Updated subscription {} mandate status to {}",
                                sub.id, status
                            );
                            break;
                        }
                    }
                }
            }
        }
        "payment_settled" | "payment_executed" => {
            if let Some(payment_id) = payload.body.get("id").and_then(|v| v.as_str()) {
                let status = payload.event_type.as_str();
                info!("Payment {} status: {}", payment_id, status);
            }
        }
        "payment_failed" => {
            if let Some(payment_id) = payload.body.get("id").and_then(|v| v.as_str()) {
                warn!("Payment {} failed", payment_id);
            }
        }
        _ => {
            info!("Unhandled webhook event type: {}", payload.event_type);
        }
    }

    Ok(Json(serde_json::json!({
        "received": true,
        "event_id": payload.event_id,
    })))
}

// ============================================================================
// Router
// ============================================================================

pub fn create_billing_router() -> axum::Router<BillingState> {
    use axum::routing::{get, post};

    axum::Router::new()
        .route("/billing/plans", get(list_plans))
        .route("/billing/subscriptions", post(create_subscription))
        .route("/billing/subscriptions/{org_id}", get(get_subscription))
        .route(
            "/billing/subscriptions/{org_id}/cancel",
            post(cancel_subscription),
        )
        .route("/billing/subscription/change-plan", post(change_plan))
        .route("/billing/invoices/{org_id}", get(list_invoices))
        .route("/billing/usage/{org_id}", get(get_usage))
        .route("/billing/mandates", post(create_mandate))
        .route("/billing/mandates/{mandate_id}", get(get_mandate_status))
        .route("/billing/payments", post(create_payment))
        .route("/billing/payments/{payment_id}", get(get_payment_status))
        .route("/billing/webhook", post(webhook_handler))
}
