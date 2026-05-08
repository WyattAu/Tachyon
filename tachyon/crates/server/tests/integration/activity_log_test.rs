use tachyon_database::{ActivityRepository, CreateActivityEvent};

use crate::common::setup::{
    create_test_pool, create_test_user, setup_database, teardown_test_user,
};

fn skip_without_db() -> bool {
    std::env::var("DATABASE_URL").is_err() && std::env::var("TEST_DATABASE_URL").is_err()
}

#[tokio::test]
async fn test_create_activity_event() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;

    let user = create_test_user(&pool).await;
    let target_id = uuid::Uuid::new_v4();

    let event = ActivityRepository::create(
        &pool,
        CreateActivityEvent {
            actor_id: user.id.as_uuid(),
            event_type: "document_created".to_string(),
            target_type: "document".to_string(),
            target_id,
            description: "Created a new test document".to_string(),
            metadata: Some(serde_json::json!({
                "document_title": "Test Activity Doc",
                "word_count": 150
            })),
        },
    )
    .await
    .expect("Failed to create activity event");

    assert!(!event.id.is_nil());
    assert_eq!(event.event_type, "document_created");
    assert_eq!(event.target_type, "document");
    assert_eq!(event.description, "Created a new test document");
    assert!(event.metadata["document_title"].is_string());

    teardown_test_user(&pool, &user.username).await;
}

#[tokio::test]
async fn test_list_activities() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;

    let user = create_test_user(&pool).await;

    for i in 0..3 {
        ActivityRepository::create(
            &pool,
            CreateActivityEvent {
                actor_id: user.id.as_uuid(),
                event_type: format!("event_type_{}", i),
                target_type: "document".to_string(),
                target_id: uuid::Uuid::new_v4(),
                description: format!("Activity event number {}", i),
                metadata: None,
            },
        )
        .await
        .expect("Failed to create activity event");
    }

    let events = ActivityRepository::list_recent(&pool, 10, 0)
        .await
        .expect("Failed to list activities");

    assert!(events.len() >= 3, "Should have at least 3 activity events");

    teardown_test_user(&pool, &user.username).await;
}

#[tokio::test]
async fn test_activity_ordering_most_recent_first() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;

    let user = create_test_user(&pool).await;

    ActivityRepository::create(
        &pool,
        CreateActivityEvent {
            actor_id: user.id.as_uuid(),
            event_type: "first_event".to_string(),
            target_type: "document".to_string(),
            target_id: uuid::Uuid::new_v4(),
            description: "First event".to_string(),
            metadata: None,
        },
    )
    .await
    .expect("Failed to create first event");

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    ActivityRepository::create(
        &pool,
        CreateActivityEvent {
            actor_id: user.id.as_uuid(),
            event_type: "second_event".to_string(),
            target_type: "document".to_string(),
            target_id: uuid::Uuid::new_v4(),
            description: "Second event".to_string(),
            metadata: None,
        },
    )
    .await
    .expect("Failed to create second event");

    let events = ActivityRepository::list_recent(&pool, 10, 0)
        .await
        .expect("Failed to list activities");

    assert!(events.len() >= 2);
    assert_eq!(
        events[0].event_type, "second_event",
        "Most recent event should be first"
    );
    assert!(
        events[0].created_at >= events[1].created_at,
        "Events should be ordered by created_at DESC"
    );

    teardown_test_user(&pool, &user.username).await;
}

#[tokio::test]
async fn test_list_activities_by_target() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;

    let user = create_test_user(&pool).await;
    let target_id = uuid::Uuid::new_v4();

    ActivityRepository::create(
        &pool,
        CreateActivityEvent {
            actor_id: user.id.as_uuid(),
            event_type: "document_updated".to_string(),
            target_type: "document".to_string(),
            target_id,
            description: "Updated the document".to_string(),
            metadata: None,
        },
    )
    .await
    .expect("Failed to create event");

    ActivityRepository::create(
        &pool,
        CreateActivityEvent {
            actor_id: user.id.as_uuid(),
            event_type: "document_commented".to_string(),
            target_type: "document".to_string(),
            target_id,
            description: "Commented on the document".to_string(),
            metadata: None,
        },
    )
    .await
    .expect("Failed to create event");

    let events = ActivityRepository::list_by_target(&pool, "document", target_id, 10)
        .await
        .expect("Failed to list activities by target");

    assert_eq!(events.len(), 2, "Should find 2 events for this target");
    for event in &events {
        assert_eq!(event.target_id, target_id);
        assert_eq!(event.target_type, "document");
    }

    teardown_test_user(&pool, &user.username).await;
}

#[tokio::test]
async fn test_list_activities_by_actor() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;

    let user = create_test_user(&pool).await;
    let other_user = create_test_user(&pool).await;

    ActivityRepository::create(
        &pool,
        CreateActivityEvent {
            actor_id: user.id.as_uuid(),
            event_type: "user_action".to_string(),
            target_type: "document".to_string(),
            target_id: uuid::Uuid::new_v4(),
            description: "User action".to_string(),
            metadata: None,
        },
    )
    .await
    .expect("Failed to create user event");

    ActivityRepository::create(
        &pool,
        CreateActivityEvent {
            actor_id: other_user.id.as_uuid(),
            event_type: "other_action".to_string(),
            target_type: "document".to_string(),
            target_id: uuid::Uuid::new_v4(),
            description: "Other user action".to_string(),
            metadata: None,
        },
    )
    .await
    .expect("Failed to create other user event");

    let events = ActivityRepository::list_by_actor(&pool, user.id.as_uuid(), 10)
        .await
        .expect("Failed to list activities by actor");

    assert!(!events.is_empty());
    for event in &events {
        assert_eq!(event.actor_id, user.id.as_uuid());
    }

    teardown_test_user(&pool, &user.username).await;
    teardown_test_user(&pool, &other_user.username).await;
}

#[tokio::test]
async fn test_activity_event_without_metadata() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;

    let user = create_test_user(&pool).await;

    let event = ActivityRepository::create(
        &pool,
        CreateActivityEvent {
            actor_id: user.id.as_uuid(),
            event_type: "simple_event".to_string(),
            target_type: "space".to_string(),
            target_id: uuid::Uuid::new_v4(),
            description: "A simple event without metadata".to_string(),
            metadata: None,
        },
    )
    .await
    .expect("Failed to create activity event");

    assert_eq!(event.metadata, serde_json::json!({}));

    teardown_test_user(&pool, &user.username).await;
}

#[tokio::test]
async fn test_list_activities_with_pagination() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;

    let user = create_test_user(&pool).await;

    for i in 0..5 {
        ActivityRepository::create(
            &pool,
            CreateActivityEvent {
                actor_id: user.id.as_uuid(),
                event_type: format!("paginated_{}", i),
                target_type: "document".to_string(),
                target_id: uuid::Uuid::new_v4(),
                description: format!("Paginated event {}", i),
                metadata: None,
            },
        )
        .await
        .expect("Failed to create event");
    }

    let page1 = ActivityRepository::list_recent(&pool, 2, 0)
        .await
        .expect("Failed to list page 1");
    assert_eq!(page1.len(), 2);

    let page2 = ActivityRepository::list_recent(&pool, 2, 2)
        .await
        .expect("Failed to list page 2");
    assert_eq!(page2.len(), 2);

    teardown_test_user(&pool, &user.username).await;
}
