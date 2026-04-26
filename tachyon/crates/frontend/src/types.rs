// Shared Types
// Common types shared between frontend components and API
//
// Note: Many types here are defined for API completeness and will be used
// as more features are implemented. Dead code warnings are allowed.

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Re-export types from database crate that API uses
// Note: These are duplicated here for frontend independence

/// Unique identifier type
pub type Id = String;

/// Timestamp type
pub type Timestamp = DateTime<Utc>;

// ============================================================================
// API Response Types (mirroring backend)
// ============================================================================

/// Generic API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl<T: Clone> ApiResponse<T> {
    pub fn success_item(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            error: None,
        }
    }

    pub fn error_msg(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message: None,
            error: Some(message),
        }
    }
}

/// Paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

// ============================================================================
// Authentication Types
// ============================================================================

/// Login request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// User info in auth response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub email: Option<String>,
    pub user_type: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Authentication response from backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticateResponse {
    pub success: bool,
    pub user_id: Option<String>,
    pub access_token: Option<String>,
    pub token_type: String,
    pub expires_in: u64,
    pub error: Option<String>,
    pub user: Option<AuthUser>,
}

/// Auth status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatusResponse {
    pub authenticated: bool,
    pub user: Option<AuthStatusUser>,
    pub message: Option<String>,
}

/// User info in auth status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatusUser {
    pub id: String,
    pub role: String,
}

/// Guest status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestStatusResponse {
    pub guest_login_enabled: bool,
    pub public_notes_enabled: bool,
}

// ============================================================================
// Lifecycle & Status Enums
// ============================================================================

/// Project lifecycle stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum Lifecycle {
    #[serde(rename = "experimental")]
    #[default]
    Experimental,
    #[serde(rename = "development")]
    Development,
    #[serde(rename = "production")]
    Production,
    #[serde(rename = "deprecated")]
    Deprecated,
}


impl std::fmt::Display for Lifecycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lifecycle::Experimental => write!(f, "Experimental"),
            Lifecycle::Development => write!(f, "Development"),
            Lifecycle::Production => write!(f, "Production"),
            Lifecycle::Deprecated => write!(f, "Deprecated"),
        }
    }
}

/// Project visibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum Visibility {
    #[serde(rename = "private")]
    #[default]
    Private,
    #[serde(rename = "internal")]
    Internal,
    #[serde(rename = "public")]
    Public,
}


/// Project status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum ProjectStatus {
    #[serde(rename = "active")]
    #[default]
    Active,
    #[serde(rename = "archived")]
    Archived,
    #[serde(rename = "suspended")]
    Suspended,
}


// ============================================================================
// Catalog Types
// ============================================================================

/// Project entity (Backstage-like service catalog)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Id,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub project_type: String,
    pub owner_id: Id,
    pub organization_id: Option<Id>,
    pub lifecycle: Lifecycle,
    pub repository_url: Option<String>,
    pub docs_url: Option<String>,
    pub api_url: Option<String>,
    pub tags: Vec<String>,
    pub metadata: serde_json::Value,
    pub language: Option<String>,
    pub framework: Option<String>,
    pub visibility: Visibility,
    pub status: ProjectStatus,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Component entity (Backstage-like component)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: Id,
    pub name: String,
    pub component_type: String,
    pub project_id: Option<Id>,
    pub owner_id: Id,
    pub system_id: Option<Id>,
    pub repository_url: Option<String>,
    pub docs_url: Option<String>,
    pub api_spec_url: Option<String>,
    pub tags: Vec<String>,
    pub lifecycle: Lifecycle,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Project member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMember {
    pub project_id: Id,
    pub user_id: Id,
    pub role: String,
    pub added_by: Option<Id>,
    pub added_at: Timestamp,
}

/// Catalog statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogStats {
    pub project_count: i64,
    pub component_count: i64,
    pub member_count: i64,
}

// ============================================================================
// Request Types
// ============================================================================

/// Create project request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub project_type: String,
    pub owner_id: Id,
    pub organization_id: Option<Id>,
    pub lifecycle: Option<Lifecycle>,
    pub repository_url: Option<String>,
    pub docs_url: Option<String>,
    pub api_url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
    pub language: Option<String>,
    pub framework: Option<String>,
    pub visibility: Option<Visibility>,
}

/// Create component request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateComponentRequest {
    pub name: String,
    pub component_type: String,
    pub project_id: Option<Id>,
    pub owner_id: Id,
    pub system_id: Option<Id>,
    pub repository_url: Option<String>,
    pub docs_url: Option<String>,
    pub api_spec_url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub lifecycle: Option<Lifecycle>,
}

/// Project filters
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectFilters {
    pub status: Option<ProjectStatus>,
    pub project_type: Option<String>,
    pub visibility: Option<Visibility>,
    pub owner_id: Option<Id>,
    pub search: Option<String>,
}

// ============================================================================
// Document Types
// ============================================================================

/// Document status lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum DocumentStatus {
    #[serde(rename = "draft")]
    #[default]
    Draft,
    #[serde(rename = "published")]
    Published,
    #[serde(rename = "archived")]
    Archived,
    #[serde(rename = "deleted")]
    Deleted,
}


