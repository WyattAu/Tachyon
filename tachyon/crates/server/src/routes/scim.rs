use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tachyon_core::id::UserId;
use tachyon_core::types::user::{User, UserRole};
use tachyon_database::{DatabasePool, TeamRepository};
use tracing::{info, instrument};

// ============================================================================
// SCIM Constants
// ============================================================================

const SCIM_USER_SCHEMA: &str = "urn:ietf:params:scim:schemas:core:2.0:User";
const SCIM_GROUP_SCHEMA: &str = "urn:ietf:params:scim:schemas:core:2.0:Group";
const SCIM_LIST_RESPONSE_SCHEMA: &str = "urn:ietf:params:scim:api:messages:2.0:ListResponse";
const SCIM_ERROR_SCHEMA: &str = "urn:ietf:params:scim:api:messages:2.0:Error";
#[allow(dead_code)]
const SCIM_PATCH_OP_SCHEMA: &str = "urn:ietf:params:scim:api:messages:2.0:PatchOp";
const SCIM_SERVICE_PROVIDER_CONFIG_SCHEMA: &str =
    "urn:ietf:params:scim:schemas:core:2.0:ServiceProviderConfig";
const SCIM_SCHEMA_SCHEMA: &str = "urn:ietf:params:scim:schemas:core:2.0:Schema";
const SCIM_CONTENT_TYPE: &str = "application/scim+json";

