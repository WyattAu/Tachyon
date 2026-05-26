//! Self-serve signup endpoint.

use axum::{extract::State, http::StatusCode, response::Json, routing::post, Router};
use serde::{Deserialize, Serialize};
use tachyon_core::{User, UserRole};
use tachyon_database::{
    DatabasePool, OrganizationRepository, SubscriptionRepository, UserRepository,
};

use crate::audit::AuditLogger;

#[derive(Debug, Clone)]
pub struct SignupState {
    pub pool: DatabasePool,
    pub audit_logger: AuditLogger,
}

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: String,
    pub plan: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SignupResponse {
    pub success: bool,
    pub user_id: String,
    pub org_id: String,
    pub access_token: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct SignupErrorResponse {
    pub code: String,
    pub message: String,
}

pub async fn signup(
    State(state): State<SignupState>,
    Json(req): Json<SignupRequest>,
) -> Result<Json<SignupResponse>, (StatusCode, Json<SignupErrorResponse>)> {
    if req.username.len() < 3 || req.username.len() > 50 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(SignupErrorResponse {
                code: "VALIDATION_ERROR".to_string(),
                message: "Username must be between 3 and 50 characters".to_string(),
            }),
        ));
    }

    if !req.email.contains('@') || !req.email.contains('.') {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(SignupErrorResponse {
                code: "VALIDATION_ERROR".to_string(),
                message: "Invalid email format".to_string(),
            }),
        ));
    }

    if req.password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(SignupErrorResponse {
                code: "VALIDATION_ERROR".to_string(),
                message: "Password must be at least 8 characters".to_string(),
            }),
        ));
    }

    if req.display_name.is_empty() || req.display_name.len() > 100 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(SignupErrorResponse {
                code: "VALIDATION_ERROR".to_string(),
                message: "Display name must be between 1 and 100 characters".to_string(),
            }),
        ));
    }

    if !req
        .username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(SignupErrorResponse {
                code: "VALIDATION_ERROR".to_string(),
                message:
                    "Username may only contain alphanumeric characters, underscores, and hyphens"
                        .to_string(),
            }),
        ));
    }

    let user_repo = UserRepository::new(state.pool.clone());
    if user_repo.get_by_email(&req.email).await.is_ok() {
        return Err((
            StatusCode::CONFLICT,
            Json(SignupErrorResponse {
                code: "CONFLICT".to_string(),
                message: "Email already registered".to_string(),
            }),
        ));
    }

    let user_id = tachyon_core::generate_user_id();
    let mut user = User::new(user_id, req.username, req.display_name, UserRole::Reader);
    user = user.with_email(req.email.clone());
    if user.set_password(&req.password).is_err() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SignupErrorResponse {
                code: "PASSWORD_ERROR".to_string(),
                message: "Failed to process password".to_string(),
            }),
        ));
    }

    let created = match user_repo.create(&user).await {
        Ok(u) => u,
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("already exists") || msg.contains("duplicate") || msg.contains("unique")
            {
                return Err((
                    StatusCode::CONFLICT,
                    Json(SignupErrorResponse {
                        code: "CONFLICT".to_string(),
                        message: "Username or email already exists".to_string(),
                    }),
                ));
            }
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SignupErrorResponse {
                    code: "INTERNAL_ERROR".to_string(),
                    message: "Failed to create user".to_string(),
                }),
            ));
        }
    };

    let org_repo = OrganizationRepository::new(state.pool.clone());
    let org_name = format!("{}'s Organization", created.display_name);
    let org = match org_repo
        .create(
            &created.id.to_string(),
            tachyon_database::CreateOrganizationRequest {
                name: org_name,
                description: None,
                icon: None,
                logo_url: None,
                default_role: None,
                max_members: None,
            },
        )
        .await
    {
        Ok(o) => o,
        Err(e) => {
            tracing::error!(
                "Failed to create personal org for user {}: {}",
                created.id,
                e
            );
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SignupErrorResponse {
                    code: "INTERNAL_ERROR".to_string(),
                    message: "User created but organization setup failed".to_string(),
                }),
            ));
        }
    };

    let plan_name = req.plan.as_deref().unwrap_or("free");
    let sub_repo = SubscriptionRepository::new(state.pool.clone());
    if let Err(e) = sub_repo
        .create(tachyon_database::CreateSubscriptionRequest {
            organization_id: org.id.clone(),
            plan: plan_name.to_string(),
        })
        .await
    {
        tracing::warn!("Failed to create subscription for org {}: {}", org.id, e);
    }

    let _ = state
        .audit_logger
        .log(
            crate::audit::AuditEvent::new(
                crate::audit::AuditEventType::UserRegistered,
                crate::audit::AuditSeverity::Medium,
                "signup",
                format!("User '{}' signed up", created.username),
            )
            .with_target(created.id.to_string(), "user"),
        )
        .await;

    Ok(Json(SignupResponse {
        success: true,
        user_id: created.id.to_string(),
        org_id: org.id,
        access_token: None,
        message: "Account created successfully".to_string(),
    }))
}

pub fn create_signup_router() -> Router<SignupState> {
    Router::new().route("/signup", post(signup))
}
