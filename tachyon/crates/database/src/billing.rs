//! Billing repository (subscriptions, invoices)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::instrument;

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;

// ============================================================================
// Subscription
// ============================================================================

const SUB_SELECT: &str = r#"
    SELECT 
        id::TEXT, organization_id::TEXT, plan, status,
        current_period_start::TEXT, current_period_end::TEXT,
        cancel_at_period_end, payment_method_id::TEXT,
        created_at::TEXT, updated_at::TEXT
    FROM subscriptions
"#;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub organization_id: String,
    pub plan: String,
    pub status: String,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub cancel_at_period_end: bool,
    pub payment_method_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub organization_id: String,
    pub plan: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSubscriptionRequest {
    pub plan: Option<String>,
    pub status: Option<String>,
    pub cancel_at_period_end: Option<bool>,
    pub payment_method_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SubscriptionRepository {
    pool: DatabasePool,
}

impl SubscriptionRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn create(&self, req: CreateSubscriptionRequest) -> DatabaseResult<Subscription> {
        let sql = r#"
            INSERT INTO subscriptions (organization_id, plan, status, current_period_end)
            VALUES ($1, $2, 'active', NOW() + INTERVAL '30 days')
            RETURNING 
                id::TEXT, organization_id::TEXT, plan, status,
                current_period_start::TEXT, current_period_end::TEXT,
                cancel_at_period_end, payment_method_id::TEXT,
                created_at::TEXT, updated_at::TEXT
        "#;

        sqlx::query_as::<_, Subscription>(&sql)
            .bind(&req.organization_id)
            .bind(&req.plan)
            .fetch_one(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(&e.to_string()))
    }

    #[instrument(skip(self))]
    pub async fn get_by_org(&self, organization_id: &str) -> DatabaseResult<Subscription> {
        let sql = format!("{} WHERE organization_id = $1 ORDER BY created_at DESC LIMIT 1", SUB_SELECT);
        sqlx::query_as::<_, Subscription>(&sql)
            .bind(organization_id)
            .fetch_optional(self.pool.inner())
            .await?
            .ok_or_else(|| DatabaseError::not_found("subscription", organization_id))
    }

    #[instrument(skip(self))]
    pub async fn update(&self, id: &str, req: UpdateSubscriptionRequest) -> DatabaseResult<Subscription> {
        let existing = self.get_by_id(id).await?;
        let now = Utc::now();

        let plan = req.plan.unwrap_or(existing.plan);
        let status = req.status.unwrap_or(existing.status);
        let cancel = req.cancel_at_period_end.unwrap_or(existing.cancel_at_period_end);
        let payment = req.payment_method_id.or(existing.payment_method_id);

        let sql = r#"
            UPDATE subscriptions 
            SET plan = $1, status = $2, cancel_at_period_end = $3, 
                payment_method_id = $4, updated_at = $5
            WHERE id = $6
            RETURNING 
                id::TEXT, organization_id::TEXT, plan, status,
                current_period_start::TEXT, current_period_end::TEXT,
                cancel_at_period_end, payment_method_id::TEXT,
                created_at::TEXT, updated_at::TEXT
        "#;

        sqlx::query_as::<_, Subscription>(&sql)
            .bind(&plan)
            .bind(&status)
            .bind(&cancel)
            .bind(&payment)
            .bind(now)
            .bind(id)
            .fetch_one(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(&e.to_string()))
    }

    #[instrument(skip(self))]
    pub async fn get_by_id(&self, id: &str) -> DatabaseResult<Subscription> {
        let sql = format!("{} WHERE id = $1", SUB_SELECT);
        sqlx::query_as::<_, Subscription>(&sql)
            .bind(id)
            .fetch_optional(self.pool.inner())
            .await?
            .ok_or_else(|| DatabaseError::not_found("subscription", id))
    }

    pub async fn list_all(&self) -> DatabaseResult<Vec<Subscription>> {
        let sql = format!("{} ORDER BY created_at DESC LIMIT 100", SUB_SELECT);
        sqlx::query_as::<_, Subscription>(&sql)
            .fetch_all(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(&e.to_string()))
    }
}

// ============================================================================
// Invoice
// ============================================================================

const INV_SELECT: &str = r#"
    SELECT 
        id::TEXT, subscription_id::TEXT, organization_id::TEXT,
        amount_cents, currency, status, description,
        invoice_date::TEXT, due_date::TEXT, payment_url, paid_at::TEXT,
        created_at::TEXT
    FROM invoices