// ============================================================================
// SCIM Data Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimName {
    #[serde(rename = "givenName", skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,
    #[serde(rename = "familyName", skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimEmail {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimPhoneNumber {
    pub value: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub phone_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimRole {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimGroupRef {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub ref_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimMeta {
    pub resource_type: String,
    pub created: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScimUser {
    #[serde(rename = "schemas")]
    pub schemas: Vec<String>,
    pub id: String,
    #[serde(rename = "externalId", skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(rename = "userName")]
    pub user_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<ScimName>,
    #[serde(rename = "displayName", skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    pub emails: Vec<ScimEmail>,
    #[serde(rename = "phoneNumbers", skip_serializing_if = "Option::is_none")]
    pub phone_numbers: Option<Vec<ScimPhoneNumber>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<ScimRole>>,
    pub active: Option<bool>,
    pub groups: Vec<ScimGroupRef>,
    pub meta: Option<ScimMeta>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScimListResponse<T: Serialize> {
    pub schemas: Vec<String>,
    #[serde(rename = "totalResults")]
    pub total_results: usize,
    #[serde(rename = "startIndex", skip_serializing_if = "Option::is_none")]
    pub start_index: Option<usize>,
    #[serde(rename = "itemsPerPage", skip_serializing_if = "Option::is_none")]
    pub items_per_page: Option<usize>,
    #[serde(rename = "Resources")]
    pub resources: Vec<T>,
}

#[derive(Debug, Serialize)]
pub struct ScimError {
    pub schemas: Vec<String>,
    pub detail: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimMember {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub ref_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimGroup {
    #[serde(rename = "schemas")]
    pub schemas: Vec<String>,
    pub id: String,
    #[serde(rename = "externalId", skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<ScimMember>>,
    pub meta: Option<ScimMeta>,
}

impl ScimGroup {
    pub fn from_team(team: &tachyon_database::Team, base_url: Option<&str>) -> Self {
        let meta = Some(ScimMeta {
            resource_type: "Group".to_string(),
            created: team.created_at,
            last_modified: Some(team.updated_at),
            version: None,
            location: base_url.map(|b| format!("{}/api/v1/scim/v2/Groups/{}", b, team.id)),
        });

        ScimGroup {
            schemas: vec![SCIM_GROUP_SCHEMA.to_string()],
            id: team.id.clone(),
            external_id: None,
            display_name: team.name.clone(),
            members: None,
            meta,
        }
    }

    pub fn from_team_with_members(
        team: &tachyon_database::Team,
        members: &[tachyon_database::TeamMember],
        base_url: Option<&str>,
    ) -> Self {
        let mut group = Self::from_team(team, base_url);
        group.members = Some(
            members
                .iter()
                .map(|m| ScimMember {
                    value: Some(m.user_id.clone()),
                    ref_url: base_url.map(|b| format!("{}/api/v1/scim/v2/Users/{}", b, m.user_id)),
                    display: None,
                })
                .collect(),
        );
        group
    }
}

#[derive(Debug, Serialize)]
pub struct ScimServiceProviderConfig {
    pub schemas: Vec<String>,
    pub patch: ScimPatchSupport,
    pub bulk: ScimBulkSupport,
    pub filter: ScimFilterSupport,
    #[serde(rename = "changePassword")]
    pub change_password: ScimChangePasswordSupport,
    pub sort: ScimSortSupport,
    pub etag: ScimEtagSupport,
    pub authentication_schemes: Vec<ScimAuthScheme>,
}

#[derive(Debug, Serialize)]
pub struct ScimPatchSupport {
    pub supported: bool,
}

#[derive(Debug, Serialize)]
pub struct ScimBulkSupport {
    pub supported: bool,
    pub max_operations: Option<usize>,
    pub max_payload_size: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct ScimFilterSupport {
    pub supported: bool,
    pub max_results: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct ScimChangePasswordSupport {
    pub supported: bool,
}

#[derive(Debug, Serialize)]
pub struct ScimSortSupport {
    pub supported: bool,
}

#[derive(Debug, Serialize)]
pub struct ScimEtagSupport {
    pub supported: bool,
}

#[derive(Debug, Serialize)]
pub struct ScimAuthScheme {
    #[serde(rename = "type")]
    pub scheme_type: String,
    pub name: String,
    pub description: Option<String>,
    pub spec_uri: Option<String>,
    pub documentation_uri: Option<String>,
    pub primary: bool,
}

#[derive(Debug, Serialize)]
pub struct ScimSchema {
    pub schemas: Vec<String>,
    pub id: String,
    pub name: String,
    pub description: String,
    pub attributes: Vec<ScimSchemaAttribute>,
}

#[derive(Debug, Serialize)]
pub struct ScimSchemaAttribute {
    pub name: String,
    #[serde(rename = "type")]
    pub attr_type: String,
    pub multi_valued: bool,
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_exact: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutability: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub returned: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uniqueness: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ScimPatchOp {
    pub op: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct ScimPatchRequest {
    #[serde(rename = "schemas")]
    pub schemas: Vec<String>,
    #[serde(rename = "Operations")]
    pub operations: Vec<ScimPatchOp>,
}

#[derive(Debug, Deserialize)]
pub struct ScimListParams {
    pub filter: Option<String>,
    #[serde(rename = "startIndex", default)]
    pub start_index: Option<usize>,
    #[serde(rename = "count", default)]
    pub count: Option<usize>,
}

// ============================================================================
// SCIM State
// ============================================================================

#[derive(Clone)]
pub struct ScimState {
    pub pool: DatabasePool,
    pub bearer_token: String,
}

// ============================================================================
// Conversion: User <-> ScimUser
// ============================================================================

impl ScimUser {
    pub fn from_user(user: &User, base_url: Option<&str>) -> Self {
        let mut emails = Vec::new();
        if let Some(ref email) = user.email {
            emails.push(ScimEmail {
                value: email.clone(),
                primary: Some(true),
            });
        }

        let name_parts: Vec<&str> = user.display_name.splitn(2, ' ').collect();
        let name = if name_parts.len() >= 2 {
            Some(ScimName {
                given_name: Some(name_parts[0].to_string()),
                family_name: Some(name_parts[1..].join(" ")),
            })
        } else if !user.display_name.is_empty() {
            Some(ScimName {
                given_name: Some(user.display_name.clone()),
                family_name: None,
            })
        } else {
            None
        };

        let meta = Some(ScimMeta {
            resource_type: "User".to_string(),
            created: user.created_at,
            last_modified: Some(user.updated_at),
            version: None,
            location: base_url.map(|b| format!("{}/api/v1/scim/v2/Users/{}", b, user.id)),
        });

        ScimUser {
            schemas: vec![SCIM_USER_SCHEMA.to_string()],
            id: user.id.as_str(),
            external_id: None,
            user_name: user.username.clone(),
            name,
            display_name: if user.display_name.is_empty() {
                None
            } else {
                Some(user.display_name.clone())
            },
            emails,
            phone_numbers: None,
            roles: Some(vec![ScimRole {
                value: user.permissions.role.to_string(),
            }]),
            active: user.is_active,
            groups: Vec::new(),
            meta,
        }
    }

    pub fn to_user_create(&self) -> Result<User, String> {
        if self.user_name.is_empty() {
            return Err("userName is required".to_string());
        }

        let user_id = tachyon_core::generate_user_id();
        let display_name = self
            .display_name
            .clone()
            .or_else(|| {
                self.name.as_ref().map(|n| {
                    let mut parts = Vec::new();
                    if let Some(ref given) = n.given_name {
                        parts.push(given.as_str());
                    }
                    if let Some(ref family) = n.family_name {
                        parts.push(family.as_str());
                    }
                    parts.join(" ")
                })
            })
            .unwrap_or_else(|| self.user_name.clone());

        let primary_email = self
            .emails
            .iter()
            .find(|e| e.primary.unwrap_or(false))
            .or_else(|| self.emails.first())
            .map(|e| e.value.clone());

        let role = self
            .roles
            .as_ref()
            .and_then(|roles| roles.first())
            .map(|r| parse_role(&r.value))
            .unwrap_or(UserRole::Reader);

        let mut user = User::new(user_id, self.user_name.clone(), display_name, role);
        user.email = primary_email;
        user.is_active = Some(self.active.unwrap_or(true));

        match user.set_password(&generate_random_password()) {
            Ok(()) => {}
            Err(e) => return Err(format!("Failed to set SCIM user password: {}", e)),
        }

        Ok(user)
    }

    pub fn to_user_update(&self, existing: &User) -> User {
        let mut user = existing.clone();

        if let Some(ref name) = self.name {
            let mut parts = Vec::new();
            if let Some(ref given) = name.given_name {
                parts.push(given.as_str());
            }
            if let Some(ref family) = name.family_name {
                parts.push(family.as_str());
            }
            if !parts.is_empty() {
                user.display_name = parts.join(" ");
            }
        }

        if let Some(ref display_name) = self.display_name {
            user.display_name = display_name.clone();
        }

        if !self.emails.is_empty() {
            if let Some(primary) = self.emails.iter().find(|e| e.primary.unwrap_or(false)) {
                user.email = Some(primary.value.clone());
            } else if let Some(first) = self.emails.first() {
                user.email = Some(first.value.clone());
            }
        }

        if let Some(active) = self.active {
            user.is_active = Some(active);
        }

        if let Some(ref roles) = self.roles {
            if let Some(role) = roles.first() {
                user.permissions.role = parse_role(&role.value);
            }
        }

        user.touch();
        user
    }
}

fn parse_role(role_str: &str) -> UserRole {
    match role_str.to_lowercase().as_str() {
        "admin" => UserRole::Admin,
        "editor" => UserRole::Editor,
        "writer" => UserRole::Writer,
        _ => UserRole::Reader,
    }
}

fn generate_random_password() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let charset: Vec<u8> = (b'a'..=b'z')
        .chain(b'A'..=b'Z')
        .chain(b'0'..=b'9')
        .collect();
    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset[idx] as char
        })
        .collect()
}

// ============================================================================
// SCIM Filter Parsing
// ============================================================================

#[derive(Debug)]
enum ScimFilter {
    UserNameEq(String),
    DisplayNameCo(String),
    ActiveEq(bool),
    EmailEq(String),
}

impl ScimFilter {
    fn matches(&self, user: &User) -> bool {
        match self {
            ScimFilter::UserNameEq(name) => user.username.eq_ignore_ascii_case(name),
            ScimFilter::DisplayNameCo(substring) => user
                .display_name
                .to_lowercase()
                .contains(&substring.to_lowercase()),
            ScimFilter::ActiveEq(active) => user.is_active.unwrap_or(true) == *active,
            ScimFilter::EmailEq(email) => user
                .email
                .as_ref()
                .map(|e| e.eq_ignore_ascii_case(email))
                .unwrap_or(false),
        }
    }
}

fn parse_filter(filter_str: &str) -> Result<ScimFilter, String> {
    let s = filter_str.trim();

    if let Some(rest) = s.strip_prefix("userName eq ") {
        let value = rest.trim().trim_matches('"').to_string();
        if value.is_empty() {
            return Err("userName value cannot be empty".to_string());
        }
        return Ok(ScimFilter::UserNameEq(value));
    }

    if let Some(rest) = s.strip_prefix("displayName co ") {
        let value = rest.trim().trim_matches('"').to_string();
        if value.is_empty() {
            return Err("displayName value cannot be empty".to_string());
        }
        return Ok(ScimFilter::DisplayNameCo(value));
    }

    if let Some(rest) = s.strip_prefix("active eq ") {
        let value = rest.trim();
        let active = match value {
            "true" => true,
            "false" => false,
            _ => return Err(format!("active must be 'true' or 'false', got: {}", value)),
        };
        return Ok(ScimFilter::ActiveEq(active));
    }

    if let Some(rest) = s.strip_prefix("emails.value eq ") {
        let value = rest.trim().trim_matches('"').to_string();
        if value.is_empty() {
            return Err("emails.value cannot be empty".to_string());
        }
        return Ok(ScimFilter::EmailEq(value));
    }

    Err(format!("Unsupported SCIM filter: {}", filter_str))
}

// ============================================================================
// SCIM Error Helpers
// ============================================================================

fn scim_error_response(status: StatusCode, detail: &str) -> Response {
    let body = ScimError {
        schemas: vec![SCIM_ERROR_SCHEMA.to_string()],
        detail: detail.to_string(),
        status: status.as_u16().to_string(),
    };
    (
        status,
        [(axum::http::header::CONTENT_TYPE, SCIM_CONTENT_TYPE)],
        Json(body),
    )
        .into_response()
}

fn scim_json_response<T: Serialize>(status: StatusCode, body: &T) -> Response {
    (
        status,
        [(axum::http::header::CONTENT_TYPE, SCIM_CONTENT_TYPE)],
        Json(body),
    )
        .into_response()
}

// ============================================================================
// Bearer Token Auth Middleware (inline)
// ============================================================================

#[allow(clippy::result_large_err)]
fn verify_bearer_token(headers: &HeaderMap, expected_token: &str) -> Result<(), Response> {
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    match auth_header {
        None => Err(scim_error_response(
            StatusCode::UNAUTHORIZED,
            "Authorization header required",
        )),
        Some(header) => {
            let token = header.strip_prefix("Bearer ").unwrap_or(header);
            if token != expected_token {
                Err(scim_error_response(
                    StatusCode::UNAUTHORIZED,
                    "Invalid bearer token",
                ))
            } else {
                Ok(())
            }
        }
    }
}

// ============================================================================
// SCIM Handlers
// ============================================================================

#[instrument(skip(state, headers))]
pub async fn list_users(
    State(state): State<ScimState>,
    Query(params): Query<ScimListParams>,
    headers: HeaderMap,
) -> Response {
    if let Err(e) = verify_bearer_token(&headers, &state.bearer_token) {
        return e;
    }

    let repo = tachyon_database::UserRepository::new(state.pool.clone());
    let page_size = params.count.unwrap_or(100).min(100);
    let page = ((params.start_index.unwrap_or(1).max(1) - 1) / page_size) + 1;

    let (users, total) = match repo.list(page, page_size, None).await {
        Ok(result) => result,
        Err(e) => {
            return scim_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Database error: {}", e),
            );
        }
    };

    let base_url: Option<String> = None;
    let mut resources: Vec<ScimUser> = users
        .iter()
        .map(|u| ScimUser::from_user(u, base_url.as_deref()))
        .collect();

    if let Some(ref filter_str) = params.filter {
        match parse_filter(filter_str) {
            Ok(filter) => {
                resources.retain(|scim_user| {
                    users
                        .iter()
                        .any(|u| u.username == scim_user.user_name && filter.matches(u))
                });
            }
            Err(e) => {
                return scim_error_response(StatusCode::BAD_REQUEST, &e);
            }
        }
    }

    let list_response = ScimListResponse {
        schemas: vec![SCIM_LIST_RESPONSE_SCHEMA.to_string()],
        total_results: if params.filter.is_some() {
            resources.len()
        } else {
            total as usize
        },
        start_index: params.start_index,
        items_per_page: Some(resources.len()),
        resources,
    };

    scim_json_response(StatusCode::OK, &list_response)
}

#[instrument(skip(state, headers))]
pub async fn get_user(
    State(state): State<ScimState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response {
    if let Err(e) = verify_bearer_token(&headers, &state.bearer_token) {
        return e;
    }

    let user_id = match UserId::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return scim_error_response(StatusCode::NOT_FOUND, &format!("User '{}' not found", id));
        }
    };

    let repo = tachyon_database::UserRepository::new(state.pool.clone());
    let user = match repo.get_by_id(&user_id).await {
        Ok(u) => u,
        Err(_) => {
            return scim_error_response(StatusCode::NOT_FOUND, &format!("User '{}' not found", id));
        }
    };

    let scim_user = ScimUser::from_user(&user, None);
    scim_json_response(StatusCode::OK, &scim_user)
}

#[instrument(skip(state, headers))]
pub async fn create_user(
    State(state): State<ScimState>,
    headers: HeaderMap,
    Json(scim_user): Json<ScimUser>,
) -> Response {
    if let Err(e) = verify_bearer_token(&headers, &state.bearer_token) {
        return e;
    }

    if scim_user.user_name.is_empty() {
        return scim_error_response(StatusCode::BAD_REQUEST, "userName is required");
    }

    let repo = tachyon_database::UserRepository::new(state.pool.clone());

    let user = match scim_user.to_user_create() {
        Ok(u) => u,
        Err(e) => {
            return scim_error_response(StatusCode::BAD_REQUEST, &e);
        }
    };

    match repo.get_by_username(&user.username).await {
        Ok(existing) => {
            let updated = scim_user.to_user_update(&existing);
            match repo
                .update(
                    &existing.id,
                    Some(&updated.display_name),
                    updated.email.as_deref(),
                    Some(updated.permissions.role),
                    updated.is_active,
                )
                .await
            {
                Ok(u) => {
                    info!("SCIM user upserted: {}", u.username);
                    let scim = ScimUser::from_user(&u, None);
                    scim_json_response(StatusCode::OK, &scim)
                }
                Err(e) => scim_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &format!("Update failed: {}", e),
                ),
            }
        }
        Err(_) => match repo.create(&user).await {
            Ok(created) => {
                info!("SCIM user created: {}", created.username);
                let scim = ScimUser::from_user(&created, None);
                scim_json_response(StatusCode::CREATED, &scim)
            }
            Err(e) => {
                let err_msg = e.to_string();
                if err_msg.contains("duplicate") || err_msg.contains("unique") {
                    scim_error_response(StatusCode::CONFLICT, &err_msg)
                } else {
                    scim_error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        &format!("Create failed: {}", e),
                    )
                }
            }
        },
    }
}

#[instrument(skip(state, headers))]
pub async fn update_user(
    State(state): State<ScimState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(scim_user): Json<ScimUser>,
) -> Response {
    if let Err(e) = verify_bearer_token(&headers, &state.bearer_token) {
        return e;
    }

    let user_id = match UserId::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return scim_error_response(StatusCode::NOT_FOUND, &format!("User '{}' not found", id));
        }
    };

    let repo = tachyon_database::UserRepository::new(state.pool.clone());
    let existing = match repo.get_by_id(&user_id).await {
        Ok(u) => u,
        Err(_) => {
            return scim_error_response(StatusCode::NOT_FOUND, &format!("User '{}' not found", id));
        }
    };

    let updated = scim_user.to_user_update(&existing);
    match repo
        .update(
            &existing.id,
            Some(&updated.display_name),
            updated.email.as_deref(),
            Some(updated.permissions.role),
            updated.is_active,
        )
        .await
    {
        Ok(u) => {
            info!("SCIM user updated: {}", u.username);
            let scim = ScimUser::from_user(&u, None);
            scim_json_response(StatusCode::OK, &scim)
        }
        Err(e) => scim_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Update failed: {}", e),
        ),
    }
}

