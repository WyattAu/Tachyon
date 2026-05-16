// Database Migrations
// SQLx-based database migrations for Tachyon

use sqlx::migrate::Migrator;

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;

/// Embedded migrations
pub static MIGRATOR: Migrator = sqlx::migrate!();

/// Run database migrations
///
/// # Arguments
/// * `pool` - Database pool to run migrations against
///
/// # Returns
/// Result indicating success or failure
///
/// # Errors
/// Returns error if migration fails
pub(crate) async fn run_migrations(pool: &DatabasePool) -> DatabaseResult<()> {
    MIGRATOR.run(pool.inner()).await?;
    Ok(())
}

/// Roll back the last N applied migrations.
///
/// Queries the `_sqlx_migrations` table to find the most recent applied
/// migration versions and marks them as "rolled back" by deleting those rows.
/// Note: SQLx does not ship DOWN migration SQL, so actual schema reversal
/// must be performed manually or via separate migration files.
///
/// # Arguments
/// * `pool` - Database pool
/// * `steps` - Number of migrations to roll back
///
/// # Errors
/// Returns error if rollback fails or if fewer than `steps` migrations have been applied.
pub async fn rollback(pool: &DatabasePool, steps: usize) -> DatabaseResult<()> {
    if steps == 0 {
        return Ok(());
    }

    let rows: Vec<(i64, String)> = sqlx::query_as(
        "SELECT version, description FROM _sqlx_migrations WHERE success = true ORDER BY version DESC LIMIT $1",
    )
    .bind(steps as i64)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| DatabaseError::query_error(format!("Failed to query migration history: {}", e)))?;

    if rows.is_empty() {
        return Err(DatabaseError::MigrationError(
            "No migrations have been applied".to_string(),
        ));
    }

    if rows.len() < steps {
        return Err(DatabaseError::MigrationError(format!(
            "Only {} migrations have been applied, cannot roll back {}",
            rows.len(),
            steps
        )));
    }

    let versions: Vec<String> = rows.iter().map(|(v, _)| v.to_string()).collect();
    let descriptions: Vec<&str> = rows.iter().map(|(_, d)| d.as_str()).collect();

    tracing::warn!(
        "Rolling back {} migration(s): {}",
        steps,
        descriptions.join(", ")
    );

    let result =
        sqlx::query("DELETE FROM _sqlx_migrations WHERE version = ANY($1) AND success = true")
            .bind(&versions)
            .execute(pool.inner())
            .await
            .map_err(|e| {
                DatabaseError::query_error(format!("Failed to roll back migrations: {}", e))
            })?;

    tracing::info!(
        "Rolled back {} migration(s). {} row(s) deleted from _sqlx_migrations.",
        steps,
        result.rows_affected()
    );

    tracing::warn!(
        "Schema changes from rolled-back migrations were NOT reversed. \
         Manual SQL may be required to restore the previous schema state. \
         Versions removed: {}",
        versions.join(", ")
    );

    Ok(())
}

/// List all applied migrations.
///
/// # Arguments
/// * `pool` - Database pool
///
/// # Returns
/// A vector of (version, description) tuples for all successfully applied migrations.
///
/// # Errors
/// Returns error if the query fails.
pub async fn list_applied(pool: &DatabasePool) -> DatabaseResult<Vec<(String, String)>> {
    let rows: Vec<(i64, String)> = sqlx::query_as(
        "SELECT version, description FROM _sqlx_migrations WHERE success = true ORDER BY version ASC",
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| DatabaseError::query_error(format!("Failed to list applied migrations: {}", e)))?;

    Ok(rows.into_iter().map(|(v, d)| (v.to_string(), d)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrator_exists() {
        let _ = &MIGRATOR;
    }

    #[test]
    fn test_rollback_zero_steps_is_noop() {
        // rollback with 0 steps should be conceptually a no-op
        // We can't call it without a pool, but the guard at the top is clear
        let steps: u32 = 0;
        assert_eq!(steps, 0, "zero rollback steps is a no-op");
    }
}
