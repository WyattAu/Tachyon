use tachyon_database::SpaceRepository;
use tachyon_database::space::{CreateSpaceRequest, UpdateSpaceRequest};

use super::common::skip_without_db;
use crate::common::setup::{
    create_test_pool, create_test_space, create_test_user, setup_database, teardown_test_user,
};

#[tokio::test]
async fn test_create_space() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let user = create_test_user(&pool).await;
    let repo = SpaceRepository::new(pool.clone());

    let space = repo
        .create(
            &user.id.as_str(),
            CreateSpaceRequest {
                name: "My Test Space".to_string(),
                description: Some("A test space".to_string()),
                icon: Some("star".to_string()),
                color: Some("#FF5733".to_string()),
                parent_id: None,
                visibility: Some("private".to_string()),
            },
        )
        .await
        .expect("Failed to create space");

    assert!(!space.id.is_empty());
    assert_eq!(space.name, "My Test Space");
    assert_eq!(space.description.as_deref(), Some("A test space"));
    assert_eq!(space.owner_id, user.id.as_str());

    teardown_test_user(&pool, &user.username).await;
}

#[tokio::test]
async fn test_get_space_by_id() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let user = create_test_user(&pool).await;
    let space = create_test_space(&pool, &user.id.as_str()).await;
    let repo = SpaceRepository::new(pool.clone());

    let fetched = repo
        .get_by_id(&space.id)
        .await
        .expect("Failed to get space by ID");
    assert_eq!(fetched.id, space.id);
    assert_eq!(fetched.name, space.name);

    teardown_test_user(&pool, &user.username).await;
}

#[tokio::test]
async fn test_list_root_spaces() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let user = create_test_user(&pool).await;
    create_test_space(&pool, &user.id.as_str()).await;
    create_test_space(&pool, &user.id.as_str()).await;

    let repo = SpaceRepository::new(pool.clone());
    let spaces = repo
        .list_root_spaces(&user.id.as_str(), Some(10))
        .await
        .expect("Failed to list root spaces");

    assert!(spaces.len() >= 2, "Should have at least 2 root spaces");

    teardown_test_user(&pool, &user.username).await;
}

#[tokio::test]
async fn test_list_spaces_with_filters() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let user = create_test_user(&pool).await;
    create_test_space(&pool, &user.id.as_str()).await;

    let repo = SpaceRepository::new(pool.clone());
    let spaces = repo
        .list(Some(&user.id.as_str()), None, None, None, Some(10), None)
        .await
        .expect("Failed to list spaces");

    assert!(!spaces.is_empty());

    teardown_test_user(&pool, &user.username).await;
}

#[tokio::test]
async fn test_create_child_space() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let user = create_test_user(&pool).await;
    let parent = create_test_space(&pool, &user.id.as_str()).await;
    let repo = SpaceRepository::new(pool.clone());

    let child = repo
        .create(
            &user.id.as_str(),
            CreateSpaceRequest {
                name: "Child Space".to_string(),
                description: Some("A child space".to_string()),
                icon: None,
                color: None,
                parent_id: Some(parent.id.clone()),
                visibility: None,
            },
        )
        .await
        .expect("Failed to create child space");

    assert!(!child.id.is_empty());
    assert_eq!(child.parent_id.as_deref(), Some(parent.id.as_str()));

    let children = repo
        .list_child_spaces(&parent.id, &user.id.as_str())
        .await
        .expect("Failed to list child spaces");

    assert_eq!(children.len(), 1);
    assert_eq!(children[0].id, child.id);

    teardown_test_user(&pool, &user.username).await;
}

#[tokio::test]
async fn test_update_space() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let user = create_test_user(&pool).await;
    let space = create_test_space(&pool, &user.id.as_str()).await;
    let repo = SpaceRepository::new(pool.clone());

    let updated = repo
        .update(
            &space.id,
            UpdateSpaceRequest {
                name: Some("Updated Space".to_string()),
                description: Some("Updated desc".to_string()),
                icon: Some("rocket".to_string()),
                color: None,
                parent_id: None,
                visibility: Some("public".to_string()),
                sort_order: Some(5),
            },
        )
        .await
        .expect("Failed to update space");

    assert_eq!(updated.name, "Updated Space");
    assert_eq!(updated.description.as_deref(), Some("Updated desc"));
    assert_eq!(updated.visibility, "public");
    assert_eq!(updated.sort_order, 5);

    teardown_test_user(&pool, &user.username).await;
}

#[tokio::test]
async fn test_delete_space() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let user = create_test_user(&pool).await;
    let space = create_test_space(&pool, &user.id.as_str()).await;
    let repo = SpaceRepository::new(pool.clone());

    repo.delete(&space.id)
        .await
        .expect("Failed to delete space");

    let result = repo.get_by_id(&space.id).await;
    assert!(result.is_err(), "Deleted space should not be found");

    teardown_test_user(&pool, &user.username).await;
}

#[tokio::test]
async fn test_space_count() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let user = create_test_user(&pool).await;
    create_test_space(&pool, &user.id.as_str()).await;
    create_test_space(&pool, &user.id.as_str()).await;

    let repo = SpaceRepository::new(pool.clone());
    let count = repo
        .count(Some(&user.id.as_str()))
        .await
        .expect("Failed to count spaces");
    assert!(count >= 2);

    teardown_test_user(&pool, &user.username).await;
}
