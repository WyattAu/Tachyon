// User repository tests
// Tests user CRUD operations and session management

use tachyon_database::{
    DatabasePool, SessionRepository, UserRepository,
    Session, SessionId,
    User, UserId,
};
use chrono::{Utc, Duration};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;

async fn setup_test_db() -> DatabasePool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tachyon:tachyon@localhost:5433/tachyon_test".to_string());
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to setup test database");
    
    DatabasePool::new(pool)
}

async fn cleanup_test_data(pool: &DatabasePool) {
    let cleanup_queries = vec![
        "DELETE FROM users WHERE username LIKE 'TEST_%'",
        "DELETE FROM sessions WHERE id::text LIKE 'TEST_%'",
    ];
    
    for query in cleanup_queries {
        sqlx::query(query)
            .execute(pool)
            .await
            .ok();
    }
}

fn create_test_user() -> User {
    User {
        id: UserId::new(uuid::Uuid::new_v4().to_string()),
        username: format!("TEST_user_{}", Utc::now().timestamp()),
        email: format!("test_{}@example.com", Utc::now().timestamp()),
        display_name: Some("Test User".to_string()),
        avatar_url: None,
        bio: None,
        status: "Active".to_string(),
        role: "User".to_string(),
        preferences: json!({}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login: None,
    }
}

fn create_test_session(user_id: &UserId) -> Session {
    Session {
        id: SessionId::new(format!("TEST_session_{}", Utc::now().timestamp())),
        user_id: user_id.clone(),
        session_type: "web".to_string(),
        status: "Active".to_string(),
        token_value: uuid::Uuid::new_v4().to_string(),
        token_type: "bearer".to_string(),
        ip_address: Some("127.0.0.1".to_string()),
        user_agent: Some("Test Agent".to_string()),
        device_info: Some("Test Device".to_string()),
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::hours(24),
        last_activity: Utc::now(),
    }
}

#[tokio::test]
async fn test_create_user() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;
    
    let repo = UserRepository::new(pool.clone());
    let user = create_test_user();
    
    let result = repo.create(&user).await;
    assert!(result.is_ok(), "Failed to create user");
    
    // Verify user was created
    let fetched = repo.get_by_id(&user.id).await;
    assert!(fetched.is_ok(), "Failed to fetch created user");
    let fetched_user = fetched.unwrap();
    assert_eq!(fetched_user.username, user.username);
    assert_eq!(fetched_user.email, user.email);
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_session() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;
    
    let user_repo = UserRepository::new(pool.clone());
    let session_repo = SessionRepository::new(pool.clone());
    
    // Create a test user first
    let user = create_test_user();
    user_repo.create(&user).await.expect("Failed to create user");
    
    // Create a session
    let session = create_test_session(&user.id);
    
    let result = session_repo.create(&session).await;
    assert!(result.is_ok(), "Failed to create session");
    
    // Verify session was created
    let fetched = session_repo.get_by_id(&session.id).await;
    assert!(fetched.is_ok(), "Failed to fetch created session");
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_session_by_token() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;
    
    let user_repo = UserRepository::new(pool.clone());
    let session_repo = SessionRepository::new(pool.clone());
    
    // Create a test user first
    let user = create_test_user();
    user_repo.create(&user).await.expect("Failed to create user");
    
    // Create a session
    let session = create_test_session(&user.id);
    let token = session.token_value.clone();
    
    session_repo.create(&session).await.expect("Failed to create session");
    
    // Fetch by token
    let fetched = session_repo.get_by_token(&token).await;
    assert!(fetched.is_ok(), "Failed to fetch session by token");
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_user_sessions() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;
    
    let user_repo = UserRepository::new(pool.clone());
    let session_repo = SessionRepository::new(pool.clone());
    
    // Create a test user first
    let user = create_test_user();
    user_repo.create(&user).await.expect("Failed to create user");
    
    // Create multiple sessions
    for i in 0..3 {
        let mut session = create_test_session(&user.id);
        session.id = SessionId::new(format!("TEST_session_{}_{}", i, Utc::now().timestamp()));
        session_repo.create(&session).await.expect("Failed to create session");
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
    
    // List sessions for user
    let sessions = session_repo.get_by_user(user.id.as_str(), false).await;
    assert!(sessions.is_ok(), "Failed to list user sessions");
    let sessions = sessions.unwrap();
    
    assert!(sessions.len() >= 3, "Should have at least 3 sessions for user");
    
    cleanup_test_data(&pool).await;
}