#[instrument(skip(state, headers))]
pub async fn patch_user(
    State(state): State<ScimState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(patch_req): Json<ScimPatchRequest>,
) -> Response {
    if let Err(e) = verify_bearer_token(&headers, &state.bearer_token) {
        return e;
    }

    let user_id = match UserId::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return scim_error_response(StatusCode::NOT_FOUND, &format!("User '{}' not found", id));
        }
    };

    let repo = tachyon_database::UserRepository::new(state.pool.clone());
    let existing = match repo.get_by_id(&user_id).await {
        Ok(u) => u,
        Err(_) => {
            return scim_error_response(StatusCode::NOT_FOUND, &format!("User '{}' not found", id));
        }
    };

    let mut user = existing.clone();
    for operation in &patch_req.operations {
        match operation.op.as_str() {
            "replace" => {
                apply_patch_op(&mut user, &operation.path, &operation.value);
            }
            "add" => {
                apply_patch_op(&mut user, &operation.path, &operation.value);
            }
            "remove" => {
                apply_remove_op(&mut user, &operation.path);
            }
            other => {
                return scim_error_response(
                    StatusCode::BAD_REQUEST,
                    &format!("Unsupported patch operation: {}", other),
                );
            }
        }
    }

    match repo
        .update(
            &user.id,
            Some(&user.display_name),
            user.email.as_deref(),
            Some(user.permissions.role),
            user.is_active,
        )
        .await
    {
        Ok(u) => {
            info!("SCIM user patched: {}", u.username);
            let scim = ScimUser::from_user(&u, None);
            scim_json_response(StatusCode::OK, &scim)
        }
        Err(e) => scim_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Patch failed: {}", e),
        ),
    }
}

