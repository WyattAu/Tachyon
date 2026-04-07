// Database Migrations
// SQLx-based database migrations for Tachyon

use sqlx::migrate::Migrator;

use crate::error::DatabaseResult;
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
pub async fn run_migrations(pool: &DatabasePool) -> DatabaseResult<()> {
    MIGRATOR.run(pool.inner()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrator_exists() {
        // Just verify the migrator is valid
        let _ = &MIGRATOR;
    }
}
