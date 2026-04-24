// Database Schema Definitions
// PostgreSQL with Apache AGE (graph extension) support

use crate::error::{DatabaseError, DatabaseResult};
use crate::types::DatabaseConfig;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Postgres;
use tracing::{debug, info, instrument};

/// Database pool for PostgreSQL connections
#[derive(Debug, Clone)]
pub struct DatabasePool {
    /// Inner connection pool
    pool: PgPool,
    /// Configuration
    config: DatabaseConfig,
}

impl DatabasePool {
    /// Create a new database pool with default configuration
    #[instrument(skip(database_url))]
    pub async fn new(database_url: &str) -> DatabaseResult<Self> {
        Self::with_config(database_url, DatabaseConfig::default()).await
    }

    /// Create a new database pool with custom configuration
    #[instrument(skip(database_url, config))]
    pub async fn with_config(database_url: &str, config: DatabaseConfig) -> DatabaseResult<Self> {
        info!("Initializing PostgreSQL database pool: {}", database_url);

        // Configure pool options
        debug!("Configuring pool options: max={}, min={}, timeout={}s", 
            config.max_connections, config.min_connections, config.connection_timeout);

        // Create the pool using the connection string directly
        debug!("Connecting to database with timeout...");
        let connect_future = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.connection_timeout))
            .idle_timeout(std::time::Duration::from_secs(300))
            .test_before_acquire(config.enable_query_logging)
            .connect(database_url);

        // Apply explicit timeout to connection (uses config.connection_timeout)
        let pool = tokio::time::timeout(
            std::time::Duration::from_secs(config.connection_timeout),
            connect_future
        )
        .await
        .map_err(|_| {
            tracing::error!(
                timeout_secs = config.connection_timeout,
                url = database_url,
                "Database connection timed out after {}s. Check that PostgreSQL is running and the URL is correct.",
                config.connection_timeout
            );
            DatabaseError::ConnectionError(sqlx::Error::PoolTimedOut)
        })?
        .map_err(|e| {
            tracing::error!(
                error = %e,
                url = database_url,
                "Database connection failed: {}", e
            );
            DatabaseError::ConnectionError(e)
        })?;

        info!("PostgreSQL pool initialized successfully");

        let db_pool = Self { pool, config };
        
        // Apply database configuration
        db_pool.apply_config().await?;

        Ok(db_pool)
    }

    /// Apply database configuration (PostgreSQL settings)
    async fn apply_config(&self) -> DatabaseResult<()> {
        let mut conn = self.pool.acquire().await
            .map_err(DatabaseError::ConnectionError)?;

        // Enable extensions if needed
        if self.config.enable_extensions {
            // Enable UUID generation
            sqlx::query("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\"")
                .execute(&mut *conn)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
            
            // Enable JSONB
            sqlx::query("CREATE EXTENSION IF NOT EXISTS \"pg_trgm\"")
                .execute(&mut *conn)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
                
            debug!("PostgreSQL extensions enabled");
        }

        debug!("PostgreSQL configuration applied successfully");
        Ok(())
    }

    /// Get a connection from the pool
    pub async fn acquire(&self) -> DatabaseResult<sqlx::pool::PoolConnection<Postgres>> {
        self.pool
            .acquire()
            .await
            .map_err(DatabaseError::ConnectionError)
    }

    /// Get the inner pool reference
    pub fn inner(&self) -> &PgPool {
        &self.pool
    }

    /// Get the configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// Close the database pool
    pub async fn close(&self) {
        self.pool.close().await;
        info!("PostgreSQL pool closed successfully");
    }

    /// Begin a transaction
    pub async fn begin(&self) -> DatabaseResult<sqlx::Transaction<'_, Postgres>> {
        self.pool
            .begin()
            .await
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))
    }

    /// Execute a query and return the number of affected rows
    pub async fn execute(&self, query: &str) -> DatabaseResult<u64> {
        let mut conn = self.acquire().await?;
        let result = sqlx::query(query)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(result.rows_affected())
    }

    /// Get database statistics
    pub async fn statistics(&self) -> DatabaseResult<serde_json::Value> {
        let mut stats = serde_json::Map::new();

        let pool_size = self.pool.size();
        stats.insert("pool_size".to_string(), serde_json::json!(pool_size));
        stats.insert("idle_connections".to_string(), serde_json::json!(self.pool.num_idle()));

        Ok(serde_json::Value::Object(stats))
    }
}

/// Convert from sqlx::Pool to DatabasePool
impl From<PgPool> for DatabasePool {
    fn from(pool: PgPool) -> Self {
        Self {
            pool,
            config: DatabaseConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_creation() {
        // Skip if no PostgreSQL available
        let result = DatabasePool::new("postgres://localhost:5432/test").await;
        if result.is_err() {
            return; // Skip test if no DB
        }
    }
}
