//! GDPR Article 20: Right to data portability.
//!
//! Aggregates all user data into a structured JSON/ZIP export.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{ImportExportError, ImportExportResult};

/// Complete user data export for GDPR compliance.
#[derive(Debug, Serialize, Deserialize)]
pub struct GdprUserExport {
    /// Export metadata.
    pub export_info: ExportMetadata,
    /// User profile data.
    pub profile: UserProfile,
    /// All documents authored by the user.
    pub documents: Vec<UserDocument>,
    /// All comments authored by the user.
    pub comments: Vec<UserComment>,
    /// Recent activity log.
    pub activities: Vec<UserActivity>,
    /// User preferences and settings.
    pub preferences: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub exported_at: String,
    pub format_version: String,
    pub user_id: String,
    pub total_documents: usize,
    pub total_comments: usize,
    pub total_activities: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub role: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDocument {
    pub id: String,
    pub title: String,
    pub content: String,
    pub slug: String,
    pub tags: Vec<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserComment {
    pub id: String,
    pub document_id: String,
    pub content: String,
    pub created_at: Option<String>,
    pub is_resolved: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserActivity {
    pub id: String,
    pub event_type: String,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub description: Option<String>,
    pub created_at: Option<String>,
}

/// Build a GDPR export from individual data components.
pub struct GdprExportBuilder {
    user_id: String,
    profile: Option<UserProfile>,
    documents: Vec<UserDocument>,
    comments: Vec<UserComment>,
    activities: Vec<UserActivity>,
    preferences: HashMap<String, serde_json::Value>,
}

impl GdprExportBuilder {
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            profile: None,
            documents: Vec::new(),
            comments: Vec::new(),
            activities: Vec::new(),
            preferences: HashMap::new(),
        }
    }

    pub fn profile(mut self, profile: UserProfile) -> Self {
        self.profile = Some(profile);
        self
    }

    pub fn add_document(mut self, doc: UserDocument) -> Self {
        self.documents.push(doc);
        self
    }

    pub fn documents(mut self, docs: Vec<UserDocument>) -> Self {
        self.documents = docs;
        self
    }

    pub fn add_comment(mut self, comment: UserComment) -> Self {
        self.comments.push(comment);
        self
    }

    pub fn comments(mut self, comments: Vec<UserComment>) -> Self {
        self.comments = comments;
        self
    }

    pub fn add_activity(mut self, activity: UserActivity) -> Self {
        self.activities.push(activity);
        self
    }

    pub fn activities(mut self, activities: Vec<UserActivity>) -> Self {
        self.activities = activities;
        self
    }

    pub fn preferences(mut self, prefs: HashMap<String, serde_json::Value>) -> Self {
        self.preferences = prefs;
        self
    }

    /// Build the final export, serializing to JSON bytes.
    pub fn build(self) -> ImportExportResult<Vec<u8>> {
        let doc_count = self.documents.len();
        let comment_count = self.comments.len();
        let activity_count = self.activities.len();

        let export = GdprUserExport {
            export_info: ExportMetadata {
                exported_at: Utc::now().to_rfc3339(),
                format_version: "1.0.0".to_string(),
                user_id: self.user_id.clone(),
                total_documents: doc_count,
                total_comments: comment_count,
                total_activities: activity_count,
            },
            profile: self.profile.unwrap_or(UserProfile {
                id: self.user_id.clone(),
                username: String::new(),
                display_name: None,
                email: None,
                role: String::new(),
                created_at: None,
                updated_at: None,
            }),
            documents: self.documents,
            comments: self.comments,
            activities: self.activities,
            preferences: self.preferences,
        };

        serde_json::to_vec_pretty(&export).map_err(|e| {
            ImportExportError::export(format!("Failed to serialize GDPR export: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gdpr_export_builder() {
        let export = GdprExportBuilder::new("user-123".to_string())
            .profile(UserProfile {
                id: "user-123".to_string(),
                username: "testuser".to_string(),
                display_name: Some("Test User".to_string()),
                email: Some("test@example.com".to_string()),
                role: "admin".to_string(),
                created_at: Some("2024-01-01T00:00:00Z".to_string()),
                updated_at: None,
            })
            .documents(vec![UserDocument {
                id: "doc-1".to_string(),
                title: "My Doc".to_string(),
                content: "Hello world".to_string(),
                slug: "my-doc".to_string(),
                tags: vec!["test".to_string()],
                created_at: Some("2024-01-01T00:00:00Z".to_string()),
                updated_at: None,
            }])
            .comments(vec![UserComment {
                id: "cmt-1".to_string(),
                document_id: "doc-1".to_string(),
                content: "Great doc!".to_string(),
                created_at: Some("2024-01-02T00:00:00Z".to_string()),
                is_resolved: false,
            }])
            .build()
            .unwrap();

        let json_str = String::from_utf8(export).unwrap();
        assert!(json_str.contains("\"user_id\": \"user-123\""));
        assert!(json_str.contains("\"username\": \"testuser\""));
        assert!(json_str.contains("\"total_documents\": 1"));
        assert!(json_str.contains("\"total_comments\": 1"));
        assert!(json_str.contains("\"format_version\": \"1.0.0\""));
        assert!(json_str.contains("\"exported_at\":"));
    }

    #[test]
    fn test_gdpr_export_empty() {
        let export = GdprExportBuilder::new("user-456".to_string())
            .build()
            .unwrap();

        let parsed: serde_json::Value = serde_json::from_slice(&export).unwrap();
        assert_eq!(parsed["export_info"]["total_documents"], 0);
        assert_eq!(parsed["export_info"]["total_comments"], 0);
    }
}