#[instrument(skip(state, headers))]
pub async fn delete_user(
    State(state): State<ScimState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response {
    if let Err(e) = verify_bearer_token(&headers, &state.bearer_token) {
        return e;
    }

    let user_id = match UserId::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return scim_error_response(StatusCode::NOT_FOUND, &format!("User '{}' not found", id));
        }
    };

    let repo = tachyon_database::UserRepository::new(state.pool.clone());
    match repo.deactivate(&user_id).await {
        Ok(()) => {
            info!("SCIM user deactivated: {}", id);
            StatusCode::NO_CONTENT.into_response()
        }
        Err(_) => scim_error_response(StatusCode::NOT_FOUND, &format!("User '{}' not found", id)),
    }
}

// ============================================================================
// SCIM Group Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ScimGroupListParams {
    pub filter: Option<String>,
    #[serde(rename = "startIndex", default)]
    pub start_index: Option<usize>,
    #[serde(rename = "count", default)]
    pub count: Option<usize>,
}

#[instrument(skip(state, headers))]
pub async fn list_groups(
    State(state): State<ScimState>,
    Query(params): Query<ScimGroupListParams>,
    headers: HeaderMap,
) -> Response {
    if let Err(e) = verify_bearer_token(&headers, &state.bearer_token) {
        return e;
    }

    let team_repo = TeamRepository::new(state.pool.clone());
    let teams = match team_repo.list_all().await {
        Ok(t) => t,
        Err(e) => {
            return scim_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Database error: {}", e),
            );
        }
    };

    let mut resources: Vec<ScimGroup> = teams
        .iter()
        .map(|t| ScimGroup::from_team(t, None))
        .collect();

    if let Some(ref filter_str) = params.filter {
        if let Some(rest) = filter_str.trim().strip_prefix("displayName eq ") {
            let value = rest.trim().trim_matches('"').to_string();
            resources.retain(|g| g.display_name.eq_ignore_ascii_case(&value));
        } else if let Some(rest) = filter_str.trim().strip_prefix("displayName co ") {
            let value = rest.trim().trim_matches('"').to_string();
            resources.retain(|g| {
                g.display_name
                    .to_lowercase()
                    .contains(&value.to_lowercase())
            });
        } else {
            return scim_error_response(
                StatusCode::BAD_REQUEST,
                &format!("Unsupported SCIM filter: {}", filter_str),
            );
        }
    }

    let page_size = params.count.unwrap_or(100).min(100);
    let start = params.start_index.unwrap_or(1).max(1);
    let end = (start - 1 + page_size).min(resources.len());
    let paged = if start <= resources.len() {
        resources[start - 1..end].to_vec()
    } else {
        Vec::new()
    };

    let total = paged.len();
    let list_response = ScimListResponse {
        schemas: vec![SCIM_LIST_RESPONSE_SCHEMA.to_string()],
        total_results: total,
        start_index: params.start_index,
        items_per_page: Some(total),
        resources: paged,
    };

    scim_json_response(StatusCode::OK, &list_response)
}

