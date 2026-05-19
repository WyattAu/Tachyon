//! Faceted search support for structured queries.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFacets {
    pub tags: Vec<String>,
    pub authors: Vec<String>,
    pub date_range: Option<DateRange>,
    pub content_types: Vec<String>,
    pub spaces: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightedResult {
    pub document_id: String,
    pub title: String,
    pub slug: String,
    pub score: f64,
    pub snippet: String,
    pub highlight_ranges: Vec<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSearch {
    pub id: String,
    pub name: String,
    pub query: String,
    pub facets: SearchFacets,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_facets() {
        let facets = SearchFacets {
            tags: vec![],
            authors: vec![],
            date_range: None,
            content_types: vec![],
            spaces: vec![],
        };
        let json = serde_json::to_string(&facets).unwrap();
        let parsed: SearchFacets = serde_json::from_str(&json).unwrap();
        assert!(parsed.tags.is_empty());
        assert!(parsed.date_range.is_none());
    }

    #[test]
    fn test_facets_with_date_range() {
        let facets = SearchFacets {
            tags: vec!["rust".to_string()],
            authors: vec!["alice".to_string()],
            date_range: Some(DateRange {
                from: Some("2024-01-01T00:00:00Z".to_string()),
                to: Some("2024-12-31T23:59:59Z".to_string()),
            }),
            content_types: vec!["markdown".to_string()],
            spaces: vec!["engineering".to_string()],
        };
        let json = serde_json::to_string(&facets).unwrap();
        let parsed: SearchFacets = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.tags, vec!["rust"]);
        assert_eq!(parsed.authors, vec!["alice"]);
        assert!(parsed.date_range.is_some());
        let dr = parsed.date_range.unwrap();
        assert_eq!(dr.from.as_deref(), Some("2024-01-01T00:00:00Z"));
        assert_eq!(dr.to.as_deref(), Some("2024-12-31T23:59:59Z"));
    }

    #[test]
    fn test_highlighted_result() {
        let result = HighlightedResult {
            document_id: "doc-123".to_string(),
            title: "Test Document".to_string(),
            slug: "test-document".to_string(),
            score: 0.95,
            snippet: "This is a <em>test</em> document".to_string(),
            highlight_ranges: vec![(10, 14)],
        };
        let json = serde_json::to_string(&result).unwrap();
        let parsed: HighlightedResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.document_id, "doc-123");
        assert_eq!(parsed.score, 0.95);
        assert_eq!(parsed.highlight_ranges, vec![(10, 14)]);
    }

    #[test]
    fn test_saved_search_roundtrip() {
        let saved = SavedSearch {
            id: "search-1".to_string(),
            name: "My Search".to_string(),
            query: "rust programming".to_string(),
            facets: SearchFacets {
                tags: vec!["rust".to_string()],
                authors: vec![],
                date_range: None,
                content_types: vec![],
                spaces: vec![],
            },
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-06-15T12:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&saved).unwrap();
        let parsed: SavedSearch = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "search-1");
        assert_eq!(parsed.name, "My Search");
        assert_eq!(parsed.query, "rust programming");
        assert_eq!(parsed.facets.tags, vec!["rust"]);
    }
}
