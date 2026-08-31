//! Cursor-based pagination support.
//!
//! Provides `Cursor` and `CursorPage` types for efficient pagination through
//! large result sets without offset-based performance degradation.

use serde::{Deserialize, Serialize};
use std::fmt;

/// An opaque pagination cursor encoding the position and sort direction.
/// Format: `{id}:{direction}` where direction is "asc" or "desc".
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Cursor(pub String);

impl fmt::Display for Cursor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Cursor {
    /// Encode a cursor from an ID and direction.
    pub fn encode(id: &str, direction: &str) -> Self {
        Self(format!("{}:{}", id, direction))
    }

    /// Decode a cursor into (id, direction).
    pub fn decode(&self) -> Option<(String, String)> {
        let parts: Vec<&str> = self.0.splitn(2, ':').collect();
        if parts.len() == 2 {
            Some((parts[0].to_string(), parts[1].to_string()))
        } else {
            None
        }
    }
}

/// Parameters for cursor-based pagination requests.
#[derive(Debug, Clone, Deserialize, Default, utoipa::IntoParams)]
pub struct CursorParams {
    /// Cursor from the previous page's `next_cursor`.
    pub after: Option<String>,
    /// Cursor from the previous page's `prev_cursor`.
    pub before: Option<String>,
    /// Maximum items per page (default: 20, max: 100).
    pub limit: Option<usize>,
}

impl CursorParams {
    pub fn limit(&self) -> usize {
        self.limit.unwrap_or(20).clamp(1, 100)
    }

    pub fn direction(&self) -> &str {
        if self.before.is_some() { "desc" } else { "asc" }
    }
}

/// A page of results with cursor-based pagination metadata.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CursorPage<T: Serialize + utoipa::ToSchema> {
    pub data: Vec<T>,
    pub has_next: bool,
    pub has_prev: bool,
    pub next_cursor: Option<String>,
    pub prev_cursor: Option<String>,
    pub total_count: Option<i64>,
}

impl<T: Serialize + utoipa::ToSchema> CursorPage<T> {
    pub fn new(data: Vec<T>, has_next: bool, has_prev: bool) -> Self {
        Self {
            data,
            has_next,
            has_prev,
            next_cursor: None,
            prev_cursor: None,
            total_count: None,
        }
    }

    /// Set cursors based on first and last item IDs.
    pub fn with_cursors(
        mut self,
        first_id: Option<&str>,
        last_id: Option<&str>,
        direction: &str,
    ) -> Self {
        if self.has_next {
            if direction == "asc" {
                self.next_cursor = last_id.map(|id| Cursor::encode(id, "asc").to_string());
            } else {
                self.next_cursor = first_id.map(|id| Cursor::encode(id, "desc").to_string());
            }
        }
        if self.has_prev {
            if direction == "asc" {
                self.prev_cursor = first_id.map(|id| Cursor::encode(id, "desc").to_string());
            } else {
                self.prev_cursor = last_id.map(|id| Cursor::encode(id, "asc").to_string());
            }
        }
        self
    }

    pub fn with_total_count(mut self, count: i64) -> Self {
        self.total_count = Some(count);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_encode_decode() {
        let c = Cursor::encode("abc123", "asc");
        assert_eq!(c.0, "abc123:asc");
        let (id, dir) = c.decode().unwrap();
        assert_eq!(id, "abc123");
        assert_eq!(dir, "asc");
    }

    #[test]
    fn test_cursor_params_defaults() {
        let params = CursorParams::default();
        assert_eq!(params.limit(), 20);
        assert_eq!(params.direction(), "asc");
    }

    #[test]
    fn test_cursor_params_before() {
        let params = CursorParams {
            before: Some("abc".to_string()),
            after: None,
            limit: Some(50),
        };
        assert_eq!(params.limit(), 50);
        assert_eq!(params.direction(), "desc");
    }

    #[test]
    fn test_cursor_page_with_cursors() {
        let page =
            CursorPage::new(vec!["a", "b"], true, true).with_cursors(Some("a"), Some("b"), "asc");
        assert_eq!(page.next_cursor.as_deref(), Some("b:asc"));
        assert_eq!(page.prev_cursor.as_deref(), Some("a:desc"));
    }
}
