// Database Types and Structures
// Core data structures for database operations

use crate::error::DatabaseResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;

// ============================================================================
// Session Models
// ============================================================================

/// Database model for session persistence (PostgreSQL)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct SessionRecord {
    /// Session ID
    pub id: String,
    /// User ID
    pub user_id: String,
    /// Session type (desktop, web, api, mobile)
    pub session_type: String,
    /// Session status (active, expired, revoked)
    pub status: String,
    /// Token value
    pub token_value: String,
    /// Token type (jwt, bearer, api_key)
    pub token_type: String,
    /// IP address
    pub ip_address: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Device info
    pub device_info: Option<String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Expires timestamp
    pub expires_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
}

impl SessionRecord {
    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if session is valid
    pub fn is_valid(&self) -> bool {
        self.status == "Active" && !self.is_expired()
    }
}

/// Database model for user-role mapping (PostgreSQL)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserRoleMapping {
    /// Mapping ID
    pub id: i64,
    /// User ID
    pub user_id: String,
    /// Role name
    pub role: String,
    /// Granted by user ID
    pub assigned_by: Option<String>,
    /// Granted at timestamp
    pub assigned_at: DateTime<Utc>,
    /// Expiration timestamp (optional)
    pub expires_at: Option<DateTime<Utc>>,
}

/// Database model for role-permission mapping (PostgreSQL)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RolePermissionMapping {
    /// Mapping ID
    pub id: i64,
    /// Role name
    pub role: String,
    /// Permission name
    pub permission: String,
    /// Resource type
    pub resource_type: Option<String>,
    /// Condition JSON
    pub conditions: Option<String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

impl RolePermissionMapping {
    /// Parse conditions from JSON string
    pub fn parse_conditions(&self) -> DatabaseResult<Option<serde_json::Value>> {
        match &self.conditions {
            Some(c) => Ok(Some(serde_json::from_str(c)?)),
            None => Ok(None),
        }
    }

    /// Serialize conditions to JSON string
    pub fn serialize_conditions(
        conditions: &Option<serde_json::Value>,
    ) -> DatabaseResult<Option<String>> {
        match conditions {
            Some(c) => Ok(Some(serde_json::to_string(c)?)),
            None => Ok(None),
        }
    }
}

