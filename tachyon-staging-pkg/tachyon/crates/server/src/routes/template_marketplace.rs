//! Template marketplace API scaffolding.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateListing {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub author: String,
    pub version: String,
    pub downloads: u64,
    pub rating: f32,
    pub tags: Vec<String>,
    pub preview_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceResponse {
    pub templates: Vec<TemplateListing>,
    pub total: usize,
    pub page: usize,
}

#[derive(Debug, Deserialize)]
pub struct MarketplaceQuery {
    pub category: Option<String>,
    pub search: Option<String>,
    pub sort: Option<String>,
    pub page: Option<usize>,
}

pub fn query_marketplace(_query: &MarketplaceQuery) -> MarketplaceResponse {
    MarketplaceResponse {
        templates: vec![],
        total: 0,
        page: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_listing_serialization() {
        let t = TemplateListing {
            id: "1".to_string(),
            name: "Meeting Notes".to_string(),
            description: "A template for meeting notes".to_string(),
            category: "business".to_string(),
            author: "Tachyon Team".to_string(),
            version: "1.0.0".to_string(),
            downloads: 150,
            rating: 4.5,
            tags: vec!["meeting".to_string(), "notes".to_string()],
            preview_url: None,
        };
        let json = serde_json::to_string(&t).unwrap();
        assert!(json.contains("Meeting Notes"));
        assert!(json.contains("business"));
    }

    #[test]
    fn test_marketplace_response() {
        let resp = query_marketplace(&MarketplaceQuery {
            category: Some("business".to_string()),
            search: None,
            sort: None,
            page: None,
        });
        assert_eq!(resp.total, 0);
        assert_eq!(resp.page, 1);
    }

    #[test]
    fn test_marketplace_query_default() {
        let query = MarketplaceQuery {
            category: None,
            search: None,
            sort: None,
            page: None,
        };
        let resp = query_marketplace(&query);
        assert!(resp.templates.is_empty());
    }
}