impl std::fmt::Display for DocumentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentStatus::Draft => write!(f, "Draft"),
            DocumentStatus::Published => write!(f, "Published"),
            DocumentStatus::Archived => write!(f, "Archived"),
            DocumentStatus::Deleted => write!(f, "Deleted"),
        }
    }
}

/// Document visibility settings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum DocumentVisibility {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "private")]
    #[default]
    Private,
    #[serde(rename = "restricted")]
    Restricted,
}


impl std::fmt::Display for DocumentVisibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentVisibility::Public => write!(f, "Public"),
            DocumentVisibility::Private => write!(f, "Private"),
            DocumentVisibility::Restricted => write!(f, "Restricted"),
        }
    }
}

/// Document entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Id,
    pub title: String,
    pub slug: Option<String>,
    pub html: Option<String>,
    pub content: String,
    pub status: String,
    pub visibility: String,
    pub tags: Vec<String>,
    pub author_id: Id,
    pub repository_id: Option<Id>,
    pub word_count: usize,
    pub character_count: usize,
    pub created_at: String,
    pub updated_at: String,
    pub published_at: Option<String>,
}

/// Document list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentListResponse {
    pub results: Vec<Document>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

/// Update document request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDocumentRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

// ============================================================================
// Document Version Types
// ============================================================================

/// Document version for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentVersion {
    pub id: String,
    pub document_id: String,
    pub version_number: i32,
    pub content: String,
    pub commit_message: Option<String>,
    pub created_at: String,
    pub created_by: String,
}

// ============================================================================
// Rendering Types
// ============================================================================

/// Response from the markdown rendering endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderMarkdownResponse {
    pub html: String,
    pub word_count: usize,
    pub character_count: usize,
}

// ============================================================================
// Attachment Types
// ============================================================================

/// Document attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    pub document_id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
    pub created_at: String,
    pub created_by: String,
}

// ============================================================================
// Template Types
// ============================================================================

/// Document template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTemplate {
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

/// Create template request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Update template request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

// ============================================================================
// Plugin Types
// ============================================================================

/// Plugin record from the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
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

/// Create plugin request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePluginRequest {
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

/// Update plugin request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdatePluginRequest {
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

// ============================================================================
// Search Types
// ============================================================================

/// Search filters for faceted search
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilters {
    pub content_type: Option<String>,
    pub status: Option<String>,
    pub visibility: Option<String>,
    pub project_id: Option<String>,
    pub author_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

/// Search result item with highlighting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub id: String,
    pub title: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub status: String,
    pub visibility: String,
    pub tags: Vec<String>,
    pub author_id: String,
    pub project_id: Option<String>,
    pub word_count: i32,
    pub rank: f64,
    pub headline: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Facet count for filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetItem {
    pub value: String,
    pub count: i64,
}

/// Search facets for filtering
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFacets {
    pub content_types: Vec<FacetItem>,
    pub statuses: Vec<FacetItem>,
    pub visibilities: Vec<FacetItem>,
    pub tags: Vec<FacetItem>,
}

/// Search results response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultsResponse {
    pub results: Vec<SearchResultItem>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub facets: SearchFacets,
}

/// Project search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSearchResultItem {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub project_type: String,
    pub status: String,
    pub rank: f64,
}

/// Global search response (documents + projects)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSearchResponse {
    pub documents: SearchResultsResponse,
    pub projects: Vec<ProjectSearchResultItem>,
}

/// Saved search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSearch {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub query: String,
    pub filters: Option<SearchFilters>,
    pub created_at: String,
    pub updated_at: String,
}

/// Create saved search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSavedSearchRequest {
    pub name: String,
    pub query: String,
    pub filters: Option<SearchFilters>,
}

/// Update saved search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSavedSearchRequest {
    pub name: Option<String>,
    pub query: Option<String>,
    pub filters: Option<SearchFilters>,
}

pub fn url_encode(s: &str) -> String {
    js_sys::encode_uri_component(s)
        .as_string()
        .unwrap_or_default()
}

// ============================================================================
// Review Types
// ============================================================================

/// Document review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentReview {
    pub id: String,
    pub document_id: String,
    pub version_number: i32,
    pub status: String,
    pub reviewer_id: String,
    pub summary: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

/// Review comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewComment {
    pub id: String,
    pub review_id: String,
    pub author_id: String,
    pub content: String,
    pub created_at: String,
}

/// Review status summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewStatusSummary {
    pub pending_count: i64,
    pub latest_status: Option<String>,
}

/// Server-side diff line
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub content: String,
    pub line_type: String,
}

/// Server-side diff response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentDiffResponse {
    pub old_lines: Vec<DiffLine>,
    pub new_lines: Vec<DiffLine>,
    pub stats: DiffStats,
}

/// Diff statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStats {
    pub added: usize,
    pub removed: usize,
    pub unchanged: usize,
}

// ============================================================================
// Conflict Types
// ============================================================================

