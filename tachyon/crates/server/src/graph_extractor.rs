use regex::Regex;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use tachyon_core::id::DocumentId;
use tachyon_database::{
    DatabaseError, DatabasePool, DatabaseResult, DocumentRepository, GraphEdge, GraphNode,
    GraphRepository,
};
use tracing::{debug, info, instrument};

static MARKDOWN_LINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]*)\]\(([^)]+)\)").unwrap());

// ============================================================================
// Result & Config Types
// ============================================================================

#[derive(Debug, Default)]
pub struct ExtractionResult {
    pub nodes_created: usize,
    pub edges_created: usize,
    pub nodes_skipped: usize,
    pub errors: Vec<String>,
}

impl ExtractionResult {
    fn merge(&mut self, other: ExtractionResult) {
        self.nodes_created += other.nodes_created;
        self.edges_created += other.edges_created;
        self.nodes_skipped += other.nodes_skipped;
        self.errors.extend(other.errors);
    }
}

#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    pub extract_headings: bool,
    pub extract_links: bool,
    pub extract_tags: bool,
    pub min_heading_level: u32,
    pub max_heading_level: u32,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            extract_headings: true,
            extract_links: true,
            extract_tags: true,
            min_heading_level: 2,
            max_heading_level: 4,
        }
    }
}

// ============================================================================
// GraphExtractor
// ============================================================================

pub struct GraphExtractor {
    pool: DatabasePool,
    config: ExtractionConfig,
}

impl GraphExtractor {
    pub fn new(pool: DatabasePool, config: ExtractionConfig) -> Self {
        Self { pool, config }
    }

    // -----------------------------------------------------------------------
    // Main pipeline
    // -----------------------------------------------------------------------

    #[instrument(skip(self), fields(document_id = %document_id))]
    pub async fn extract_document(&self, document_id: &str) -> DatabaseResult<ExtractionResult> {
        let doc_repo = DocumentRepository::new(self.pool.clone());
        let graph_repo = GraphRepository::new(self.pool.clone());

        let doc_id = DocumentId::parse_str(document_id)
            .map_err(|e| DatabaseError::ValidationError(format!("Invalid document ID: {}", e)))?;
        let doc = doc_repo.get_by_id(&doc_id).await?;

        let mut result = ExtractionResult::default();

        let content = doc.content.as_deref().unwrap_or("");
        let (frontmatter, body) = Self::parse_frontmatter(content);

        let doc_node_id = self
            .upsert_document_node(&graph_repo, &doc)
            .await
            .map_err(|e| {
                result.errors.push(format!("document node: {}", e));
                e
            })?;

        if self.config.extract_tags {
            let tags = Self::extract_tags_from_frontmatter(&frontmatter);
            let tag_result = self
                .process_tags(&graph_repo, &doc_node_id, &tags, document_id)
                .await;
            result.merge(tag_result);
        }

        if self.config.extract_headings {
            let headings = Self::parse_markdown_headings(
                body,
                self.config.min_heading_level,
                self.config.max_heading_level,
            );
            let heading_result = self
                .process_headings(&graph_repo, &doc_node_id, &headings, document_id)
                .await;
            result.merge(heading_result);
        }

        if self.config.extract_links {
            let links = Self::parse_markdown_links(body);
            let link_result = self
                .process_links(&graph_repo, &doc_repo, &doc_node_id, &links, &doc.slug)
                .await;
            result.merge(link_result);
        }

        info!(
            "Extraction complete for {}: {} nodes, {} edges, {} skipped, {} errors",
            document_id,
            result.nodes_created,
            result.edges_created,
            result.nodes_skipped,
            result.errors.len()
        );

        Ok(result)
    }

    #[instrument(skip(self))]
    pub async fn extract_all_documents(&self) -> DatabaseResult<ExtractionResult> {
        let doc_repo = DocumentRepository::new(self.pool.clone());
        let mut total = ExtractionResult::default();

        let mut offset: i64 = 0;
        let batch_size: i64 = 100;
        loop {
            let docs: Vec<_> = doc_repo.list_all(Some(batch_size), Some(offset)).await?;
            if docs.is_empty() {
                break;
            }
            for doc in &docs {
                match self.extract_document(&doc.id).await {
                    Ok(r) => total.merge(r),
                    Err(e) => total.errors.push(format!("{}: {}", doc.id, e)),
                }
            }
            offset += batch_size;
        }

        Ok(total)
    }