#[instrument(skip(state, headers))]
pub async fn get_group(
    State(state): State<ScimState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response {
    if let Err(e) = verify_bearer_token(&headers, &state.bearer_token) {
        return e;
    }

    let team_repo = TeamRepository::new(state.pool.clone());
    match team_repo.get_by_id(&id).await {
        Ok(team) => {
            let members = team_repo.list_members(&id).await.unwrap_or_default();
            let scim_group = ScimGroup::from_team_with_members(&team, &members, None);
            scim_json_response(StatusCode::OK, &scim_group)
        }
        Err(_) => scim_error_response(StatusCode::NOT_FOUND, &format!("Group '{}' not found", id)),
    }
}

#[derive(Debug, Deserialize)]
pub struct ScimCreateGroup {
    #[serde(rename = "schemas")]
    pub schemas: Vec<String>,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<ScimMember>>,
}

#[instrument(skip(state, headers))]
pub async fn create_group(
    State(state): State<ScimState>,
    headers: HeaderMap,
    Json(body): Json<ScimCreateGroup>,
) -> Response {
    if let Err(e) = verify_bearer_token(&headers, &state.bearer_token) {
        return e;
    }

    if body.display_name.is_empty() {
        return scim_error_response(StatusCode::BAD_REQUEST, "displayName is required");
    }

    let team_repo = TeamRepository::new(state.pool.clone());
    let slug = body
        .display_name
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();

    let owner_id = "00000000-0000-0000-0000-000000000000".to_string();
    let team = tachyon_database::Team::new(body.display_name.clone(), slug, owner_id);

    match team_repo.create(&team).await {
        Ok(created) => {
            info!("SCIM group created: {}", created.name);
            let scim_group = ScimGroup::from_team(&created, None);
            scim_json_response(StatusCode::CREATED, &scim_group)
        }
        Err(e) => {
            let err_msg = e.to_string();
            if err_msg.contains("duplicate") || err_msg.contains("unique") {
                scim_error_response(StatusCode::CONFLICT, &err_msg)
            } else {
                scim_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &format!("Create failed: {}", e),
                )
            }
        }
    }
}

