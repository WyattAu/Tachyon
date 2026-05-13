// Plugin Routes
// REST API endpoints for plugin management

use crate::error::ServerError;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
pub use tachyon_database::UpdatePluginRequest;
use tachyon_database::{CreatePluginRequest, DatabasePool, PluginRepository};
use tachyon_plugin_runtime::PluginRuntime;
use tracing::info;

#[derive(Clone)]
pub struct PluginState {
    pub pool: DatabasePool,
    pub runtime: PluginRuntime,
}

// ============================================================================
// Response / Request Types
// ============================================================================

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PluginResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub author: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub extension_points: Vec<String>,
    pub manifest: Option<serde_json::Value>,
    pub runtime_type: String,
    pub entry_point: Option<String>,
    pub enabled: bool,
    pub installed_at: String,
    pub updated_at: String,
    pub installed_by: Option<String>,
}

impl From<tachyon_database::Plugin> for PluginResponse {
    fn from(p: tachyon_database::Plugin) -> Self {
        let ext_points = p.parse_extension_points().unwrap_or_default();
        let manifest = p.parse_manifest().ok().flatten();
        Self {
            id: p.id,
            name: p.name,
            description: p.description,
            version: p.version,
            author: p.author,
            homepage: p.homepage,
            license: p.license,
            extension_points: ext_points,
            manifest,
            runtime_type: p.runtime_type,
            entry_point: p.entry_point,
            enabled: p.enabled,
            installed_at: p.installed_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
            installed_by: p.installed_by,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreatePluginBody {
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub author: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub extension_points: Option<Vec<String>>,
    pub manifest: Option<serde_json::Value>,
    pub runtime_type: Option<String>,
    pub entry_point: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdatePluginBody {
    pub description: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub extension_points: Option<Vec<String>>,
    pub manifest: Option<serde_json::Value>,
    pub entry_point: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct PluginQuery {
    pub enabled: Option<bool>,
    pub runtime_type: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ============================================================================
// Handlers
// ============================================================================

/// List installed plugins.
///
/// `GET /api/v1/plugins`
///
/// Supports `enabled`, `runtime_type`, `limit`, and `offset` query filters.
#[utoipa::path(
    get,
    path = "/plugins",
    params(
        PluginQuery,
    ),
    responses(
        (status = 200, description = "List of plugins", body = Vec<PluginResponse>),
        (status = 500, description = "Internal server error"),
    ),
    tag = "plugins",
    security(("bearer_auth" = [])),
)]
pub async fn list_plugins(
    Query(query): Query<PluginQuery>,
    State(state): State<PluginState>,
) -> Result<Json<Vec<PluginResponse>>, ServerError> {
    let repo = PluginRepository::new(state.pool.clone());
    let plugins = repo
        .list(
            query.enabled,
            query.runtime_type.as_deref(),
            query.limit,
            query.offset,
        )
        .await
        .map_err(|e| ServerError::internal(format!("Failed to list plugins: {}", e)))?;

    Ok(Json(
        plugins.into_iter().map(PluginResponse::from).collect(),
    ))
}

/// Get a plugin by ID.
///
/// `GET /api/v1/plugins/{plugin_id}`
#[utoipa::path(
    get,
    path = "/plugins/{plugin_id}",
    params(
        ("plugin_id" = String, Path, description = "Plugin ID"),
    ),
    responses(
        (status = 200, description = "Plugin details", body = PluginResponse),
        (status = 404, description = "Plugin not found"),
    ),
    tag = "plugins",
    security(("bearer_auth" = [])),
)]
pub async fn get_plugin(
    Path(plugin_id): Path<String>,
    State(state): State<PluginState>,
) -> Result<Json<PluginResponse>, ServerError> {
    let repo = PluginRepository::new(state.pool.clone());
    let plugin = repo
        .get_by_id(&plugin_id)
        .await
        .map_err(|e| ServerError::not_found("plugin", &e.to_string()))?;

    Ok(Json(PluginResponse::from(plugin)))
}

/// Install a new plugin.
///
/// `POST /api/v1/plugins`
///
/// Requires `name` and `version` fields.
#[utoipa::path(
    post,
    path = "/plugins",
    request_body(content = CreatePluginBody, description = "Plugin installation request"),
    responses(
        (status = 200, description = "Plugin installed", body = PluginResponse),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "plugins",
    security(("bearer_auth" = [])),
)]
pub async fn create_plugin(
    State(state): State<PluginState>,
    Json(body): Json<CreatePluginBody>,
) -> Result<Json<PluginResponse>, ServerError> {
    if body.name.trim().is_empty() {
        return Err(ServerError::bad_request("Plugin name is required"));
    }
    if body.version.trim().is_empty() {
        return Err(ServerError::bad_request("Plugin version is required"));
    }

    let repo = PluginRepository::new(state.pool.clone());
    let plugin = repo
        .create(CreatePluginRequest {
            name: body.name,
            description: body.description,
            version: body.version,
            author: body.author,
            homepage: body.homepage,
            license: body.license,
            extension_points: body.extension_points,
            manifest: body.manifest,
            runtime_type: body.runtime_type,
            entry_point: body.entry_point,
            enabled: body.enabled,
            installed_by: None,
        })
        .await
        .map_err(|e| ServerError::internal(format!("Failed to create plugin: {}", e)))?;

    info!("Installed plugin: {} v{}", plugin.name, plugin.version);
    Ok(Json(PluginResponse::from(plugin)))
}

/// Update a plugin.
///
/// `PUT /api/v1/plugins/{plugin_id}`
///
/// Accepts partial updates for description, version, author, homepage, license,
/// extension points, manifest, entry point, and enabled state.
#[utoipa::path(
    put,
    path = "/plugins/{plugin_id}",
    params(
        ("plugin_id" = String, Path, description = "Plugin ID"),
    ),
    request_body(content = UpdatePluginBody, description = "Plugin update request"),
    responses(
        (status = 200, description = "Plugin updated", body = PluginResponse),
        (status = 404, description = "Plugin not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "plugins",
    security(("bearer_auth" = [])),
)]
pub async fn update_plugin(
    Path(plugin_id): Path<String>,
    State(state): State<PluginState>,
    Json(body): Json<UpdatePluginBody>,
) -> Result<Json<PluginResponse>, ServerError> {
    let repo = PluginRepository::new(state.pool.clone());

    let plugin = repo
        .update(
            &plugin_id,
            UpdatePluginRequest {
                description: body.description,
                version: body.version,
                author: body.author,
                homepage: body.homepage,
                license: body.license,
                extension_points: body.extension_points,
                manifest: body.manifest,
                entry_point: body.entry_point,
                enabled: body.enabled,
            },
        )
        .await
        .map_err(|e| ServerError::internal(format!("Failed to update plugin: {}", e)))?;

    info!("Updated plugin: {}", plugin.name);
    Ok(Json(PluginResponse::from(plugin)))
}

/// Uninstall a plugin.
///
/// `DELETE /api/v1/plugins/{plugin_id}`
#[utoipa::path(
    delete,
    path = "/plugins/{plugin_id}",
    params(
        ("plugin_id" = String, Path, description = "Plugin ID"),
    ),
    responses(
        (status = 204, description = "Plugin uninstalled"),
        (status = 404, description = "Plugin not found"),
    ),
    tag = "plugins",
    security(("bearer_auth" = [])),
)]
pub async fn delete_plugin(
    Path(plugin_id): Path<String>,
    State(state): State<PluginState>,
) -> Result<StatusCode, ServerError> {
    let repo = PluginRepository::new(state.pool.clone());
    repo.delete(&plugin_id)
        .await
        .map_err(|e| ServerError::not_found("plugin", &e.to_string()))?;

    info!("Uninstalled plugin: {}", plugin_id);
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Invoke Hook
// ============================================================================

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct InvokeHookRequest {
    pub hook: String,
    pub input: serde_json::Value,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_timeout() -> u64 {
    5000
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct InvokeHookResponse {
    pub results: Vec<serde_json::Value>,
    pub hook: String,
    pub plugins_invoked: usize,
}

/// POST /api/v1/plugins/invoke — Invoke a hook across all enabled plugins
#[utoipa::path(
    post,
    path = "/plugins/invoke",
    request_body(content = InvokeHookRequest, description = "Hook invocation request"),
    responses(
        (status = 200, description = "Hook results", body = InvokeHookResponse),
    ),
    tag = "plugins",
    security(("bearer_auth" = [])),
)]
pub async fn invoke_hook(
    State(state): State<PluginState>,
    Json(req): Json<InvokeHookRequest>,
) -> Result<Json<InvokeHookResponse>, ServerError> {
    let results = state
        .runtime
        .invoke_hook(&req.hook, req.input, req.timeout_ms);

    let plugins_invoked = results.len();
    info!(
        "Hook '{}' invoked across {} plugins",
        req.hook, plugins_invoked
    );

    let results: Vec<serde_json::Value> = results
        .into_iter()
        .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
        .collect();

    Ok(Json(InvokeHookResponse {
        results,
        hook: req.hook,
        plugins_invoked,
    }))
}

// ============================================================================
// Router
// ============================================================================

pub fn create_plugin_router_with_state(state: PluginState) -> axum::Router {
    axum::Router::new()
        .route("/plugins", axum::routing::get(list_plugins))
        .route("/plugins", axum::routing::post(create_plugin))
        .route("/plugins/invoke", axum::routing::post(invoke_hook))
        .route("/plugins/{plugin_id}", axum::routing::get(get_plugin))
        .route("/plugins/{plugin_id}", axum::routing::put(update_plugin))
        .route("/plugins/{plugin_id}", axum::routing::delete(delete_plugin))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_plugin_body_deserialization() {
        let body = CreatePluginBody {
            name: "test-plugin".to_string(),
            description: Some("A test plugin".to_string()),
            version: "0.1.0".to_string(),
            author: Some("Test Author".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            extension_points: Some(vec!["editor:command".to_string()]),
            manifest: None,
            runtime_type: Some("wasm".to_string()),
            entry_point: None,
            enabled: Some(false),
        };
        assert_eq!(body.name, "test-plugin");
        assert_eq!(body.version, "0.1.0");
        assert_eq!(body.extension_points.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_plugin_response_serialization() {
        let resp = PluginResponse {
            id: "00000000-0000-0000-0000-000000000000".to_string(),
            name: "test-plugin".to_string(),
            description: Some("A test".to_string()),
            version: "0.1.0".to_string(),
            author: None,
            homepage: None,
            license: Some("MIT".to_string()),
            extension_points: vec!["editor:command".to_string()],
            manifest: None,
            runtime_type: "wasm".to_string(),
            entry_point: None,
            enabled: true,
            installed_at: "2026-04-13T00:00:00+00:00".to_string(),
            updated_at: "2026-04-13T00:00:00+00:00".to_string(),
            installed_by: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("test-plugin"));
        assert!(json.contains("MIT"));
        assert!(json.contains("editor:command"));
    }
}
