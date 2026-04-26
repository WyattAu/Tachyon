use tachyon_database::{CreateTemplateRequest, TemplateRepository, UpdateTemplateRequest};

use crate::common::setup::{
    create_test_pool, create_test_user, setup_database, teardown_database,
};

fn skip_without_db() -> bool {
    std::env::var("DATABASE_URL").is_err()
        && std::env::var("TEST_DATABASE_URL").is_err()
}

#[tokio::test]
async fn test_create_template() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let _ = teardown_database(&pool).await;

    let user = create_test_user(&pool).await;
    let repo = TemplateRepository::new(pool.clone());

    let template = repo
        .create(CreateTemplateRequest {
            name: "Meeting Notes".to_string(),
            description: Some("Template for meeting notes".to_string()),
            content: "# Meeting Notes\n\n## Attendees\n\n## Agenda\n\n## Action Items".to_string(),
            category: Some("productivity".to_string()),
            tags: Some(vec!["meeting".to_string(), "notes".to_string()]),
            created_by: user.id.as_str(),
        })
        .await
        .expect("Failed to create template");

    assert!(!template.id.is_empty());
    assert_eq!(template.name, "Meeting Notes");
    assert_eq!(template.description.as_deref(), Some("Template for meeting notes"));
    assert_eq!(template.category.as_deref(), Some("productivity"));

    let tags = template.parse_tags().expect("Failed to parse tags");
    assert!(tags.contains(&"meeting".to_string()));
    assert!(tags.contains(&"notes".to_string()));

    teardown_database(&pool).await;
}

#[tokio::test]
async fn test_get_template_by_id() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let _ = teardown_database(&pool).await;

    let user = create_test_user(&pool).await;
    let repo = TemplateRepository::new(pool.clone());

    let created = repo
        .create(CreateTemplateRequest {
            name: "Get Test Template".to_string(),
            description: None,
            content: "# Hello".to_string(),
            category: None,
            tags: None,
            created_by: user.id.as_str(),
        })
        .await
        .expect("Failed to create template");

    let fetched = repo.get_by_id(&created.id).await.expect("Failed to get template by ID");
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, "Get Test Template");

    teardown_database(&pool).await;
}

#[tokio::test]
async fn test_get_template_by_name() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let _ = teardown_database(&pool).await;

    let user = create_test_user(&pool).await;
    let repo = TemplateRepository::new(pool.clone());

    let unique_name = format!("Unique Template {}", uuid::Uuid::new_v4());
    repo.create(CreateTemplateRequest {
        name: unique_name.clone(),
        description: None,
        content: "# Content".to_string(),
        category: None,
        tags: None,
        created_by: user.id.as_str(),
    })
    .await
    .expect("Failed to create template");

    let fetched = repo.get_by_name(&unique_name).await.expect("Failed to get template by name");
    assert_eq!(fetched.name, unique_name);

    teardown_database(&pool).await;
}

#[tokio::test]
async fn test_list_templates() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let _ = teardown_database(&pool).await;

    let user = create_test_user(&pool).await;
    let repo = TemplateRepository::new(pool.clone());

    for i in 0..3 {
        repo.create(CreateTemplateRequest {
            name: format!("List Template {}", i),
            description: None,
            content: format!("# Template {}", i),
            category: Some("general".to_string()),
            tags: None,
            created_by: user.id.as_str(),
        })
        .await
        .expect("Failed to create template");
    }

    let all = repo.list(None, Some(10), None).await.expect("Failed to list templates");
    assert!(all.len() >= 3);

    let filtered = repo
        .list(Some("general"), Some(10), None)
        .await
        .expect("Failed to list templates by category");
    assert!(filtered.len() >= 3);

    teardown_database(&pool).await;
}

#[tokio::test]
async fn test_update_template() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let _ = teardown_database(&pool).await;

    let user = create_test_user(&pool).await;
    let repo = TemplateRepository::new(pool.clone());

    let created = repo
        .create(CreateTemplateRequest {
            name: "Before Update".to_string(),
            description: Some("Old description".to_string()),
            content: "# Old Content".to_string(),
            category: None,
            tags: None,
            created_by: user.id.as_str(),
        })
        .await
        .expect("Failed to create template");

    let updated = repo
        .update(
            &created.id,
            UpdateTemplateRequest {
                name: Some("After Update".to_string()),
                description: Some("New description".to_string()),
                content: Some("# New Content".to_string()),
                category: Some("docs".to_string()),
                tags: Some(vec!["updated".to_string()]),
            },
        )
        .await
        .expect("Failed to update template");

    assert_eq!(updated.name, "After Update");
    assert_eq!(updated.description.as_deref(), Some("New description"));
    assert_eq!(updated.category.as_deref(), Some("docs"));

    let tags = updated.parse_tags().expect("Failed to parse tags");
    assert!(tags.contains(&"updated".to_string()));

    teardown_database(&pool).await;
}

#[tokio::test]
async fn test_delete_template() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let _ = teardown_database(&pool).await;

    let user = create_test_user(&pool).await;
    let repo = TemplateRepository::new(pool.clone());

    let created = repo
        .create(CreateTemplateRequest {
            name: "To Delete".to_string(),
            description: None,
            content: "# Delete me".to_string(),
            category: None,
            tags: None,
            created_by: user.id.as_str(),
        })
        .await
        .expect("Failed to create template");

    repo.delete(&created.id).await.expect("Failed to delete template");

    let result = repo.get_by_id(&created.id).await;
    assert!(result.is_err(), "Deleted template should not be found");

    teardown_database(&pool).await;
}

#[tokio::test]
async fn test_template_count() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let _ = teardown_database(&pool).await;

    let user = create_test_user(&pool).await;
    let repo = TemplateRepository::new(pool.clone());

    repo.create(CreateTemplateRequest {
        name: "Count Template 1".to_string(),
        description: None,
        content: "# 1".to_string(),
        category: Some("test".to_string()),
        tags: None,
        created_by: user.id.as_str(),
    })
    .await
    .expect("Failed to create template");

    let total = repo.count(None).await.expect("Failed to count templates");
    assert!(total >= 1);

    let by_category = repo.count(Some("test")).await.expect("Failed to count by category");
    assert!(by_category >= 1);

    teardown_database(&pool).await;
}