#[instrument(skip(state, headers))]
pub async fn delete_group(
    State(state): State<ScimState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response {
    if let Err(e) = verify_bearer_token(&headers, &state.bearer_token) {
        return e;
    }

    let team_repo = TeamRepository::new(state.pool.clone());
    match team_repo.delete(&id).await {
        Ok(()) => {
            info!("SCIM group deleted: {}", id);
            StatusCode::NO_CONTENT.into_response()
        }
        Err(_) => scim_error_response(StatusCode::NOT_FOUND, &format!("Group '{}' not found", id)),
    }
}

// ============================================================================
// SCIM Service Provider Config
// ============================================================================

#[instrument(skip(_headers))]
pub async fn service_provider_config(_headers: HeaderMap) -> Response {
    let config = ScimServiceProviderConfig {
        schemas: vec![SCIM_SERVICE_PROVIDER_CONFIG_SCHEMA.to_string()],
        patch: ScimPatchSupport { supported: true },
        bulk: ScimBulkSupport {
            supported: false,
            max_operations: None,
            max_payload_size: None,
        },
        filter: ScimFilterSupport {
            supported: true,
            max_results: Some(100),
        },
        change_password: ScimChangePasswordSupport { supported: false },
        sort: ScimSortSupport { supported: false },
        etag: ScimEtagSupport { supported: false },
        authentication_schemes: vec![ScimAuthScheme {
            scheme_type: "oauthbearertoken".to_string(),
            name: "OAuth Bearer Token".to_string(),
            description: Some("Authentication scheme using Bearer tokens".to_string()),
            spec_uri: Some("https://tools.ietf.org/html/rfc6750".to_string()),
            documentation_uri: None,
            primary: true,
        }],
    };

    scim_json_response(StatusCode::OK, &config)
}

// ============================================================================
// SCIM Schemas
// ============================================================================

#[instrument(skip(_headers))]
pub async fn schemas_endpoint(_headers: HeaderMap) -> Response {
    let user_schema = ScimSchema {
        schemas: vec![SCIM_SCHEMA_SCHEMA.to_string()],
        id: SCIM_USER_SCHEMA.to_string(),
        name: "User".to_string(),
        description: "Tachyon User".to_string(),
        attributes: vec![
            ScimSchemaAttribute {
                name: "userName".to_string(),
                attr_type: "string".to_string(),
                multi_valued: false,
                description: Some("Unique identifier for the user".to_string()),
                required: Some(true),
                case_exact: Some(true),
                mutability: Some("readWrite".to_string()),
                returned: Some("always".to_string()),
                uniqueness: Some("server".to_string()),
            },
            ScimSchemaAttribute {
                name: "displayName".to_string(),
                attr_type: "string".to_string(),
                multi_valued: false,
                description: Some("The name of the user".to_string()),
                required: Some(false),
                case_exact: Some(false),
                mutability: Some("readWrite".to_string()),
                returned: Some("always".to_string()),
                uniqueness: Some("none".to_string()),
            },
            ScimSchemaAttribute {
                name: "emails".to_string(),
                attr_type: "complex".to_string(),
                multi_valued: true,
                description: Some("Email addresses for the user".to_string()),
                required: Some(false),
                case_exact: Some(false),
                mutability: Some("readWrite".to_string()),
                returned: Some("always".to_string()),
                uniqueness: Some("none".to_string()),
            },
            ScimSchemaAttribute {
                name: "active".to_string(),
                attr_type: "boolean".to_string(),
                multi_valued: false,
                description: Some("Whether the user is active".to_string()),
                required: Some(false),
                case_exact: None,
                mutability: Some("readWrite".to_string()),
                returned: Some("always".to_string()),
                uniqueness: Some("none".to_string()),
            },
            ScimSchemaAttribute {
                name: "roles".to_string(),
                attr_type: "complex".to_string(),
                multi_valued: true,
                description: Some("User roles".to_string()),
                required: Some(false),
                case_exact: Some(false),
                mutability: Some("readWrite".to_string()),
                returned: Some("always".to_string()),
                uniqueness: Some("none".to_string()),
            },
        ],
    };

    let group_schema = ScimSchema {
        schemas: vec![SCIM_SCHEMA_SCHEMA.to_string()],
        id: SCIM_GROUP_SCHEMA.to_string(),
        name: "Group".to_string(),
        description: "Tachyon Group (Team)".to_string(),
        attributes: vec![
            ScimSchemaAttribute {
                name: "displayName".to_string(),
                attr_type: "string".to_string(),
                multi_valued: false,
                description: Some("A human-readable name for the group".to_string()),
                required: Some(true),
                case_exact: Some(false),
                mutability: Some("readWrite".to_string()),
                returned: Some("always".to_string()),
                uniqueness: Some("none".to_string()),
            },
            ScimSchemaAttribute {
                name: "members".to_string(),
                attr_type: "complex".to_string(),
                multi_valued: true,
                description: Some("Members of the group".to_string()),
                required: Some(false),
                case_exact: Some(false),
                mutability: Some("readWrite".to_string()),
                returned: Some("default".to_string()),
                uniqueness: Some("none".to_string()),
            },
        ],
    };

    let list_response = ScimListResponse {
        schemas: vec![SCIM_LIST_RESPONSE_SCHEMA.to_string()],
        total_results: 2,
        start_index: Some(1),
        items_per_page: Some(2),
        resources: vec![user_schema, group_schema],
    };

    scim_json_response(StatusCode::OK, &list_response)
}

// ============================================================================
// Patch Helpers
// ============================================================================

fn apply_patch_op(user: &mut User, path: &str, value: &Option<Value>) {
    let value = match value {
        Some(v) => v,
        None => return,
    };

    match path {
        "active" => {
            if let Some(active) = value.as_bool() {
                user.is_active = Some(active);
            }
        }
        "name.givenName" | "name.given" => {
            if let Some(s) = value.as_str() {
                let family = user
                    .display_name
                    .split_once(' ')
                    .map(|(_, f)| f.to_string())
                    .unwrap_or_default();
                user.display_name = if family.is_empty() {
                    s.to_string()
                } else {
                    format!("{} {}", s, family)
                };
            }
        }
        "name.familyName" | "name.family" => {
            if let Some(s) = value.as_str() {
                let given = user
                    .display_name
                    .split_once(' ')
                    .map(|(g, _)| g.to_string())
                    .unwrap_or_default();
                user.display_name = if given.is_empty() {
                    s.to_string()
                } else {
                    format!("{} {}", given, s)
                };
            }
        }
        "displayName" => {
            if let Some(s) = value.as_str() {
                user.display_name = s.to_string();
            }
        }
        "emails[value]" | "emails" => {
            if let Some(arr) = value.as_array() {
                if let Some(first) = arr.first() {
                    if let Some(email_val) = first.get("value").and_then(|v| v.as_str()) {
                        user.email = Some(email_val.to_string());
                    } else if let Some(email_str) = first.as_str() {
                        user.email = Some(email_str.to_string());
                    }
                }
            }
        }
        "emails.value" => {
            if let Some(s) = value.as_str() {
                user.email = Some(s.to_string());
            }
        }
        "userName" => {
            if let Some(s) = value.as_str() {
                user.username = s.to_string();
            }
        }
        "roles" => {
            if let Some(arr) = value.as_array() {
                if let Some(first) = arr.first() {
                    let role_str = first
                        .get("value")
                        .and_then(|v| v.as_str())
                        .or_else(|| first.as_str())
                        .unwrap_or("reader");
                    user.permissions.role = parse_role(role_str);
                }
            }
        }
        _ => {}
    }
}

fn apply_remove_op(user: &mut User, path: &str) {
    match path {
        "emails" => {
            user.email = None;
        }
        "active" => {
            user.is_active = Some(false);
        }
        "roles" => {
            user.permissions.role = UserRole::Reader;
        }
        _ => {}
    }
}

// ============================================================================
// Router
// ============================================================================

pub fn create_scim_router() -> axum::Router<ScimState> {
    axum::Router::new()
        .route("/Users", axum::routing::get(list_users))
        .route("/Users", axum::routing::post(create_user))
        .route("/Users/{id}", axum::routing::get(get_user))
        .route("/Users/{id}", axum::routing::put(update_user))
        .route("/Users/{id}", axum::routing::patch(patch_user))
        .route("/Users/{id}", axum::routing::delete(delete_user))
        .route("/Groups", axum::routing::get(list_groups))
        .route("/Groups", axum::routing::post(create_group))
        .route("/Groups/{id}", axum::routing::get(get_group))
        .route("/Groups/{id}", axum::routing::delete(delete_group))
        .route(
            "/ServiceProviderConfig",
            axum::routing::get(service_provider_config),
        )
        .route("/Schemas", axum::routing::get(schemas_endpoint))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tachyon_core::types::user::UserType;

    fn make_test_user() -> User {
        let id = UserId::new();
        let mut user = User::new(
            id,
            "john.doe".to_string(),
            "John Doe".to_string(),
            UserRole::Editor,
        );
        user.email = Some("john@example.com".to_string());
        user.is_active = Some(true);
        user.user_type = UserType::Regular;
        user.created_at = Utc::now();
        user.updated_at = Utc::now();
        user
    }

    #[test]
    fn test_scim_user_serialization() {
        let user = make_test_user();
        let scim = ScimUser::from_user(&user, Some("https://tachyon.example.com"));

        let json = serde_json::to_string(&scim).unwrap();
        assert!(json.contains("urn:ietf:params:scim:schemas:core:2.0:User"));
        assert!(json.contains("\"userName\""));
        assert!(json.contains("john.doe"));
        assert!(json.contains("John Doe"));
        assert!(json.contains("emails"));
        assert!(json.contains("john@example.com"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_scim_user_schema_field() {
        let user = make_test_user();
        let scim = ScimUser::from_user(&user, None);
        assert_eq!(scim.schemas, vec![SCIM_USER_SCHEMA]);
    }

    #[test]
    fn test_scim_user_deserialization() {
        let json = r#"{
            "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
            "id": "test-user-id",
            "userName": "jane.smith",
            "displayName": "Jane Smith",
            "name": {
                "givenName": "Jane",
                "familyName": "Smith"
            },
            "emails": [{"value": "jane@example.com", "primary": true}],
            "active": true,
            "roles": [{"value": "admin"}],
            "groups": []
        }"#;

        let scim: ScimUser = serde_json::from_str(json).unwrap();
        assert_eq!(scim.user_name, "jane.smith");
        assert_eq!(scim.display_name, Some("Jane Smith".to_string()));
        assert_eq!(scim.emails.len(), 1);
        assert_eq!(scim.emails[0].value, "jane@example.com");
        assert_eq!(scim.active, Some(true));
        assert_eq!(scim.roles.as_ref().unwrap()[0].value, "admin");
    }

    #[test]
    fn test_scim_list_response_format() {
        let user = make_test_user();
        let scim = ScimUser::from_user(&user, None);
        let list = ScimListResponse {
            schemas: vec![SCIM_LIST_RESPONSE_SCHEMA.to_string()],
            total_results: 1,
            start_index: Some(1),
            items_per_page: Some(100),
            resources: vec![scim],
        };

        let json = serde_json::to_string(&list).unwrap();
        assert!(
            json.contains("\"schemas\":[\"urn:ietf:params:scim:api:messages:2.0:ListResponse\"]")
        );
        assert!(json.contains("\"totalResults\":1"));
        assert!(json.contains("\"startIndex\":1"));
        assert!(json.contains("\"itemsPerPage\":100"));
        assert!(json.contains("\"Resources\""));
    }

    #[test]
    fn test_scim_error_format() {
        let err = ScimError {
            schemas: vec![SCIM_ERROR_SCHEMA.to_string()],
            detail: "User not found".to_string(),
            status: "404".to_string(),
        };

        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"schemas\":[\"urn:ietf:params:scim:api:messages:2.0:Error\"]"));
        assert!(json.contains("\"detail\":\"User not found\""));
        assert!(json.contains("\"status\":\"404\""));
    }

    #[test]
    fn test_scim_user_to_internal_user_conversion() {
        let scim = ScimUser {
            schemas: vec![SCIM_USER_SCHEMA.to_string()],
            id: String::new(),
            external_id: None,
            user_name: "test.user".to_string(),
            name: Some(ScimName {
                given_name: Some("Test".to_string()),
                family_name: Some("User".to_string()),
            }),
            display_name: Some("Test User".to_string()),
            emails: vec![ScimEmail {
                value: "test@example.com".to_string(),
                primary: Some(true),
            }],
            phone_numbers: None,
            roles: Some(vec![ScimRole {
                value: "writer".to_string(),
            }]),
            active: Some(true),
            groups: Vec::new(),
            meta: None,
        };

        let user = scim.to_user_create().unwrap();
        assert_eq!(user.username, "test.user");
        assert_eq!(user.display_name, "Test User");
        assert_eq!(user.email, Some("test@example.com".to_string()));
        assert_eq!(user.permissions.role, UserRole::Writer);
        assert_eq!(user.is_active, Some(true));
    }

    #[test]
    fn test_scim_user_to_internal_user_update() {
        let existing = make_test_user();
        let scim = ScimUser {
            schemas: vec![SCIM_USER_SCHEMA.to_string()],
            id: existing.id.as_str(),
            external_id: None,
            user_name: existing.username.clone(),
            name: Some(ScimName {
                given_name: Some("Updated".to_string()),
                family_name: Some("Name".to_string()),
            }),
            display_name: Some("Updated Name".to_string()),
            emails: vec![ScimEmail {
                value: "updated@example.com".to_string(),
                primary: Some(true),
            }],
            phone_numbers: None,
            roles: Some(vec![ScimRole {
                value: "admin".to_string(),
            }]),
            active: Some(false),
            groups: Vec::new(),
            meta: None,
        };

        let updated = scim.to_user_update(&existing);
        assert_eq!(updated.display_name, "Updated Name");
        assert_eq!(updated.email, Some("updated@example.com".to_string()));
        assert_eq!(updated.permissions.role, UserRole::Admin);
        assert_eq!(updated.is_active, Some(false));
    }

    #[test]
    fn test_parse_role() {
        assert!(matches!(parse_role("admin"), UserRole::Admin));
        assert!(matches!(parse_role("Admin"), UserRole::Admin));
        assert!(matches!(parse_role("editor"), UserRole::Editor));
        assert!(matches!(parse_role("writer"), UserRole::Writer));
        assert!(matches!(parse_role("reader"), UserRole::Reader));
        assert!(matches!(parse_role("unknown"), UserRole::Reader));
    }

    #[test]
    fn test_filter_parse_username() {
        let filter = parse_filter(r#"userName eq "john.doe""#).unwrap();
        let user = make_test_user();
        assert!(filter.matches(&user));
    }

    #[test]
    fn test_filter_parse_active() {
        let filter = parse_filter("active eq true").unwrap();
        let user = make_test_user();
        assert!(filter.matches(&user));

        let filter = parse_filter("active eq false").unwrap();
        assert!(!filter.matches(&user));
    }

    #[test]
    fn test_filter_parse_email() {
        let filter = parse_filter(r#"emails.value eq "john@example.com""#).unwrap();
        let user = make_test_user();
        assert!(filter.matches(&user));
    }

    #[test]
    fn test_filter_parse_unsupported() {
        let result = parse_filter("displayName eq 'test'");
        assert!(result.is_err());
    }

    #[test]
    fn test_patch_request_deserialization() {
        let json = r#"{
            "schemas": ["urn:ietf:params:scim:api:messages:2.0:PatchOp"],
            "Operations": [
                {"op": "replace", "path": "active", "value": false},
                {"op": "replace", "path": "displayName", "value": "New Name"}
            ]
        }"#;

        let patch: ScimPatchRequest = serde_json::from_str(json).unwrap();
        assert_eq!(patch.operations.len(), 2);
        assert_eq!(patch.operations[0].op, "replace");
        assert_eq!(patch.operations[0].path, "active");
    }

    #[test]
    fn test_scim_user_create_requires_username() {
        let scim = ScimUser {
            schemas: vec![SCIM_USER_SCHEMA.to_string()],
            id: String::new(),
            external_id: None,
            user_name: String::new(),
            name: None,
            display_name: None,
            emails: Vec::new(),
            phone_numbers: None,
            roles: None,
            active: None,
            groups: Vec::new(),
            meta: None,
        };
        assert!(scim.to_user_create().is_err());
    }

    #[test]
    fn test_scim_user_display_name_from_name() {
        let scim = ScimUser {
            schemas: vec![SCIM_USER_SCHEMA.to_string()],
            id: String::new(),
            external_id: None,
            user_name: "jdoe".to_string(),
            name: Some(ScimName {
                given_name: Some("John".to_string()),
                family_name: Some("Doe".to_string()),
            }),
            display_name: None,
            emails: Vec::new(),
            phone_numbers: None,
            roles: None,
            active: None,
            groups: Vec::new(),
            meta: None,
        };

        let user = scim.to_user_create().unwrap();
        assert_eq!(user.display_name, "John Doe");
    }

    #[test]
    fn test_apply_patch_active() {
        let mut user = make_test_user();
        let value = Some(Value::Bool(false));
        apply_patch_op(&mut user, "active", &value);
        assert_eq!(user.is_active, Some(false));
    }

    #[test]
    fn test_apply_patch_display_name() {
        let mut user = make_test_user();
        let value = Some(Value::String("New Display Name".to_string()));
        apply_patch_op(&mut user, "displayName", &value);
        assert_eq!(user.display_name, "New Display Name");
    }

    #[test]
    fn test_apply_patch_email() {
        let mut user = make_test_user();
        let value = Some(Value::String("newemail@example.com".to_string()));
        apply_patch_op(&mut user, "emails.value", &value);
        assert_eq!(user.email, Some("newemail@example.com".to_string()));
    }

    #[test]
    fn test_apply_patch_role() {
        let mut user = make_test_user();
        let value = Some(Value::Array(vec![Value::Object(
            [("value".to_string(), Value::String("admin".to_string()))]
                .into_iter()
                .collect(),
        )]));
        apply_patch_op(&mut user, "roles", &value);
        assert_eq!(user.permissions.role, UserRole::Admin);
    }

    #[test]
    fn test_apply_remove_emails() {
        let mut user = make_test_user();
        apply_remove_op(&mut user, "emails");
        assert!(user.email.is_none());
    }

    #[test]
    fn test_bearer_token_verification_valid() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer valid-token".parse().unwrap(),
        );
        assert!(verify_bearer_token(&headers, "valid-token").is_ok());
    }

    #[test]
    fn test_bearer_token_verification_missing() {
        let headers = HeaderMap::new();
        assert!(verify_bearer_token(&headers, "valid-token").is_err());
    }

    #[test]
    fn test_bearer_token_verification_invalid() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer wrong-token".parse().unwrap(),
        );
        assert!(verify_bearer_token(&headers, "valid-token").is_err());
    }

    #[test]
    fn test_random_password_generation() {
        let pw = generate_random_password();
        assert_eq!(pw.len(), 32);
        assert!(pw.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_filter_parse_display_name_co() {
        let filter = parse_filter(r#"displayName co "ohn""#).unwrap();
        let user = make_test_user();
        assert!(filter.matches(&user));

        let filter = parse_filter(r#"displayName co "xyz""#).unwrap();
        assert!(!filter.matches(&user));
    }

    #[test]
    fn test_filter_parse_display_name_co_case_insensitive() {
        let filter = parse_filter(r#"displayName co "JOHN""#).unwrap();
        let user = make_test_user();
        assert!(filter.matches(&user));
    }

    #[test]
    fn test_scim_group_serialization() {
        let team = tachyon_database::Team {
            id: "test-team-id".to_string(),
            name: "Engineering".to_string(),
            slug: "engineering".to_string(),
            description: Some("Engineering team".to_string()),
            owner_id: "owner-id".to_string(),
            avatar_url: None,
            settings: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let scim_group = ScimGroup::from_team(&team, Some("https://tachyon.example.com"));
        let json = serde_json::to_string(&scim_group).unwrap();
        assert!(json.contains("urn:ietf:params:scim:schemas:core:2.0:Group"));
        assert!(json.contains("Engineering"));
        assert!(json.contains("test-team-id"));
    }

    #[test]
    fn test_scim_group_with_members() {
        let team = tachyon_database::Team {
            id: "team-1".to_string(),
            name: "Dev".to_string(),
            slug: "dev".to_string(),
            description: None,
            owner_id: "owner".to_string(),
            avatar_url: None,
            settings: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let members = vec![
            tachyon_database::TeamMember {
                id: 1,
                team_id: "team-1".to_string(),
                user_id: "user-1".to_string(),
                role_id: 1,
                role_name: "member".to_string(),
                joined_at: Utc::now(),
                invited_by: None,
            },
            tachyon_database::TeamMember {
                id: 2,
                team_id: "team-1".to_string(),
                user_id: "user-2".to_string(),
                role_id: 1,
                role_name: "member".to_string(),
                joined_at: Utc::now(),
                invited_by: None,
            },
        ];

        let scim_group = ScimGroup::from_team_with_members(&team, &members, None);
        let json = serde_json::to_string(&scim_group).unwrap();
        assert!(json.contains("\"members\""));
        assert!(json.contains("\"user-1\""));
        assert!(json.contains("\"user-2\""));
    }

    #[test]
    fn test_scim_group_from_team_without_base_url() {
        let team = tachyon_database::Team {
            id: "team-2".to_string(),
            name: "Product".to_string(),
            slug: "product".to_string(),
            description: None,
            owner_id: "owner".to_string(),
            avatar_url: None,
            settings: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let scim_group = ScimGroup::from_team(&team, None);
        assert!(scim_group.meta.as_ref().unwrap().location.is_none());
    }

    #[test]
    fn test_scim_group_from_team_with_base_url() {
        let team = tachyon_database::Team {
            id: "team-3".to_string(),
            name: "Sales".to_string(),
            slug: "sales".to_string(),
            description: None,
            owner_id: "owner".to_string(),
            avatar_url: None,
            settings: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let scim_group = ScimGroup::from_team(&team, Some("https://tachyon.example.com"));
        assert_eq!(
            scim_group.meta.as_ref().unwrap().location,
            Some("https://tachyon.example.com/api/v1/scim/v2/Groups/team-3".to_string())
        );
    }
}
