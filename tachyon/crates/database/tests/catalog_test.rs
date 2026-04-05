use tachyon_database::{CatalogRepository, Project, init_with_migrations};
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;

async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tachyon:tachyon@localhost:5432/tachyon_test".to_string());
    
    let pool = init_with_migrations(&database_url)
        .await
        .expect("Failed to setup test database");
    
    pool
}

async fn cleanup_test_data(pool: &PgPool) {
    sqlx::query("DELETE FROM projects WHERE name LIKE 'TEST_%'")
        .execute(pool)
        .await
        .ok();
}

fn create_test_project() -> Project {
    let timestamp = Utc::now().timestamp();
    Project {
        id: uuid::Uuid::new_v4().to_string(),
        name: format!("TEST_Project_{}", timestamp),
        slug: format!("test-project-{}", timestamp),
        description: Some("Test project for catalog".to_string()),
        project_type: "service".to_string(),
        owner_id: uuid::Uuid::new_v4().to_string(),
        organization_id: None,
        lifecycle: "production".to_string(),
        repository_url: Some(format!("https://github.com/test/{}", timestamp)),
        docs_url: None,
        api_url: None,
        tags: vec!["test".to_string(), "catalog".to_string()],
        metadata: json!({"environment": "test"}),
        language: Some("Rust".to_string()),
        framework: Some("Axum".to_string()),
        visibility: "Private".to_string(),
        status: "Active".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[tokio::test]
async fn test_create_project() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;
    
    let repo = CatalogRepository::new(pool.clone());
    let project = create_test_project();
    
    let result = repo.create_project(&project).await;
    assert!(result.is_ok(), "Failed to create project: {:?}", result.err());
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_project_by_id() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;
    
    let repo = CatalogRepository::new(pool.clone());
    let project = create_test_project();
    
    repo.create_project(&project).await.expect("Failed to create project");
    
    let retrieved = repo.get_project(&project.id).await
        .expect("Failed to get project");
    
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, project.id);
    assert_eq!(retrieved.name, project.name);
    assert_eq!(retrieved.slug, project.slug);
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_project_by_slug() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;
    
    let repo = CatalogRepository::new(pool.clone());
    let project = create_test_project();
    
    repo.create_project(&project).await.expect("Failed to create project");
    
    let retrieved = repo.get_project_by_slug(&project.slug).await
        .expect("Failed to get project by slug");
    
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.slug, project.slug);
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_project() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;
    
    let repo = CatalogRepository::new(pool.clone());
    let mut project = create_test_project();
    
    repo.create_project(&project).await.expect("Failed to create project");
    
    project.description = Some("Updated description".to_string());
    project.lifecycle = "development".to_string();
    project.updated_at = Utc::now();
    
    let result = repo.update_project(&project).await;
    assert!(result.is_ok(), "Failed to update project: {:?}", result.err());
    
    let updated = repo.get_project(&project.id).await
        .expect("Failed to get updated project")
        .expect("Project not found");
    
    assert_eq!(updated.description, Some("Updated description".to_string()));
    assert_eq!(updated.lifecycle, "development");
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_delete_project() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;
    
    let repo = CatalogRepository::new(pool.clone());
    let project = create_test_project();
    
    repo.create_project(&project).await.expect("Failed to create project");
    
    let result = repo.delete_project(&project.id).await;
    assert!(result.is_ok(), "Failed to delete project: {:?}", result.err());
    
    let retrieved = repo.get_project(&project.id).await
        .expect("Failed to query project");
    assert!(retrieved.is_none(), "Project should be deleted");
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_projects() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;
    
    let repo = CatalogRepository::new(pool.clone());
    
    for i in 0..3 {
        let mut project = create_test_project();
        project.name = format!("TEST_List_{}_{}", i, Utc::now().timestamp());
        project.slug = format!("test-list-{}-{}", i, Utc::now().timestamp());
        repo.create_project(&project).await.expect("Failed to create project");
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
    
    let projects = repo.list_projects(None, Some(10), Some(0)).await
        .expect("Failed to list projects");
    
    assert!(projects.len() >= 3, "Should have at least 3 test projects");
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_filter_projects_by_lifecycle() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;
    
    let repo = CatalogRepository::new(pool.clone());
    
    let mut dev_project = create_test_project();
    dev_project.lifecycle = "development".to_string();
    dev_project.name = format!("TEST_Dev_{}", Utc::now().timestamp());
    repo.create_project(&dev_project).await.expect("Failed to create project");
    
    let mut prod_project = create_test_project();
    prod_project.lifecycle = "production".to_string();
    prod_project.name = format!("TEST_Prod_{}", Utc::now().timestamp());
    prod_project.slug = format!("test-prod-{}", Utc::now().timestamp());
    repo.create_project(&prod_project).await.expect("Failed to create project");
    
    let results = repo.filter_by_lifecycle("production").await
        .expect("Failed to filter by lifecycle");
    
    assert!(!results.is_empty(), "Should find production projects");
    assert!(results.iter().any(|p| p.lifecycle == "production"));
    
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_search_projects() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;
    
    let repo = CatalogRepository::new(pool.clone());
    let mut project = create_test_project();
    project.tags = vec!["unique-search-tag".to_string()];
    
    repo.create_project(&project).await.expect("Failed to create project");
    
    let results = repo.search_projects("unique-search-tag").await
        .expect("Failed to search projects");
    
    assert!(!results.is_empty(), "Should find project by search term");
    
    cleanup_test_data(&pool).await;
}
