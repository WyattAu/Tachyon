//! Billing types
//!
//! Type definitions for billing API.

use chrono::{DateTime, Utc};
use hmac::Hmac;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::truelayer::TrueLayerClient;

pub type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, PartialEq)]
pub enum TransitionType {
    Upgrade,
    Downgrade,
}

/// Proration calculation result.
///
/// Reserved for future use: subscription plan transition billing.
#[derive(Debug, Clone)]
#[allow(dead_code)] // reserved for future billing calculation endpoints
pub struct ProrationResult {
    pub prorated_amount: f64,
    pub credit: f64,
    pub charge: f64,
    pub days_remaining: u32,
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
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
            Plan::Pro => vec![
                "Advanced editor",
                "50GB storage",
                "SSG export",
                "Plugin system",
                "Email support",
            ],
            Plan::Team => vec![
                "Everything in Pro",
                "Collaboration",
                "Admin panel",
                "Audit logs",
                "Priority support",
            ],
            Plan::Enterprise => vec![
                "Everything in Team",
                "SSO/SAML",
                "Custom integrations",
                "SLA",
                "Dedicated support",
            ],
        }
    }
}

/// Usage metrics for the billing period
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
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

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateSubscriptionRequest {
    pub organization_id: String,
    pub plan: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SubscriptionResponse {
    pub subscription: tachyon_database::Subscription,
    pub plan_details: PlanDetails,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PlanDetails {
    pub name: String,
    pub price_monthly_cents: u64,
    pub max_documents: usize,
    pub max_members: usize,
    pub features: Vec<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PlansResponse {
    pub plans: Vec<PlanInfo>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PlanInfo {
    pub name: String,
    pub price_monthly_cents: u64,
    pub max_documents: usize,
    pub max_members: usize,
    pub features: Vec<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct InvoicesResponse {
    pub invoices: Vec<tachyon_database::Invoice>,
    pub total: usize,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UsageResponse {
    pub usage: UsageMetrics,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateMandateRequest {
    pub organization_id: String,
    pub return_url: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BillingErrorResponse {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MandateResponse {
    pub mandate_id: String,
    pub authorization_url: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MandateStatusResponse {
    pub mandate_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreatePaymentRequest {
    pub mandate_id: String,
    pub organization_id: String,
    pub amount_cents: u64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PaymentResponse {
    pub payment_id: String,
    pub status: String,
    pub amount: u64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PaymentStatusResponse {
    pub payment_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ChangePlanRequest {
    pub organization_id: String,
    pub new_plan: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ChangePlanResponse {
    pub subscription_id: String,
    pub old_plan: String,
    pub new_plan: String,
    pub status: String,
    pub effective_at: String,
    pub prorated_amount: Option<f64>,
    pub next_billing_date: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct WebhookPayload {
    #[serde(rename = "type")]
    pub event_type: String,
    pub event_id: String,
    pub body: serde_json::Value,
}