/// Database model for RBAC policy (PostgreSQL)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PolicyRecord {
    /// Policy ID
    pub id: i64,
    /// Policy name
    pub name: String,
    /// Policy description
    pub description: Option<String>,
    /// Policy rules (JSON)
    pub rules: String,
    /// Effect (allow/deny)
    pub effect: String,
    /// Policy type
    pub policy_type: Option<String>,
    /// Created by
    pub created_by: Option<String>,
    /// Whether policy is enabled
    pub enabled: bool,
    /// Priority for evaluation
    pub priority: i32,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Database model for permission audit log (PostgreSQL)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PermissionAuditLog {
    /// Log ID
    pub id: i64,
    /// User ID
    pub user_id: String,
    /// Session ID
    pub session_id: Option<String>,
    /// Subject type (user/role/service)
    pub subject_type: Option<String>,
    /// Subject ID
    pub subject_id: Option<String>,
    /// Role name
    pub role: Option<String>,
    /// Permission name
    pub permission: Option<String>,
    /// Resource type
    pub resource_type: Option<String>,
    /// Resource ID
    pub resource_id: Option<String>,
    /// Action (grant/revoke/check)
    pub action: String,
    /// Effect (allow/deny)
    pub effect: String,
    /// Policy ID
    pub policy_id: Option<i64>,
    /// Reason
    pub reason: Option<String>,
    /// IP address
    pub ip_address: Option<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl PolicyRecord {
    /// Parse rules from JSON string
    pub fn parse_rules(&self) -> DatabaseResult<Vec<serde_json::Value>> {
        serde_json::from_str(&self.rules)
            .map_err(|e| crate::error::DatabaseError::SerializationError(e.to_string()))
    }

    /// Serialize rules to JSON string
    pub fn serialize_rules(rules: &[serde_json::Value]) -> DatabaseResult<String> {
        serde_json::to_string(rules)
            .map_err(|e| crate::error::DatabaseError::SerializationError(e.to_string()))
    }
}

// ============================================================================
// Document Metadata Models
// ============================================================================

/// Database model for document metadata
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct DocumentMetadata {
    /// Document ID
    pub id: String,
    /// Document title
    pub title: String,
    /// Document slug
    pub slug: Option<String>,
    /// Author user ID
    pub author_id: String,
    /// Document description
    pub description: Option<String>,
    /// Tags (JSON array)
    pub tags: String,
    /// Frontmatter (JSON object)
    pub frontmatter: Option<String>,
    /// Project ID (was repository_id, renamed to match schema)
    pub project_id: Option<String>,
    /// Document visibility
    pub visibility: String,
    /// Document status
    pub status: String,
    /// Content type
    pub content_type: String,
    /// Word count
    pub word_count: i32,
    /// Character count
    pub character_count: i32,
    /// Read count
    pub read_count: i32,
    /// Edit count
    pub edit_count: i32,
    /// Document content (raw markdown)
    pub content: Option<String>,
    /// Rendered HTML
    pub html: Option<String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Published timestamp
    pub published_at: Option<DateTime<Utc>>,
    /// SHA-256 hash of raw markdown content (for integrity checking)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    /// Whether a sync conflict was detected (content changed in both file and DB)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflict_detected: Option<bool>,
}

impl DocumentMetadata {
    /// Parse tags from JSON string
    ///
    /// # Returns
    /// Vector of tags
    pub fn parse_tags(&self) -> DatabaseResult<Vec<String>> {
        serde_json::from_str(&self.tags)
            .map_err(|e| crate::error::DatabaseError::SerializationError(e.to_string()))
    }

    /// Serialize tags to JSON string
    ///
    /// # Arguments
    /// * `tags` - Vector of tags
    ///
    /// # Returns
    /// JSON string of tags
    pub fn serialize_tags(tags: &[String]) -> DatabaseResult<String> {
        serde_json::to_string(tags)
            .map_err(|e| crate::error::DatabaseError::SerializationError(e.to_string()))
    }

    /// Parse frontmatter from JSON string
    ///
    /// # Returns
    /// HashMap of frontmatter key-value pairs
    pub fn parse_frontmatter(&self) -> DatabaseResult<HashMap<String, serde_json::Value>> {
        Ok(self
            .frontmatter
            .as_ref()
            .map(|fm| serde_json::from_str::<HashMap<String, serde_json::Value>>(fm))
            .transpose()
            .map_err(|e| crate::error::DatabaseError::SerializationError(e.to_string()))?
            .unwrap_or_default())
    }

    /// Serialize frontmatter to JSON string
    ///
    /// # Arguments
    /// * `frontmatter` - HashMap of frontmatter data
    ///
    /// # Returns
    /// JSON string of frontmatter
    pub fn serialize_frontmatter(
        frontmatter: &HashMap<String, serde_json::Value>,
    ) -> DatabaseResult<Option<String>> {
        if frontmatter.is_empty() {
            Ok(None)
        } else {
            serde_json::to_string(frontmatter)
                .map_err(|e| crate::error::DatabaseError::SerializationError(e.to_string()))
                .map(Some)
        }
    }
}

/// Full-text search index entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SearchIndex {
    /// Entry ID
    pub id: i64,
    /// Document ID
    pub document_id: String,
    /// Content type (title, content, tags)
    pub content_type: String,
    /// Text content for searching
    pub content: String,
    /// Weight for ranking
    pub weight: f64,
    /// Indexed timestamp
    pub indexed_at: DateTime<Utc>,
}

// ============================================================================
// Repository Metadata Models
// ============================================================================

