//! Citation graph API endpoints.

use axum::{
    extract::{Path, State},
    response::Json,
};
use serde::Deserialize;
use tracing::info;

use crate::error::ServerError;

#[derive(Debug, Clone)]
pub struct CitationState {
    pub pool: tachyon_database::DatabasePool,
}

#[derive(Debug, Deserialize)]
pub struct AddCitationRequest {
    pub target_document_id: String,
    pub context: Option<String>,
}

pub async fn get_references(
    Path(document_id): Path<String>,
    State(state): State<CitationState>,
) -> Result<Json<serde_json::Value>, ServerError> {
    let refs = tachyon_database::citation::get_references(&state.pool, &document_id)
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;
    Ok(Json(
        serde_json::json!({ "references": refs, "total": refs.len() }),
    ))
}

pub async fn get_citations(
    Path(document_id): Path<String>,
    State(state): State<CitationState>,
) -> Result<Json<serde_json::Value>, ServerError> {
    let citations = tachyon_database::citation::get_citations(&state.pool, &document_id)
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;
    Ok(Json(
        serde_json::json!({ "citations": citations, "total": citations.len() }),
    ))
}

pub async fn get_citation_metrics(
    Path(document_id): Path<String>,
    State(state): State<CitationState>,
) -> Result<Json<serde_json::Value>, ServerError> {
    let metrics = tachyon_database::citation::get_document_metrics(&state.pool, &document_id)
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;
    Ok(Json(serde_json::json!(metrics)))
}

pub async fn get_citation_stats(
    State(state): State<CitationState>,
) -> Result<Json<serde_json::Value>, ServerError> {
    let stats = tachyon_database::citation::get_corpus_stats(&state.pool)
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;
    Ok(Json(serde_json::json!(stats)))
}

pub async fn add_citation(
    Path(document_id): Path<String>,
    State(state): State<CitationState>,
    Json(req): Json<AddCitationRequest>,
) -> Result<Json<serde_json::Value>, ServerError> {
    tachyon_database::citation::add_citation(
        &state.pool,
        &document_id,
        &req.target_document_id,
        req.context.as_deref(),
    )
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    info!(
        source = %document_id,
        target = %req.target_document_id,
        "Citation added"
    );
    Ok(Json(serde_json::json!({ "created": true })))
}

pub fn create_citation_router() -> axum::Router<CitationState> {
    axum::Router::new()
        .route(
            "/documents/{document_id}/references",
            axum::routing::get(get_references),
        )
        .route(
            "/documents/{document_id}/citations",
            axum::routing::get(get_citations),
        )
        .route(
            "/documents/{document_id}/citations/metrics",
            axum::routing::get(get_citation_metrics),
        )
        .route(
            "/documents/{document_id}/citations/add",
            axum::routing::post(add_citation),
        )
        .route("/citations/stats", axum::routing::get(get_citation_stats))
}
