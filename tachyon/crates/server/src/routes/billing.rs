//! Billing API routes
//! TrueLayer open banking payment integration (NOT Stripe)

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use tracing::{info, warn};
use tachyon_database::error::DatabaseError;

use crate::truelayer::{TrueLayerClient, TrueLayerError};

type HmacSha256 = Hmac<Sha256>;

fn verify_webhook_signature(payload: &[u8], signature_header: &str, secret: &str) -> bool {
    let sig_part = signature_header.strip_prefix("v1=").unwrap_or(signature_header);

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(payload);

    let expected = hex::encode(mac.finalize().into_bytes());

    expected.as_bytes().ct_eq(sig_part.as_bytes()).into()
}

/// Billing state
#[derive(Clone)]
pub struct BillingState {
    pub pool: tachyon_database::DatabasePool,
    pub truelayer: Option<TrueLayerClient>,
}

impl BillingState {
    pub fn new(pool: tachyon_database::DatabasePool, truelayer: Option<TrueLayerClient>) -> Self {
        Self { pool, truelayer }
    }
}

// ============================================================================
// Types
// ============================================================================

/// Subscription plan
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Plan {
    Free,
    Pro,
    Team,
    Enterprise,
}

impl Plan {
    pub fn price_monthly(&self) -> u64 {
        match self {
            Plan::Free => 0,
            Plan::Pro => 12_00,
            Plan::Team => 29_00,
            Plan::Enterprise => 0,
        }
    }

    pub fn max_documents(&self) -> usize {
        match self {
            Plan::Free => 100,
            Plan::Pro => 10_000,
            Plan::Team => 100_000,
            Plan::Enterprise => usize::MAX,
        }
    }

    pub fn max_members(&self) -> usize {
        match self {
            Plan::Free => 1,
            Plan::Pro => 5,
            Plan::Team => 50,
            Plan::Enterprise => usize::MAX,
        }
    }

    pub fn features(&self) -> Vec<&'static str> {
        match self {
            Plan::Free => vec!["Basic editor", "5GB storage", "Community support"],
            Plan::Pro => vec!["Advanced editor", "50GB storage", "SSG export", "Plugin system", "Email support"],
            Plan::Team => vec!["Everything in Pro", "Collaboration", "Admin panel", "Audit logs", "Priority support"],
            Plan::Enterprise => vec!["Everything in Team", "SSO/SAML", "Custom integrations", "SLA", "Dedicated support"],
        }
    }
}

/// Usage metrics for the billing period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageMetrics {
    pub organization_id: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub documents_created: usize,
    pub documents_total: usize,
    pub members_total: usize,
    pub storage_bytes: u64,
    pub plan: Plan,
}

// ============================================================================
// Request/Response types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub organization_id: String,
    pub plan: String,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionResponse {
    pub subscription: tachyon_database::Subscription,
    pub plan_details: PlanDetails,
}

