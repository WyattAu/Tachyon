use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use super::{DocumentState, ErrorResponse};

#[derive(Debug, Serialize)]
pub struct TemplateResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: String,
}

impl From<tachyon_database::DocumentTemplate> for TemplateResponse {
    fn from(t: tachyon_database::DocumentTemplate) -> Self {
        let tags = t.parse_tags().unwrap_or_default();
        Self {
            id: t.id,
            name: t.name,
            description: t.description,
            content: t.content,
            category: t.category,
            tags,
            created_at: t.created_at.to_rfc3339(),
            updated_at: t.updated_at.to_rfc3339(),
            created_by: t.created_by,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateTemplateBody {
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTemplateBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct TemplateQuery {
    pub category: Option<String>,
}

pub async fn list_templates(
    Query(query): Query<TemplateQuery>,
    State(state): State<DocumentState>,
) -> Result<Json<Vec<TemplateResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let repo = tachyon_database::TemplateRepository::new(state.pool.clone());
    let templates = repo
        .list(query.category.as_deref(), Some(50), None)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "QUERY_ERROR".to_string(),
                    message: format!("Failed to list templates: {}", e),
                    details: None,
                }),
            )
        })?;

    Ok(Json(
        templates.into_iter().map(TemplateResponse::from).collect(),
    ))
}

pub async fn get_template(
    Path(template_id): Path<String>,
    State(state): State<DocumentState>,
) -> Result<Json<TemplateResponse>, (StatusCode, Json<ErrorResponse>)> {
    let repo = tachyon_database::TemplateRepository::new(state.pool.clone());
    let template = repo.get_by_id(&template_id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "NOT_FOUND".to_string(),
                message: format!("Template not found: {}", e),
                details: None,
            }),
        )
    })?;

    Ok(Json(TemplateResponse::from(template)))
}

pub async fn create_template(
    State(state): State<DocumentState>,
    Json(body): Json<CreateTemplateBody>,
) -> Result<Json<TemplateResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user_id = tachyon_core::generate_user_id();
    let repo = tachyon_database::TemplateRepository::new(state.pool.clone());

    let template = repo
        .create(tachyon_database::CreateTemplateRequest {
            name: body.name,
            description: body.description,
            content: body.content,
            category: body.category,
            tags: body.tags,
            created_by: user_id.to_string(),
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "CREATE_ERROR".to_string(),
                    message: format!("Failed to create template: {}", e),
                    details: None,
                }),
            )
        })?;

    tracing::info!("Created template: {}", template.name);
    Ok(Json(TemplateResponse::from(template)))
}

pub async fn update_template(
    Path(template_id): Path<String>,
    State(state): State<DocumentState>,
    Json(body): Json<UpdateTemplateBody>,
) -> Result<Json<TemplateResponse>, (StatusCode, Json<ErrorResponse>)> {
    let repo = tachyon_database::TemplateRepository::new(state.pool.clone());

    let template = repo
        .update(
            &template_id,
            tachyon_database::UpdateTemplateRequest {
                name: body.name,
                description: body.description,
                content: body.content,
                category: body.category,
                tags: body.tags,
            },
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "UPDATE_ERROR".to_string(),
                    message: format!("Failed to update template: {}", e),
                    details: None,
                }),
            )
        })?;

    Ok(Json(TemplateResponse::from(template)))
}

pub async fn delete_template(
    Path(template_id): Path<String>,
    State(state): State<DocumentState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let repo = tachyon_database::TemplateRepository::new(state.pool.clone());
    repo.delete(&template_id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "NOT_FOUND".to_string(),
                message: format!("Template not found: {}", e),
                details: None,
            }),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}