"#;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub subscription_id: String,
    pub organization_id: String,
    pub amount_cents: i64,
    pub currency: String,
    pub status: String,
    pub description: String,
    pub invoice_date: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    pub payment_url: Option<String>,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInvoiceRequest {
    pub subscription_id: String,
    pub organization_id: String,
    pub amount_cents: i64,
    pub currency: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInvoiceRequest {
    pub status: Option<String>,
    pub paid_at: Option<DateTime<Utc>>,
    pub payment_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InvoiceRepository {
    pool: DatabasePool,
}

impl InvoiceRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn create(&self, req: CreateInvoiceRequest) -> DatabaseResult<Invoice> {
        let sql = r#"
            INSERT INTO invoices (subscription_id, organization_id, amount_cents, currency, description)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING 
                id::TEXT, subscription_id::TEXT, organization_id::TEXT,
                amount_cents, currency, status, description,
                invoice_date::TEXT, due_date::TEXT, payment_url, paid_at::TEXT,
                created_at::TEXT
        "#;

        sqlx::query_as::<_, Invoice>(&sql)
            .bind(&req.subscription_id)
            .bind(&req.organization_id)
            .bind(req.amount_cents)
            .bind(&req.currency)
            .bind(&req.description)
            .fetch_one(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(&e.to_string()))
    }

    #[instrument(skip(self))]
    pub async fn list_by_org(&self, organization_id: &str) -> DatabaseResult<Vec<Invoice>> {
        let sql = format!("{} WHERE organization_id = $1 ORDER BY invoice_date DESC LIMIT 100", INV_SELECT);
        sqlx::query_as::<_, Invoice>(&sql)
            .bind(organization_id)
            .fetch_all(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(&e.to_string()))
    }

    #[instrument(skip(self))]
    pub async fn update(&self, id: &str, req: UpdateInvoiceRequest) -> DatabaseResult<Invoice> {
        let existing = self.get_by_id(id).await?;
        let status = req.status.unwrap_or(existing.status);
        let paid_at = req.paid_at.or(existing.paid_at);
        let payment_url = req.payment_url.or(existing.payment_url);

        let sql = r#"
            UPDATE invoices SET status = $1, paid_at = $2, payment_url = $3
            WHERE id = $4
            RETURNING 
                id::TEXT, subscription_id::TEXT, organization_id::TEXT,
                amount_cents, currency, status, description,
                invoice_date::TEXT, due_date::TEXT, payment_url, paid_at::TEXT,
                created_at::TEXT
        "#;

        sqlx::query_as::<_, Invoice>(&sql)
            .bind(&status)
            .bind(&paid_at)
            .bind(&payment_url)
            .bind(id)
            .fetch_one(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(&e.to_string()))
    }

    pub async fn get_by_id(&self, id: &str) -> DatabaseResult<Invoice> {
        let sql = format!("{} WHERE id = $1", INV_SELECT);
        sqlx::query_as::<_, Invoice>(&sql)
            .bind(id)
            .fetch_optional(self.pool.inner())
            .await?
            .ok_or_else(|| DatabaseError::not_found("invoice", id))
    }
}

// ============================================================================
// Notification Preferences
// ============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct NotificationPreference {
    pub user_id: String,
    pub notification_type: String,
    pub enabled: bool,
    pub channel: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertNotificationPrefRequest {
    pub notification_type: String,
    pub enabled: bool,
    pub channel: String,
}

#[derive(Debug, Clone)]
pub struct NotificationPreferenceRepository {
    pool: DatabasePool,
}

impl NotificationPreferenceRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn upsert(&self, req: UpsertNotificationPrefRequest, user_id: &str) -> DatabaseResult<NotificationPreference> {
        let sql = r#"
            INSERT INTO notification_preferences (user_id, notification_type, enabled, channel, updated_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (user_id, notification_type) 
                DO UPDATE SET enabled = $3, channel = $4, updated_at = NOW()
            RETURNING user_id::TEXT, notification_type, enabled, channel, updated_at::TEXT
        "#;

        sqlx::query_as::<_, NotificationPreference>(&sql)
            .bind(user_id)
            .bind(&req.notification_type)
            .bind(&req.enabled)
            .bind(&req.channel)
            .fetch_one(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(&e.to_string()))
    }

    pub async fn list_by_user(&self, user_id: &str) -> DatabaseResult<Vec<NotificationPreference>> {
        let sql = r#"
            SELECT user_id::TEXT, notification_type, enabled, channel, updated_at::TEXT
            FROM notification_preferences WHERE user_id = $1
            ORDER BY notification_type LIMIT 50
        "#;
        sqlx::query_as::<_, NotificationPreference>(&sql)
            .bind(user_id)
            .fetch_all(self.pool.inner())
            .await
            .map_err(|e| DatabaseError::query_error(&e.to_string()))
    }
}