#[derive(Debug, Serialize)]
pub struct PlanDetails {
    pub name: String,
    pub price_monthly_cents: u64,
    pub max_documents: usize,
    pub max_members: usize,
    pub features: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PlansResponse {
    pub plans: Vec<PlanInfo>,
}

#[derive(Debug, Serialize)]
pub struct PlanInfo {
    pub name: String,
    pub price_monthly_cents: u64,
    pub max_documents: usize,
    pub max_members: usize,
    pub features: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct InvoicesResponse {
    pub invoices: Vec<tachyon_database::Invoice>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct UsageResponse {
    pub usage: UsageMetrics,
}

#[derive(Debug, Serialize)]
pub struct BillingErrorResponse {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateMandateRequest {
    pub organization_id: String,
    pub return_url: String,
}

#[derive(Debug, Serialize)]
pub struct MandateResponse {
    pub mandate_id: String,
    pub authorization_url: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct MandateStatusResponse {
    pub mandate_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePaymentRequest {
    pub mandate_id: String,
    pub organization_id: String,
    pub amount_cents: u64,
}

#[derive(Debug, Serialize)]
pub struct PaymentResponse {
    pub payment_id: String,
    pub status: String,
    pub amount: u64,
}

#[derive(Debug, Serialize)]
pub struct PaymentStatusResponse {
    pub payment_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct WebhookPayload {
    #[serde(rename = "type")]
    pub event_type: String,
    pub event_id: String,
    pub body: serde_json::Value,
}

// ============================================================================
// Helper functions
// ============================================================================

fn plan_price(plan: &str) -> u64 {
    match plan {
        "free" => 0,
        "pro" => 12_00,
        "team" => 29_00,
        _ => 0,
    }
}

fn plan_max_docs(plan: &str) -> usize {
    match plan {
        "free" => 100, "pro" => 10_000, "team" => 100_000, _ => usize::MAX,
    }
}

fn plan_max_members(plan: &str) -> usize {
    match plan {
        "free" => 1, "pro" => 5, "team" => 50, _ => usize::MAX,
    }
}

fn plan_features(plan: &str) -> Vec<String> {
    match plan {
        "free" => vec!["Basic editor".into(), "5GB storage".into(), "Community support".into()],
        "pro" => vec!["Advanced editor".into(), "50GB storage".into(), "SSG export".into(), "Plugin system".into(), "Email support".into()],
        "team" => vec!["Everything in Pro".into(), "Collaboration".into(), "Admin panel".into(), "Audit logs".into(), "Priority support".into()],
        _ => vec![],
    }
}

fn truelayer_error_response(e: TrueLayerError) -> (StatusCode, Json<BillingErrorResponse>) {
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

// ============================================================================
// Route handlers
// ============================================================================

/// GET /api/v1/billing/plans — List available plans
pub async fn list_plans() -> Json<PlansResponse> {
    let plans = vec![
        PlanInfo {
            name: "free".to_string(),
            price_monthly_cents: Plan::Free.price_monthly(),
            max_documents: Plan::Free.max_documents(),
            max_members: Plan::Free.max_members(),
            features: Plan::Free.features().iter().map(|s| s.to_string()).collect(),
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
            features: Plan::Team.features().iter().map(|s| s.to_string()).collect(),
        },
        PlanInfo {
            name: "enterprise".to_string(),
            price_monthly_cents: Plan::Enterprise.price_monthly(),
            max_documents: Plan::Enterprise.max_documents(),
            max_members: Plan::Enterprise.max_members(),
            features: Plan::Enterprise.features().iter().map(|s| s.to_string()).collect(),
        },
    ];
    Json(PlansResponse { plans })
}

/// POST /api/v1/billing/subscriptions — Create a subscription
pub async fn create_subscription(
    State(state): State<BillingState>,
    Json(req): Json<CreateSubscriptionRequest>,
) -> Result<Json<SubscriptionResponse>, (StatusCode, Json<BillingErrorResponse>)> {
    info!("Creating {} subscription for org {}",
          req.plan,
          req.organization_id);

    let repo = tachyon_database::SubscriptionRepository::new(state.pool.clone());
    let sub = repo.create(tachyon_database::CreateSubscriptionRequest {
        organization_id: req.organization_id.clone(),
        plan: req.plan.clone(),
    }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(BillingErrorResponse {
        code: "DB_ERROR".to_string(),
        message: e.to_string(),
    })))?;

    let plan_details = PlanDetails {
        name: sub.plan.clone(),
        price_monthly_cents: plan_price(&sub.plan),
        max_documents: plan_max_docs(&sub.plan),
        max_members: plan_max_members(&sub.plan),
        features: plan_features(&sub.plan),
    };

    Ok(Json(SubscriptionResponse { subscription: sub, plan_details }))
}

/// GET /api/v1/billing/subscriptions/{org_id} — Get subscription
pub async fn get_subscription(
    State(state): State<BillingState>,
    axum::extract::Path(org_id): axum::extract::Path<String>,
) -> Result<Json<SubscriptionResponse>, (StatusCode, Json<BillingErrorResponse>)> {
    let repo = tachyon_database::SubscriptionRepository::new(state.pool.clone());
    let sub = repo.get_by_org(&org_id).await.map_err(|e| {
        if matches!(e, DatabaseError::NotFound { .. }) {
            (StatusCode::NOT_FOUND, Json(BillingErrorResponse {
                code: "NOT_FOUND".to_string(),
                message: "No subscription found".to_string(),
            }))
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(BillingErrorResponse {
                code: "DB_ERROR".to_string(),
                message: e.to_string(),
            }))
        }
    })?;

    let plan_details = PlanDetails {
        name: sub.plan.clone(),
        price_monthly_cents: plan_price(&sub.plan),
        max_documents: plan_max_docs(&sub.plan),
        max_members: plan_max_members(&sub.plan),
        features: plan_features(&sub.plan),
    };

    Ok(Json(SubscriptionResponse { subscription: sub, plan_details }))
}

/// GET /api/v1/billing/invoices/{org_id} — List invoices
pub async fn list_invoices(
    State(state): State<BillingState>,
    axum::extract::Path(org_id): axum::extract::Path<String>,
) -> Json<InvoicesResponse> {
    let repo = tachyon_database::InvoiceRepository::new(state.pool.clone());
    let invoices = repo.list_by_org(&org_id).await.unwrap_or_default();
    let total = invoices.len();
    Json(InvoicesResponse { invoices, total })
}

/// GET /api/v1/billing/usage/{org_id} — Get usage metrics
pub async fn get_usage(
    State(_state): State<BillingState>,
    axum::extract::Path(org_id): axum::extract::Path<String>,
) -> Json<UsageResponse> {
    let now = Utc::now();
    let period_start = now - chrono::Duration::days(30);

    Json(UsageResponse {
        usage: UsageMetrics {
            organization_id: org_id,
            period_start,
            period_end: now,
            documents_created: 0,
            documents_total: 0,
            members_total: 1,
            storage_bytes: 0,
            plan: Plan::Free,
        },
    })
}

/// POST /api/v1/billing/subscriptions/{org_id}/cancel — Cancel subscription
pub async fn cancel_subscription(
    State(state): State<BillingState>,
    axum::extract::Path(org_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<BillingErrorResponse>)> {
    let repo = tachyon_database::SubscriptionRepository::new(state.pool.clone());
    let sub = repo.get_by_org(&org_id).await.map_err(|e| {
        if matches!(e, DatabaseError::NotFound { .. }) {
            (StatusCode::NOT_FOUND, Json(BillingErrorResponse {
                code: "NOT_FOUND".to_string(),
                message: "No subscription found".to_string(),
            }))
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(BillingErrorResponse {
                code: "DB_ERROR".to_string(),
                message: e.to_string(),
            }))
        }
    })?;

    let updated = repo.update(&sub.id, tachyon_database::UpdateSubscriptionRequest {
        cancel_at_period_end: Some(true),
        plan: None, status: None, payment_method_id: None,
    }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(BillingErrorResponse {
        code: "DB_ERROR".to_string(),
        message: e.to_string(),
    })))?;

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
pub async fn create_mandate(
    State(state): State<BillingState>,
    Json(req): Json<CreateMandateRequest>,
) -> Result<Json<MandateResponse>, (StatusCode, Json<BillingErrorResponse>)> {
    let truelayer = state.truelayer.as_ref().ok_or_else(|| {
        (StatusCode::SERVICE_UNAVAILABLE, Json(BillingErrorResponse {
            code: "PAYMENTS_DISABLED".to_string(),
            message: "Payment processing is not enabled".to_string(),
        }))
    })?;

    let result = truelayer.create_payment_mandate(&req.organization_id, &req.return_url)
        .await
        .map_err(truelayer_error_response)?;

    let repo = tachyon_database::SubscriptionRepository::new(state.pool.clone());
    if let Ok(sub) = repo.get_by_org(&req.organization_id).await {
        let _ = repo.update(&sub.id, tachyon_database::UpdateSubscriptionRequest {
            payment_method_id: Some(result.id.clone()),
            plan: None,
            status: None,
            cancel_at_period_end: None,
        }).await;
    }

    info!("Created mandate {} for org {}", result.id, req.organization_id);

    Ok(Json(MandateResponse {
        mandate_id: result.id,
        authorization_url: result.authorization_url,
        status: result.status,
    }))
}

/// GET /api/v1/billing/mandates/{mandate_id} — Check mandate status
pub async fn get_mandate_status(
    State(state): State<BillingState>,
    axum::extract::Path(mandate_id): axum::extract::Path<String>,
) -> Result<Json<MandateStatusResponse>, (StatusCode, Json<BillingErrorResponse>)> {
    let truelayer = state.truelayer.as_ref().ok_or_else(|| {
        (StatusCode::SERVICE_UNAVAILABLE, Json(BillingErrorResponse {
            code: "PAYMENTS_DISABLED".to_string(),
            message: "Payment processing is not enabled".to_string(),
        }))
    })?;

    let result = truelayer.get_mandate_status(&mandate_id)
        .await
        .map_err(truelayer_error_response)?;

    Ok(Json(MandateStatusResponse {
        mandate_id: result.id,
        status: result.status,
    }))
}

/// POST /api/v1/billing/payments — Create a payment
pub async fn create_payment(
    State(state): State<BillingState>,
    Json(req): Json<CreatePaymentRequest>,
) -> Result<Json<PaymentResponse>, (StatusCode, Json<BillingErrorResponse>)> {
    let truelayer = state.truelayer.as_ref().ok_or_else(|| {
        (StatusCode::SERVICE_UNAVAILABLE, Json(BillingErrorResponse {
            code: "PAYMENTS_DISABLED".to_string(),
            message: "Payment processing is not enabled".to_string(),
        }))
    })?;

    let reference = format!("payment-{}", uuid::Uuid::new_v4());
    let result = truelayer.create_payment(&req.mandate_id, req.amount_cents, &reference)
        .await
        .map_err(truelayer_error_response)?;

    let invoice_repo = tachyon_database::InvoiceRepository::new(state.pool.clone());
    let sub_repo = tachyon_database::SubscriptionRepository::new(state.pool.clone());

    let subscription_id = sub_repo.get_by_org(&req.organization_id)
        .await
        .map(|s| s.id)
        .unwrap_or_default();

    if let Ok(_invoice) = invoice_repo.create(tachyon_database::CreateInvoiceRequest {
        subscription_id: subscription_id.clone(),
        organization_id: req.organization_id.clone(),
        amount_cents: req.amount_cents as i64,
        currency: "GBP".to_string(),
        description: format!("Payment {}", result.id),
    }).await {
        info!("Created invoice for payment {}", result.id);
    }

    info!("Created payment {} for org {} ({} pence)", result.id, req.organization_id, req.amount_cents);

    Ok(Json(PaymentResponse {
        payment_id: result.id,
        status: result.status,
        amount: result.amount_in_minor,
    }))
}

/// GET /api/v1/billing/payments/{payment_id} — Check payment status
pub async fn get_payment_status(
    State(state): State<BillingState>,
    axum::extract::Path(payment_id): axum::extract::Path<String>,
) -> Result<Json<PaymentStatusResponse>, (StatusCode, Json<BillingErrorResponse>)> {
    let truelayer = state.truelayer.as_ref().ok_or_else(|| {
        (StatusCode::SERVICE_UNAVAILABLE, Json(BillingErrorResponse {
            code: "PAYMENTS_DISABLED".to_string(),
            message: "Payment processing is not enabled".to_string(),
        }))
    })?;

    let result = truelayer.get_payment_status(&payment_id)
        .await
        .map_err(truelayer_error_response)?;

    Ok(Json(PaymentStatusResponse {
        payment_id: result.id,
        status: result.status,
    }))
}

/// POST /api/v1/billing/webhook — TrueLayer webhook handler
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
        (StatusCode::BAD_REQUEST, Json(BillingErrorResponse {
            code: "INVALID_PAYLOAD".to_string(),
            message: format!("Invalid webhook payload: {}", e),
        }))
    })?;

    info!("Received TrueLayer webhook: {} ({})", payload.event_type, payload.event_id);

    match payload.event_type.as_str() {
        "mandate_approved" | "mandate_active" => {
            if let Some(mandate_id) = payload.body.get("id").and_then(|v| v.as_str()) {
                let repo = tachyon_database::SubscriptionRepository::new(state.pool.clone());
                let status = payload.event_type.as_str();
                if let Ok(sub) = repo.list_all().await {
                    for sub in sub {
                        if sub.payment_method_id.as_deref() == Some(mandate_id) {
                            let _ = repo.update(&sub.id, tachyon_database::UpdateSubscriptionRequest {
                                status: Some(status.to_string()),
                                plan: None,
                                cancel_at_period_end: None,
                                payment_method_id: None,
                            }).await;
                            info!("Updated subscription {} mandate status to {}", sub.id, status);
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
        .route("/billing/subscriptions/{org_id}/cancel", post(cancel_subscription))
        .route("/billing/invoices/{org_id}", get(list_invoices))
        .route("/billing/usage/{org_id}", get(get_usage))
        .route("/billing/mandates", post(create_mandate))
        .route("/billing/mandates/{mandate_id}", get(get_mandate_status))
        .route("/billing/payments", post(create_payment))
        .route("/billing/payments/{payment_id}", get(get_payment_status))
        .route("/billing/webhook", post(webhook_handler))
}