/// Database model for repository metadata
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct RepositoryMetadata {
    /// Repository ID
    pub id: String,
    /// Repository name
    pub name: String,
    /// Repository slug
    pub slug: Option<String>,
    /// Repository description
    pub description: Option<String>,
    /// Repository type
    pub repository_type: String,
    /// Owner user ID
    pub owner_id: String,
    /// Repository visibility
    pub visibility: String,
    /// Repository status
    pub status: String,
    /// Default branch
    pub default_branch: Option<String>,
    /// Auto sync enabled
    pub auto_sync: bool,
    /// Sync interval in seconds
    pub sync_interval_seconds: i64,
    /// File watching enabled
    pub file_watching_enabled: bool,
    /// Remote URL
    pub remote_url: Option<String>,
    /// Last commit hash
    pub last_commit_hash: Option<String>,
    /// Current branch
    pub current_branch: Option<String>,
    /// Commits ahead
    pub commits_ahead: Option<i64>,
    /// Commits behind
    pub commits_behind: Option<i64>,
    /// Document count
    pub document_count: i64,
    /// Total storage bytes
    pub total_storage_bytes: i64,
    /// Member count
    pub member_count: i64,
    /// Local path
    pub local_path: Option<String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Project/Service Catalog Models (Backstage-like)
// ============================================================================

/// Database model for projects/services
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct Project {
    /// Project ID (UUID)
    pub id: String,
    /// Project name
    pub name: String,
    /// Project slug (URL-friendly)
    pub slug: String,
    /// Project description
    pub description: Option<String>,
    /// Project type (service, library, website, documentation, etc.)
    pub project_type: String,
    /// Owner/team ID
    pub owner_id: String,
    /// Organization ID (optional)
    pub organization_id: Option<String>,
    /// Lifecycle stage (experimental, production, deprecated)
    pub lifecycle: String,
    /// Repository URL
    pub repository_url: Option<String>,
    /// Documentation URL
    pub docs_url: Option<String>,
    /// API endpoint URL
    pub api_url: Option<String>,
    /// Tags (JSON array)
    pub tags: Vec<String>,
    /// Metadata (JSON)
    pub metadata: serde_json::Value,
    /// Language (e.g., rust, typescript, python)
    pub language: Option<String>,
    /// Framework
    pub framework: Option<String>,
    /// Visibility (public, internal, private)
    pub visibility: String,
    /// Status (active, archived)
    pub status: String,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Database model for project members
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct ProjectMember {
    /// Member ID
    pub id: i64,
    /// Project ID
    pub project_id: String,
    /// User ID
    pub user_id: String,
    /// Role (owner, admin, maintainer, viewer)
    pub role: String,
    /// Added by
    pub added_by: Option<String>,
    /// Added timestamp
    pub added_at: DateTime<Utc>,
}

/// Database model for component catalog (Backstage component)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct Component {
    /// Component ID
    pub id: String,
    /// Component name
    pub name: String,
    /// Component type (service, website, library, etc.)
    pub component_type: String,
    /// Project ID
    pub project_id: String,
    /// Owner team ID
    pub owner_id: String,
    /// System/domain ID
    pub system_id: Option<String>,
    /// Repository URL
    pub repository_url: Option<String>,
    /// Documentation URL
    pub docs_url: Option<String>,
    /// API specification URL
    pub api_spec_url: Option<String>,
    /// Tags
    pub tags: Vec<String>,
    /// Lifecycle
    pub lifecycle: String,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Database model for API specifications
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiSpec {
    /// Spec ID
    pub id: String,
    /// Component ID
    pub component_id: String,
    /// API type (openapi, graphql, grpc, asyncapi)
    pub api_type: String,
    /// Specification content (JSON/YAML)
    pub spec_content: String,
    /// Specification version
    pub version: String,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Knowledge Graph Models (Neo4j/AGE-like)
// ============================================================================

/// Database model for knowledge graph nodes
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct GraphNode {
    /// Node ID
    pub id: String,
    /// Node type (concept, document, reference, media)
    pub node_type: String,
    /// Display name
    pub name: String,
    /// URL-friendly slug
    pub slug: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Full content (for document/concept nodes)
    pub content: Option<String>,
    /// Visibility (public, private, restricted)
    pub visibility: String,
    /// Node weight for ranking
    pub weight: f64,
    /// Properties (JSON)
    pub properties: serde_json::Value,
    /// Project ID (optional, for scoping)
    pub project_id: Option<String>,
    /// Associated document ID
    pub document_id: Option<String>,
    /// Created by user ID
    pub created_by: Option<String>,
    /// Soft delete flag
    pub is_active: bool,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
    /// When the node was deactivated (None if currently active)
    pub deactivated_at: Option<DateTime<Utc>>,
}

/// Database model for knowledge graph edges
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct GraphEdge {
    /// Edge ID
    pub id: String,
    /// Source node ID
    pub source_id: String,
    /// Target node ID
    pub target_id: String,
    /// Edge type (references, depends_on, similar_to, part_of, related_to, tagged_with)
    pub edge_type: String,
    /// Display label
    pub label: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Edge weight for scoring
    pub weight: f64,
    /// Confidence score (0.0 to 1.0)
    pub confidence: Option<f64>,
    /// Properties (JSON)
    pub properties: serde_json::Value,
    /// Project ID (optional, for scoping)
    pub project_id: Option<String>,
    /// Created by user ID
    pub created_by: Option<String>,
    /// Soft delete flag
    pub is_active: bool,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
    /// When the edge was deactivated (None if currently active)
    pub deactivated_at: Option<DateTime<Utc>>,
}

// ============================================================================
// CI/CD Pipeline Models
// ============================================================================

/// Database model for CI/CD pipelines
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Pipeline {
    /// Pipeline ID
    pub id: String,
    /// Pipeline name
    pub name: String,
    /// Project ID
    pub project_id: String,
    /// Pipeline type (ci, cd, or ci/cd)
    pub pipeline_type: String,
    /// Repository URL
    pub repository_url: String,
    /// Default branch
    pub default_branch: String,
    /// Configuration (YAML/JSON)
    pub config: String,
    /// Status (active, paused, disabled)
    pub status: String,
    /// Last run timestamp
    pub last_run_at: Option<DateTime<Utc>>,
    /// Created by user ID
    pub created_by: String,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Database model for pipeline runs
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PipelineRun {
    /// Run ID
    pub id: String,
    /// Pipeline ID
    pub pipeline_id: String,
    /// Run number
    pub run_number: i64,
    /// Commit hash
    pub commit_hash: String,
    /// Commit message
    pub commit_message: String,
    /// Branch name
    pub branch: String,
    /// Trigger type (push, pull_request, manual, schedule)
    pub trigger_type: String,
    /// Status (pending, running, success, failed, cancelled)
    pub status: String,
    /// Started timestamp
    pub started_at: Option<DateTime<Utc>>,
    /// Finished timestamp
    pub finished_at: Option<DateTime<Utc>>,
    /// Duration in seconds
    pub duration_seconds: Option<i64>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

/// Database model for pipeline stages
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PipelineStage {
    /// Stage ID
    pub id: String,
    /// Pipeline run ID
    pub pipeline_run_id: String,
    /// Stage name
    pub name: String,
    /// Stage order
    pub stage_order: i32,
    /// Status
    pub status: String,
    /// Started timestamp
    pub started_at: Option<DateTime<Utc>>,
    /// Finished timestamp
    pub finished_at: Option<DateTime<Utc>>,
    /// Logs
    pub logs: Option<String>,
}

/// Database model for deployment environments
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Environment {
    /// Environment ID
    pub id: String,
    /// Environment name
    pub name: String,
    /// Project ID
    pub project_id: String,
    /// Environment type (development, staging, production)
    pub env_type: String,
    /// Deployment URL
    pub url: Option<String>,
    /// Kubernetes namespace or cloud region
    pub namespace: Option<String>,
    /// Configuration overrides (JSON)
    pub config: serde_json::Value,
    /// Auto-deploy enabled
    pub auto_deploy: bool,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Publishing/Deployment Models (SSG/SSR)
// ============================================================================

/// Database model for published sites
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PublishedSite {
    /// Site ID
    pub id: String,
    /// Site name
    pub name: String,
    /// Project ID
    pub project_id: String,
    /// Custom domain
    pub domain: Option<String>,
    /// Site URL
    pub url: Option<String>,
    /// Build output path
    pub build_output: String,
    /// Rendering mode (ssg, ssr, isr)
    pub render_mode: String,
    /// Branch to deploy
    pub deploy_branch: String,
    /// Custom headers (JSON)
    pub headers: Option<serde_json::Value>,
    /// Environment variables (JSON, encrypted)
    pub env_vars: Option<String>,
    /// Status (building, deployed, failed, deleted)
    pub status: String,
    /// Last deployed commit
    pub last_commit: Option<String>,
    /// Last deployed timestamp
    pub last_deployed_at: Option<DateTime<Utc>>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Database Configuration
// ============================================================================

/// Database configuration options
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Maximum connection pool size
    pub max_connections: u32,
    /// Minimum connection pool size
    pub min_connections: u32,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Enable PostgreSQL extensions
    pub enable_extensions: bool,
    /// Enable query logging
    pub enable_query_logging: bool,
    /// Schema name (for multi-tenancy)
    pub schema: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            max_connections: std::env::var("TACHYON_DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            min_connections: std::env::var("TACHYON_DB_MIN_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            connection_timeout: std::env::var("TACHYON_DB_CONNECTION_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            enable_extensions: true,
            enable_query_logging: std::env::var("TACHYON_DB_QUERY_LOGGING")
                .ok()
                .map(|s| s == "true" || s == "1")
                .unwrap_or(false),
            schema: std::env::var("TACHYON_DB_SCHEMA")
                .ok()
                .unwrap_or_else(|| "public".to_string()),
        }
    }
}

impl DatabaseConfig {
    /// Create new database config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum connections
    pub fn with_max_connections(mut self, max_connections: u32) -> Self {
        self.max_connections = max_connections;
        self
    }

    /// Set minimum connections
    pub fn with_min_connections(mut self, min_connections: u32) -> Self {
        self.min_connections = min_connections;
        self
    }

    /// Set connection timeout
    pub fn with_connection_timeout(mut self, timeout_seconds: u64) -> Self {
        self.connection_timeout = timeout_seconds;
        self
    }

    /// Enable/disable extensions
    pub fn with_extensions(mut self, enable: bool) -> Self {
        self.enable_extensions = enable;
        self
    }

    /// Enable/disable query logging
    pub fn with_query_logging(mut self, enable: bool) -> Self {
        self.enable_query_logging = enable;
        self
    }

    /// Set schema
    pub fn with_schema(mut self, schema: &str) -> Self {
        self.schema = schema.to_string();
        self
    }
}