    #[instrument(skip(self), fields(document_id = %document_id))]
    pub async fn remove_document(&self, document_id: &str) -> DatabaseResult<()> {
        let graph_repo = GraphRepository::new(self.pool.clone());

        let doc_slug = format!("doc:{}", document_id);
        match graph_repo.get_node_by_slug(&doc_slug).await {
            Ok(node) => {
                let _ = graph_repo.deactivate_edges_for_node(&node.id).await;
                graph_repo.deactivate_node(&node.id).await?;
                info!("Removed graph entities for document {}", document_id);
            }
            Err(DatabaseError::NotFound { .. }) => {
                debug!("No graph node found for document {}", document_id);
            }
            Err(e) => return Err(e),
        }

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Frontmatter parsing
    // -----------------------------------------------------------------------

    pub fn parse_frontmatter(content: &str) -> (serde_yaml::Value, &str) {
        let trimmed = content.trim_start();
        if !trimmed.starts_with("---") {
            return (serde_yaml::Value::Null, content);
        }

        let after_delim = &trimmed[3..];
        let end = match after_delim.find("\n---") {
            Some(pos) => pos,
            None => return (serde_yaml::Value::Null, content),
        };

        let yaml_block = &after_delim[..end];
        let body = after_delim[end + 4..].trim_start_matches('\n');

        let frontmatter = serde_yaml::from_str(yaml_block).unwrap_or(serde_yaml::Value::Null);
        (frontmatter, body)
    }

    fn extract_tags_from_frontmatter(frontmatter: &serde_yaml::Value) -> Vec<String> {
        let tags_val = match frontmatter {
            serde_yaml::Value::Mapping(m) => m.get(serde_yaml::Value::String("tags".to_string())),
            _ => None,
        };
        match tags_val {
            Some(serde_yaml::Value::Sequence(seq)) => seq
                .iter()
                .filter_map(|v| match v {
                    serde_yaml::Value::String(s) => Some(s.clone()),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Markdown link parsing
    // -----------------------------------------------------------------------

    pub fn parse_markdown_links(content: &str) -> Vec<(String, String)> {
        MARKDOWN_LINK_RE
            .captures_iter(content)
            .filter_map(|cap| {
                let text = cap.get(1)?.as_str().trim().to_string();
                let url = cap.get(2)?.as_str().trim().to_string();
                if text.is_empty() || url.is_empty() {
                    return None;
                }
                Some((text, url))
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // Markdown heading parsing
    // -----------------------------------------------------------------------

    pub fn parse_markdown_headings(
        content: &str,
        min_level: u32,
        max_level: u32,
    ) -> Vec<(u32, String)> {
        content
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim_start();
                if !trimmed.starts_with('#') {
                    return None;
                }

                let hashes = trimmed.chars().take_while(|&c| c == '#').count() as u32;
                if hashes < min_level || hashes > max_level {
                    return None;
                }

                let text = trimmed[hashes as usize..].trim();
                if text.is_empty() {
                    return None;
                }

                Some((hashes, text.to_string()))
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // Node / edge creation helpers
    // -----------------------------------------------------------------------

    async fn upsert_document_node(
        &self,
        graph_repo: &GraphRepository,
        doc: &tachyon_database::DocumentMetadata,
    ) -> Result<String, DatabaseError> {
        let slug = format!("doc:{}", doc.id);
        let word_count = doc.word_count;
        let character_count = doc.character_count;

        match graph_repo.get_node_by_slug(&slug).await {
            Ok(existing) => {
                graph_repo
                    .update_node(
                        &existing.id,
                        Some(&doc.title),
                        None,
                        doc.description.as_deref(),
                        None,
                        None,
                        None,
                        Some(&json!({
                            "word_count": word_count,
                            "character_count": character_count,
                            "status": doc.status,
                            "visibility": doc.visibility,
                        })),
                    )
                    .await?;
                Ok(existing.id)
            }
            Err(DatabaseError::NotFound { .. }) => {
                let node = GraphNode {
                    id: uuid::Uuid::new_v4().to_string(),
                    node_type: "document".to_string(),
                    name: doc.title.clone(),
                    slug: Some(slug),
                    description: doc.description.clone(),
                    content: None,
                    visibility: doc.visibility.clone(),
                    weight: 1.0,
                    properties: json!({
                        "word_count": word_count,
                        "character_count": character_count,
                        "status": doc.status,
                        "visibility": doc.visibility,
                    }),
                    project_id: doc.project_id.clone(),
                    document_id: Some(doc.id.clone()),
                    created_by: Some(doc.author_id.clone()),
                    is_active: true,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    deactivated_at: None,
                };
                let created = graph_repo.create_node(&node).await?;
                Ok(created.id)
            }
            Err(e) => Err(e),
        }
    }

    async fn process_tags(
        &self,
        graph_repo: &GraphRepository,
        doc_node_id: &str,
        tags: &[String],
        _document_id: &str,
    ) -> ExtractionResult {
        let mut result = ExtractionResult::default();
        let mut seen = HashSet::new();
        let unique_tags: Vec<String> = tags
            .iter()
            .filter(|t| seen.insert((*t).clone()))
            .cloned()
            .collect();

        if unique_tags.is_empty() {
            return result;
        }

        let tag_slugs: Vec<String> = unique_tags
            .iter()
            .map(|t| format!("tag:{}", Self::slugify(t)))
            .collect();

        let existing_nodes = match graph_repo.get_nodes_by_slugs_batch(&tag_slugs).await {
            Ok(nodes) => nodes,
            Err(e) => {
                for _tag in &unique_tags {
                    result.errors.push(format!("tag batch lookup: {}", e));
                }
                return result;
            }
        };

        let slug_to_node: HashMap<String, GraphNode> = existing_nodes
            .into_iter()
            .filter_map(|n| n.slug.clone().map(|s| (s, n)))
            .collect();

        for tag in &unique_tags {
            let tag_slug = format!("tag:{}", Self::slugify(tag));
            let tag_node_id = if let Some(existing) = slug_to_node.get(&tag_slug) {
                existing.id.clone()
            } else {
                match self
                    .get_or_create_concept_node(graph_repo, tag, &tag_slug)
                    .await
                {
                    Ok(id) => id,
                    Err(e) => {
                        result.errors.push(format!("tag '{}': {}", tag, e));
                        continue;
                    }
                }
            };

            match self
                .create_edge_if_missing(
                    graph_repo,
                    doc_node_id,
                    &tag_node_id,
                    "tagged_with",
                    Some(tag),
                    1.5,
                )
                .await
            {
                Ok(true) => result.edges_created += 1,
                Ok(false) => result.nodes_skipped += 1,
                Err(e) => result.errors.push(format!("tag edge '{}': {}", tag, e)),
            }
        }

        result
    }

    async fn process_headings(
        &self,
        graph_repo: &GraphRepository,
        doc_node_id: &str,
        headings: &[(u32, String)],
        document_id: &str,
    ) -> ExtractionResult {
        let mut result = ExtractionResult::default();

        for (level, text) in headings {
            let concept_slug = format!("concept:{}:{}", document_id, Self::slugify(text));
            let concept_node_id = match self
                .get_or_create_concept_node(graph_repo, text, &concept_slug)
                .await
            {
                Ok(id) => id,
                Err(e) => {
                    result.errors.push(format!("heading '{}': {}", text, e));
                    continue;
                }
            };

            match self
                .create_edge_if_missing(
                    graph_repo,
                    &concept_node_id,
                    doc_node_id,
                    "part_of",
                    Some(&format!("{} heading", level)),
                    1.0,
                )
                .await
            {
                Ok(true) => result.edges_created += 1,
                Ok(false) => result.nodes_skipped += 1,
                Err(e) => result
                    .errors
                    .push(format!("heading edge '{}': {}", text, e)),
            }

            if let Err(e) = graph_repo
                .update_node(
                    &concept_node_id,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(&json!({
                        "level": level,
                        "document_id": document_id,
                    })),
                )
                .await
            {
                result
                    .errors
                    .push(format!("heading props '{}': {}", text, e));
            }
        }

        result
    }

    async fn process_links(
        &self,
        graph_repo: &GraphRepository,
        doc_repo: &DocumentRepository,
        doc_node_id: &str,
        links: &[(String, String)],
        _doc_slug: &Option<String>,
    ) -> ExtractionResult {
        let mut result = ExtractionResult::default();

        let internal_slugs: Vec<String> = links
            .iter()
            .filter_map(|(_, url)| {
                let is_internal = url.starts_with('/')
                    || (!url.starts_with("http://")
                        && !url.starts_with("https://")
                        && !url.contains('.'));
                if is_internal {
                    Some(url.trim_start_matches('/').to_string())
                } else {
                    None
                }
            })
            .collect();

        let doc_by_slug: HashMap<String, tachyon_database::DocumentMetadata> =
            if !internal_slugs.is_empty() {
                match doc_repo.get_by_slugs_batch(&internal_slugs).await {
                    Ok(docs) => docs
                        .into_iter()
                        .filter_map(|d| d.slug.clone().map(|s| (s, d)))
                        .collect(),
                    Err(e) => {
                        result.errors.push(format!("batch slug lookup: {}", e));
                        HashMap::new()
                    }
                }
            } else {
                HashMap::new()
            };

        let internal_node_slugs: Vec<String> = doc_by_slug
            .values()
            .map(|d| format!("doc:{}", d.id))
            .collect();

        let existing_graph_nodes: HashMap<String, GraphNode> = if !internal_node_slugs.is_empty() {
            match graph_repo
                .get_nodes_by_slugs_batch(&internal_node_slugs)
                .await
            {
                Ok(nodes) => nodes
                    .into_iter()
                    .filter_map(|n| n.slug.clone().map(|s| (s, n)))
                    .collect(),
                Err(_) => HashMap::new(),
            }
        } else {
            HashMap::new()
        };

        for (text, url) in links {
            let is_internal = url.starts_with('/')
                || (!url.starts_with("http://")
                    && !url.starts_with("https://")
                    && !url.contains('.'));

            if is_internal {
                let target_slug = url.trim_start_matches('/');
                if let Some(target_doc) = doc_by_slug.get(target_slug) {
                    let target_graph_slug = format!("doc:{}", target_doc.id);
                    if let Some(target_node) = existing_graph_nodes.get(&target_graph_slug) {
                        match self
                            .create_edge_if_missing(
                                graph_repo,
                                doc_node_id,
                                &target_node.id,
                                "references",
                                Some(text),
                                1.0,
                            )
                            .await
                        {
                            Ok(true) => result.edges_created += 1,
                            Ok(false) => result.nodes_skipped += 1,
                            Err(e) => result.errors.push(format!("link edge '{}': {}", url, e)),
                        }
                    } else {
                        result.errors.push(format!(
                            "target document node not found for slug: {}",
                            target_slug
                        ));
                    }
                } else {
                    debug!("Internal link target not found: {}", target_slug);
                }
            } else {
                let ref_slug = format!("ref:{}", Self::slugify(url));
                let ref_node_id = match self
                    .get_or_create_reference_node(graph_repo, text, &ref_slug, url)
                    .await
                {
                    Ok(id) => id,
                    Err(e) => {
                        result
                            .errors
                            .push(format!("reference node '{}': {}", url, e));
                        continue;
                    }
                };

                match self
                    .create_edge_if_missing(
                        graph_repo,
                        doc_node_id,
                        &ref_node_id,
                        "references",
                        Some(text),
                        1.0,
                    )
                    .await
                {
                    Ok(true) => result.edges_created += 1,
                    Ok(false) => result.nodes_skipped += 1,
                    Err(e) => result
                        .errors
                        .push(format!("reference edge '{}': {}", url, e)),
                }
            }
        }

        result
    }

    // -----------------------------------------------------------------------
    // Dedup helpers
    // -----------------------------------------------------------------------

    async fn get_or_create_concept_node(
        &self,
        graph_repo: &GraphRepository,
        name: &str,
        slug: &str,
    ) -> Result<String, DatabaseError> {
        match graph_repo.get_node_by_slug(slug).await {
            Ok(existing) => Ok(existing.id),
            Err(DatabaseError::NotFound { .. }) => {
                let node = GraphNode {
                    id: uuid::Uuid::new_v4().to_string(),
                    node_type: "concept".to_string(),
                    name: name.to_string(),
                    slug: Some(slug.to_string()),
                    description: None,
                    content: None,
                    visibility: "public".to_string(),
                    weight: 1.0,
                    properties: json!({}),
                    project_id: None,
                    document_id: None,
                    created_by: None,
                    is_active: true,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    deactivated_at: None,
                };
                let created = graph_repo.create_node(&node).await?;
                Ok(created.id)
            }
            Err(e) => Err(e),
        }
    }

    async fn get_or_create_reference_node(
        &self,
        graph_repo: &GraphRepository,
        label: &str,
        slug: &str,
        url: &str,
    ) -> Result<String, DatabaseError> {
        match graph_repo.get_node_by_slug(slug).await {
            Ok(existing) => Ok(existing.id),
            Err(DatabaseError::NotFound { .. }) => {
                let node = GraphNode {
                    id: uuid::Uuid::new_v4().to_string(),
                    node_type: "reference".to_string(),
                    name: label.to_string(),
                    slug: Some(slug.to_string()),
                    description: None,
                    content: Some(url.to_string()),
                    visibility: "public".to_string(),
                    weight: 0.5,
                    properties: json!({ "url": url }),
                    project_id: None,
                    document_id: None,
                    created_by: None,
                    is_active: true,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    deactivated_at: None,
                };
                let created = graph_repo.create_node(&node).await?;
                Ok(created.id)
            }
            Err(e) => Err(e),
        }
    }

    async fn create_edge_if_missing(
        &self,
        graph_repo: &GraphRepository,
        source_id: &str,
        target_id: &str,
        edge_type: &str,
        label: Option<&str>,
        weight: f64,
    ) -> Result<bool, DatabaseError> {
        let existing = graph_repo
            .list_edges(Some(source_id), Some(target_id), Some(edge_type), None)
            .await?;

        if !existing.is_empty() {
            return Ok(false);
        }

        let edge = GraphEdge {
            id: uuid::Uuid::new_v4().to_string(),
            source_id: source_id.to_string(),
            target_id: target_id.to_string(),
            edge_type: edge_type.to_string(),
            label: label.map(|l| l.to_string()),
            description: None,
            weight,
            confidence: None,
            properties: json!({}),
            project_id: None,
            created_by: None,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deactivated_at: None,
        };

        graph_repo.create_edge(&edge).await?;
        Ok(true)
    }

    // -----------------------------------------------------------------------
    // Slugify
    // -----------------------------------------------------------------------

    fn slugify(text: &str) -> String {
        text.to_lowercase()
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else if c.is_whitespace() {
                    '-'
                } else {
                    '\0'
                }
            })
            .filter(|c| *c != '\0')
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
            .chars()
            .take(128)
            .collect()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_with_yaml() {
        let content = "---\ntitle: Hello\ntags: [rust, web]\n---\n\nSome body text";
        let (fm, body) = GraphExtractor::parse_frontmatter(content);
        assert_eq!(body.trim(), "Some body text");
        let tags = fm.get("tags").unwrap();
        assert!(tags.is_sequence());
    }

    #[test]
    fn test_parse_frontmatter_no_frontmatter() {
        let content = "Just some text without frontmatter";
        let (fm, body) = GraphExtractor::parse_frontmatter(content);
        assert_eq!(body, content);
        assert!(fm.is_null());
    }

    #[test]
    fn test_parse_markdown_links() {
        let content = "Check out [Rust](https://rust-lang.org) and [this doc](/notes/something).";
        let links = GraphExtractor::parse_markdown_links(content);
        assert_eq!(links.len(), 2);
        assert_eq!(
            links[0],
            ("Rust".to_string(), "https://rust-lang.org".to_string())
        );
        assert_eq!(
            links[1],
            ("this doc".to_string(), "/notes/something".to_string())
        );
    }

    #[test]
    fn test_parse_markdown_headings() {
        let content = "# Title\n\n## Section A\n\n### Subsection\n\n#### Too Deep\n\n## Section B";
        let headings = GraphExtractor::parse_markdown_headings(content, 2, 3);
        assert_eq!(headings.len(), 3);
        assert_eq!(headings[0], (2, "Section A".to_string()));
        assert_eq!(headings[1], (3, "Subsection".to_string()));
        assert_eq!(headings[2], (2, "Section B".to_string()));
    }

    #[test]
    fn test_extract_tags_from_frontmatter() {
        let yaml: serde_yaml::Value = serde_yaml::from_str("tags: [rust, web, api]").unwrap();
        let tags = GraphExtractor::extract_tags_from_frontmatter(&yaml);
        assert_eq!(tags, vec!["rust", "web", "api"]);
    }

    #[test]
    fn test_extract_tags_from_frontmatter_empty() {
        let yaml: serde_yaml::Value = serde_yaml::from_str("title: no tags").unwrap();
        let tags = GraphExtractor::extract_tags_from_frontmatter(&yaml);
        assert!(tags.is_empty());
    }

    #[test]
    fn test_slugify() {
        assert_eq!(GraphExtractor::slugify("Hello World"), "hello-world");
        assert_eq!(
            GraphExtractor::slugify("Rust & WebAssembly"),
            "rust-webassembly"
        );
        assert_eq!(
            GraphExtractor::slugify("  multiple   spaces  "),
            "multiple-spaces"
        );
    }
}
