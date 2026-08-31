use sqlx::PgPool;
use tachyon_database::{DatabasePool, init_with_migrations};

pub async fn setup_test_pool() -> Option<DatabasePool> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tachyon:tachyon@localhost:5432/tachyon_test".to_string());
    
    init_with_migrations(&database_url).await.ok()
}

pub async fn cleanup_documents(pool: &DatabasePool) {
    let _ = sqlx::query("DELETE FROM documents WHERE title LIKE 'TEST_%' OR title LIKE 'Test Document%'")
        .execute(pool.inner())
        .await;
}

pub async fn cleanup_projects(pool: &DatabasePool) {
    let _ = sqlx::query("DELETE FROM projects WHERE name LIKE 'TEST_%' OR name LIKE 'Test Project%'")
        .execute(pool.inner())
        .await;
}

pub async fn cleanup_sessions(pool: &DatabasePool) {
    let _ = sqlx::query("DELETE FROM sessions WHERE id LIKE 'TEST_%'")
        .execute(pool.inner())
        .await;
}

pub async fn cleanup_all_test_data(pool: &DatabasePool) {
    cleanup_documents(pool).await;
    cleanup_projects(pool).await;
    cleanup_sessions(pool).await;
}

pub fn get_test_database_url() -> String {
    std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tachyon:tachyon@localhost:5432/tachyon_test".to_string())
}

pub fn is_database_available() -> bool {
    std::env::var("TEST_DATABASE_URL").is_ok() || 
        cfg!(feature = "integration-tests")
}

pub async fn wait_for_database(pool: &DatabasePool, timeout_ms: u64) -> bool {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(timeout_ms);
    
    while start.elapsed() < timeout {
        match sqlx::query("SELECT 1").execute(pool.inner()).await {
            Ok(_) => return true,
            Err(_) => {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }
    
    false
}