/// Conflict information for a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub document_id: String,
    pub has_conflict: bool,
    pub base_content: Option<String>,
    pub current_content: Option<String>,
    pub incoming_content: Option<String>,
    pub merge_result: Option<MergeResultInfo>,
}

/// Merge result information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResultInfo {
    pub status: String,
    pub content: String,
    pub conflict_count: usize,
}

// ============================================================================
// Activity Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActivityEvent {
    pub id: String,
    pub actor_id: String,
    pub event_type: String,
    pub target_type: String,
    pub target_id: String,
    pub description: String,
    pub metadata: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ActivityListResponse {
    pub events: Vec<ActivityEvent>,
    pub count: usize,
}

// ============================================================================
// Notification Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Notification {
    pub id: String,
    pub user_id: String,
    #[serde(rename = "type")]
    pub notification_type: String,
    pub title: String,
    pub body: Option<String>,
    pub link: Option<String>,
    pub read: bool,
    pub metadata: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NotificationListResponse {
    pub notifications: Vec<Notification>,
    pub count: usize,
}

// ============================================================================
// Backlink Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct BacklinksResponse {
    pub backlinks: Vec<BacklinkItem>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BacklinkItem {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub updated_at: String,
}

// ============================================================================
// Tag Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TagInfo {
    pub tag: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TagsResponse {
    pub tags: Vec<TagInfo>,
    pub total: usize,
}

// ============================================================================
// Webhook Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WebhookInfo {
    pub id: String,
    pub url: String,
    pub events: Vec<String>,
    pub active: bool,
    pub created_at: String,
    pub last_triggered_at: Option<String>,
}

// ============================================================================
// Space Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Space {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: String,
    pub color: String,
    pub owner_id: String,
    pub parent_id: Option<String>,
    pub visibility: String,
    pub sort_order: i32,
    pub is_default: bool,
    pub document_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CreateSpaceRequest {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub parent_id: Option<String>,
    pub visibility: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct UpdateSpaceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub parent_id: Option<Option<String>>,
    pub visibility: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpaceMember {
    pub id: String,
    pub space_id: String,
    pub user_id: String,
    pub role: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub joined_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AddSpaceMemberRequest {
    pub user_id: String,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct UpdateSpaceMemberRequest {
    pub role: String,
}

// ============================================================================
// SSG Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SsgBuildRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub base_url: Option<String>,
    pub theme: Option<String>,
    pub custom_css: Option<String>,
    pub nav_links: Option<Vec<SsgNavLink>>,
    pub group_by_tag: Option<bool>,
    pub project_id: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SsgNavLink {
    pub label: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SsgBuildResponse {
    pub success: bool,
    pub result: SsgBuildResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SsgBuildResult {
    pub pages: usize,
    pub category_pages: usize,
    pub total_files: usize,
    pub build_time_ms: u64,
    pub output_size_bytes: u64,
    pub generated_pages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SsgConfig {
    pub title: String,
    pub description: String,
    pub base_url: String,
    pub theme: String,
    pub custom_css: Option<String>,
    pub nav_links: Vec<SsgNavLink>,
    pub group_by_tag: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SsgStatus {
    pub last_build: Option<SsgBuildResult>,
    pub available: bool,
}

// ============================================================================
// Billing Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BillingPlanInfo {
    pub name: String,
    pub price_monthly_cents: u64,
    pub max_documents: usize,
    pub max_members: usize,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct BillingPlansResponse {
    pub plans: Vec<BillingPlanInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubscriptionInfo {
    pub id: String,
    pub organization_id: String,
    pub plan: String,
    pub status: String,
    pub payment_method_id: Option<String>,
    pub cancel_at_period_end: bool,
    pub current_period_start: String,
    pub current_period_end: String,
    pub trial_end: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanDetails {
    pub name: String,
    pub price_monthly_cents: u64,
    pub max_documents: usize,
    pub max_members: usize,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubscriptionResponse {
    pub subscription: SubscriptionInfo,
    pub plan_details: PlanDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvoiceInfo {
    pub id: String,
    pub subscription_id: String,
    pub organization_id: String,
    pub amount_cents: i64,
    pub currency: String,
    pub description: String,
    pub status: String,
    pub paid_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct InvoicesResponse {
    pub invoices: Vec<InvoiceInfo>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsageMetrics {
    pub organization_id: String,
    pub period_start: String,
    pub period_end: String,
    pub documents_created: usize,
    pub documents_total: usize,
    pub members_total: usize,
    pub storage_bytes: u64,
    pub plan: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsageResponse {
    pub usage: UsageMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MandateResponse {
    pub mandate_id: String,
    pub authorization_url: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MandateStatusResponse {
    pub mandate_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaymentResponse {
    pub payment_id: String,
    pub status: String,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaymentStatusResponse {
    pub payment_id: String,
    pub status: String,
}

// ============================================================================
// Audit Log Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditLogEntry {
    pub id: String,
    pub action: String,
    pub actor_id: String,
    pub actor_name: Option<String>,
    pub target_type: String,
    pub target_id: String,
    pub details: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AuditLogResponse {
    pub entries: Vec<AuditLogEntry>,
    pub total: usize,
    pub page: u32,
    pub page_size: u32,
}
