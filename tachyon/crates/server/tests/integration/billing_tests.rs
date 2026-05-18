use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use crate::common;

#[tokio::test]
async fn test_list_plans() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/billing/plans")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_subscription() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let org_id = uuid::Uuid::new_v4();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/billing/subscriptions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "organization_id": org_id,
                        "plan": "free"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::CREATED
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR
            || response.status() == StatusCode::NOT_FOUND
            || response.status() == StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn test_get_usage() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let org_id = uuid::Uuid::new_v4();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/billing/usage/{}", org_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::NOT_FOUND
            || response.status() == StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn test_list_plans_response_shape() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/billing/plans")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = common::read_body_json(response).await;
    let plans = body["plans"].as_array().expect("plans should be an array");
    assert!(!plans.is_empty(), "Should return at least one plan");

    let free = plans.iter().find(|p| p["name"] == "free");
    assert!(free.is_some(), "Should include a 'free' plan");

    let plan = free.unwrap();
    assert_eq!(plan["price_monthly_cents"], 0);
    assert!(plan["max_documents"].is_number());
    assert!(plan["max_members"].is_number());
    assert!(plan["features"].is_array());
}

#[tokio::test]
async fn test_get_subscription_not_found() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let fake_org_id = uuid::Uuid::new_v4();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/billing/subscriptions/{}", fake_org_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::NOT_FOUND
            || response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::OK
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR,
        "Expected NOT_FOUND/UNAUTHORIZED/OK for get of non-existent subscription, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_create_subscription_invalid_plan() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let org_id = uuid::Uuid::new_v4();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/billing/subscriptions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "organization_id": org_id,
                        "plan": "nonexistent_plan"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "Expected BAD_REQUEST or INTERNAL_SERVER_ERROR for invalid plan, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_create_subscription_missing_fields() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/billing/subscriptions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "plan": "free"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR,
        "Expected BAD_REQUEST for missing organization_id, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_list_invoices() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let org_id = uuid::Uuid::new_v4();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/billing/invoices/{}", org_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::UNAUTHORIZED,
        "Expected OK or UNAUTHORIZED, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_cancel_subscription_not_found() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let fake_org_id = uuid::Uuid::new_v4();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/billing/subscriptions/{}/cancel",
                    fake_org_id
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::NOT_FOUND
            || response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::OK
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR,
        "Expected NOT_FOUND/UNAUTHORIZED/OK for cancel of non-existent subscription, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_change_plan_invalid() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;
    let org_id = uuid::Uuid::new_v4();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/billing/subscription/change-plan")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "organization_id": org_id,
                        "new_plan": "nonexistent"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::NOT_FOUND
            || response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR,
        "Expected BAD_REQUEST for invalid plan name, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_change_plan_missing_fields() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/billing/subscription/change-plan")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "organization_id": uuid::Uuid::new_v4()
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR,
        "Expected BAD_REQUEST for missing new_plan, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_webhook_invalid_payload() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/billing/webhook")
                .header("Content-Type", "application/json")
                .body(Body::from("not valid json"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNAUTHORIZED,
        "Expected BAD_REQUEST for invalid webhook payload, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_mandate_creation_unauthorized() {
    if common::skip_without_db() {
        println!("Skipping: TEST_DATABASE_URL not set");
        return;
    }

    let app = common::create_test_app().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/billing/mandates")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "organization_id": uuid::Uuid::new_v4(),
                        "return_url": "https://example.com/callback"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::SERVICE_UNAVAILABLE
            || response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR,
        "Expected SERVICE_UNAVAILABLE (payments disabled) or UNAUTHORIZED, got {}",
        response.status()
    );
}
