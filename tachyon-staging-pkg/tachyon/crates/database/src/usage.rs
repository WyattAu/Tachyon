//! Usage metering for multi-tenant billing.

use crate::{DatabaseError, DatabasePool};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UsageRecord {
    pub id: String,
    pub org_id: String,
    pub metric_type: String,
    pub value: i64,
    pub recorded_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub metric_type: String,
    pub total: i64,
    pub period_start: chrono::DateTime<chrono::Utc>,
    pub period_end: chrono::DateTime<chrono::Utc>,
}

pub struct UsageRepository {
    pool: DatabasePool,
}

impl UsageRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    pub async fn record(
        &self,
        org_id: &str,
        metric_type: &str,
        value: i64,
    ) -> Result<(), DatabaseError> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query("INSERT INTO usage_records (org_id, metric_type, value) VALUES ($1, $2, $3)")
            .bind(org_id)
            .bind(metric_type)
            .bind(value)
            .execute(&mut *conn)
            .await?;
        Ok(())
    }

    pub async fn get_summary(
        &self,
        org_id: &str,
        start: &chrono::DateTime<chrono::Utc>,
        end: &chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<UsageSummary>, DatabaseError> {
        let mut conn = self.pool.acquire().await?;
        let rows = sqlx::query_as::<_, (String, i64)>(
            "SELECT metric_type, SUM(value) as total FROM usage_records WHERE org_id = $1 AND recorded_at >= $2 AND recorded_at < $3 GROUP BY metric_type",
        )
        .bind(org_id)
        .bind(start)
        .bind(end)
        .fetch_all(&mut *conn)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(metric_type, total)| UsageSummary {
                metric_type,
                total,
                period_start: *start,
                period_end: *end,
            })
            .collect())
    }

    pub async fn get_current_usage(
        &self,
        org_id: &str,
    ) -> Result<Vec<(String, i64)>, DatabaseError> {
        let mut conn = self.pool.acquire().await?;
        let rows = sqlx::query_as::<_, (String, i64)>(
            "SELECT metric_type, SUM(value) as total FROM usage_records WHERE org_id = $1 AND recorded_at >= NOW() - INTERVAL '30 days' GROUP BY metric_type",
        )
        .bind(org_id)
        .fetch_all(&mut *conn)
        .await?;
        Ok(rows)
    }
}
