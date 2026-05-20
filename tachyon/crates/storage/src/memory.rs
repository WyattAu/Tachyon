// In-Memory Document Store
//
// HashMap-based implementation for testing.
// Not intended for production use.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tachyon_core::id::{generate_document_id, DocumentId};
use tachyon_core::types::document::{Document, DocumentContent, DocumentMetadata, DocumentStatus};
use tachyon_core::types::storage::{
    DocumentListSummary, DocumentStore, ListParams, ListResult, SortDirection, SortField,
    StorageError, StorageResult,
};

/// In-memory document store backed by a HashMap.
///
/// Thread-safe via `RwLock`. All documents live in memory and
/// are lost when the store is dropped. Intended for testing only.
#[derive(Debug, Clone)]
pub struct MemoryStore {
    documents: Arc<RwLock<HashMap<String, Document>>>,
}

impl MemoryStore {
    /// Create a new empty in-memory store.
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

// Helper: extract text for search matching
fn text_matches(query: &str, doc: &Document) -> bool {
    if query.is_empty() {
        return true;
    }
    let q = query.to_lowercase();
    let title_match = doc.metadata.title.to_lowercase().contains(&q);
    let desc_match = doc
        .metadata
        .description
        .as_ref()
        .map(|d| d.to_lowercase().contains(&q))
        .unwrap_or(false);
    let content_match = doc
        .content
        .as_text()
        .map(|t| t.to_lowercase().contains(&q))
        .unwrap_or(false);
    title_match || desc_match || content_match
}

fn sort_documents(docs: &mut [Document], sort_by: SortField, sort_dir: SortDirection) {
    match (sort_by, sort_dir) {
        (SortField::UpdatedAt, SortDirection::Desc) => {
            docs.sort_by_key(|b| std::cmp::Reverse(b.metadata.updated_at));
        }
        (SortField::UpdatedAt, SortDirection::Asc) => {
            docs.sort_by_key(|a| a.metadata.updated_at);
        }
        (SortField::CreatedAt, SortDirection::Desc) => {
            docs.sort_by_key(|b| std::cmp::Reverse(b.metadata.created_at));
        }
        (SortField::CreatedAt, SortDirection::Asc) => {
            docs.sort_by_key(|a| a.metadata.created_at);
        }
        (SortField::Title, SortDirection::Asc) => {
            docs.sort_by(|a, b| a.metadata.title.cmp(&b.metadata.title));
        }
        (SortField::Title, SortDirection::Desc) => {
            docs.sort_by(|a, b| b.metadata.title.cmp(&a.metadata.title));
        }
    }
}

impl DocumentStore for MemoryStore {
    fn create_document<'a>(
        &'a self,
        metadata: DocumentMetadata,
        content: DocumentContent,
    ) -> Pin<Box<dyn Future<Output = StorageResult<Document>> + Send + 'a>> {
        Box::pin(async move {
            let id = generate_document_id();
            let mut doc = Document::new(id, metadata.title.clone(), metadata.author_id, content);

            // Preserve metadata fields that Document::new() doesn't set from the input
            doc.metadata.slug = metadata.slug;
            doc.metadata.description = metadata.description;
            doc.metadata.tags = metadata.tags;
            doc.metadata.frontmatter = metadata.frontmatter;
            doc.metadata.published_at = metadata.published_at;

            // Check slug uniqueness
            let docs = self.documents.read();
            let slug = doc.metadata.slug.as_deref().unwrap_or(&doc.metadata.title);
            for existing in docs.values() {
                let existing_slug = existing
                    .metadata
                    .slug
                    .as_deref()
                    .unwrap_or(&existing.metadata.title);
                if existing_slug == slug {
                    return Err(StorageError::ConstraintViolation {
                        field: "slug".to_string(),
                        value: slug.to_string(),
                    });
                }
            }
            drop(docs);

            let mut docs = self.documents.write();
            docs.insert(id.as_str(), doc);
            docs.get(&id.as_str())
                .cloned()
                .ok_or_else(|| StorageError::Internal {
                    message: "Document lost after insert".to_string(),
                })
        })
    }

    fn get_document<'a>(
        &'a self,
        id: &'a DocumentId,
    ) -> Pin<Box<dyn Future<Output = StorageResult<Document>> + Send + 'a>> {
        Box::pin(async move {
            let docs = self.documents.read();
            docs.get(&id.as_str())
                .cloned()
                .ok_or_else(|| StorageError::NotFound {
                    id: id.as_str().to_string(),
                })
        })
    }

    fn update_document_content<'a>(
        &'a self,
        id: &'a DocumentId,
        content: DocumentContent,
        _expected_version: Option<u64>,
    ) -> Pin<Box<dyn Future<Output = StorageResult<Document>> + Send + 'a>> {
        Box::pin(async move {
            let mut docs = self.documents.write();
            let doc = docs
                .get_mut(&id.as_str())
                .ok_or_else(|| StorageError::NotFound {
                    id: id.as_str().to_string(),
                })?;
            doc.update_content(content);
            Ok(doc.clone())
        })
    }

    fn update_document_metadata<'a>(
        &'a self,
        id: &'a DocumentId,
        metadata: DocumentMetadata,
        _expected_version: Option<u64>,
    ) -> Pin<Box<dyn Future<Output = StorageResult<Document>> + Send + 'a>> {
        Box::pin(async move {
            let mut docs = self.documents.write();
            let doc = docs
                .get_mut(&id.as_str())
                .ok_or_else(|| StorageError::NotFound {
                    id: id.as_str().to_string(),
                })?;
            doc.metadata = metadata;
            doc.metadata.touch();
            Ok(doc.clone())
        })
    }

    fn delete_document<'a>(
        &'a self,
        id: &'a DocumentId,
    ) -> Pin<Box<dyn Future<Output = StorageResult<()>> + Send + 'a>> {
        Box::pin(async move {
            let mut docs = self.documents.write();
            if let Some(doc) = docs.get_mut(&id.as_str()) {
                doc.status = DocumentStatus::Deleted;
                doc.metadata.touch();
            }
            Ok(())
        })
    }

    fn permanently_delete_document<'a>(
        &'a self,
        id: &'a DocumentId,
    ) -> Pin<Box<dyn Future<Output = StorageResult<()>> + Send + 'a>> {
        Box::pin(async move {
            let mut docs = self.documents.write();
            docs.remove(&id.as_str());
            Ok(())
        })
    }

    fn list_documents<'a>(
        &'a self,
        params: ListParams,
    ) -> Pin<Box<dyn Future<Output = StorageResult<ListResult>> + Send + 'a>> {
        Box::pin(async move {
            let docs = self.documents.read();
            let mut items: Vec<Document> = docs
                .values()
                .filter(|doc| doc.status != DocumentStatus::Deleted)
                .cloned()
                .collect();

            // Filter by author
            if let Some(ref author_id) = params.author_id {
                items.retain(|d| d.metadata.author_id == *author_id);
            }

            // Filter by status
            if let Some(ref status) = params.status {
                items.retain(|d| d.status == *status);
            }

            // Filter by tags (any match)
            if !params.tags.is_empty() {
                items.retain(|d| params.tags.iter().any(|t| d.metadata.tags.contains(t)));
            }

            // Filter by search query
            if let Some(ref query) = params.query {
                items.retain(|d| text_matches(query, d));
            }

            let total = items.len();
            sort_documents(&mut items, params.sort_by, params.sort_dir);

            // Paginate
            let start = (params.page.saturating_sub(1)) * params.page_size;
            let end = (start + params.page_size).min(total);
            let page_items: Vec<Document> =
                items.into_iter().skip(start).take(end - start).collect();

            Ok(ListResult {
                total,
                items: page_items,
                page: params.page,
                page_size: params.page_size,
            })
        })
    }

    fn search_documents<'a>(
        &'a self,
        query: &'a str,
        page: usize,
        page_size: usize,
    ) -> Pin<Box<dyn Future<Output = StorageResult<ListResult>> + Send + 'a>> {
        Box::pin(async move {
            let params = ListParams {
                query: Some(query.to_string()),
                page,
                page_size,
                ..ListParams::default()
            };
            self.list_documents(params).await
        })
    }

    fn get_list_summary<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = StorageResult<DocumentListSummary>> + Send + 'a>> {
        Box::pin(async move {
            let docs = self.documents.read();
            let active: Vec<&Document> = docs
                .values()
                .filter(|d| d.status != DocumentStatus::Deleted)
                .collect();

            let mut all_tags = std::collections::HashSet::new();
            let mut total_words = 0usize;

            for doc in &active {
                all_tags.extend(doc.metadata.tags.iter().cloned());
                total_words += doc.stats.word_count;
            }

            Ok(DocumentListSummary {
                total_documents: active.len(),
                draft_count: active
                    .iter()
                    .filter(|d| d.status == DocumentStatus::Draft)
                    .count(),
                published_count: active
                    .iter()
                    .filter(|d| d.status == DocumentStatus::Published)
                    .count(),
                archived_count: active
                    .iter()
                    .filter(|d| d.status == DocumentStatus::Archived)
                    .count(),
                total_word_count: total_words,
                total_tags: all_tags.len(),
            })
        })
    }

    fn get_all_tags<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = StorageResult<Vec<String>>> + Send + 'a>> {
        Box::pin(async move {
            let docs = self.documents.read();
            let mut tag_counts: HashMap<String, usize> = HashMap::new();

            for doc in docs.values() {
                if doc.status == DocumentStatus::Deleted {
                    continue;
                }
                for tag in &doc.metadata.tags {
                    *tag_counts.entry(tag.clone()).or_insert(0) += 1;
                }
            }

            let mut tags: Vec<String> = tag_counts.into_keys().collect();
            tags.sort();
            Ok(tags)
        })
    }

    fn get_documents_by_tag<'a>(
        &'a self,
        tag: &'a str,
        page: usize,
        page_size: usize,
    ) -> Pin<Box<dyn Future<Output = StorageResult<ListResult>> + Send + 'a>> {
        Box::pin(async move {
            let params = ListParams {
                tags: vec![tag.to_string()],
                page,
                page_size,
                ..ListParams::default()
            };
            self.list_documents(params).await
        })
    }

    fn is_available<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = StorageResult<bool>> + Send + 'a>> {
        Box::pin(async move { Ok(true) })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tachyon_core::id::generate_user_id;

    fn make_store() -> MemoryStore {
        MemoryStore::new()
    }

    fn make_user_id() -> tachyon_core::id::UserId {
        generate_user_id()
    }

    #[tokio::test]
    async fn test_crud() {
        let store = make_store();
        let user_id = make_user_id();

        // Create
        let metadata = DocumentMetadata::new("Test".to_string(), user_id);
        let content = DocumentContent::markdown("# Hello".to_string());
        let doc = store.create_document(metadata, content).await.unwrap();
        assert_eq!(doc.metadata.title, "Test");

        // Read
        let fetched = store.get_document(&doc.id).await.unwrap();
        assert_eq!(fetched.id, doc.id);

        // Update content
        let updated = store
            .update_document_content(
                &doc.id,
                DocumentContent::markdown("# Updated".to_string()),
                None,
            )
            .await
            .unwrap();
        assert_eq!(updated.content.as_text(), Some("# Updated"));

        // Update metadata
        let mut meta = updated.metadata.clone();
        meta.description = Some("desc".to_string());
        let updated = store
            .update_document_metadata(&doc.id, meta, None)
            .await
            .unwrap();
        assert_eq!(updated.metadata.description, Some("desc".to_string()));

        // Delete (soft)
        store.delete_document(&doc.id).await.unwrap();
        let list = store.list_documents(ListParams::default()).await.unwrap();
        assert_eq!(list.total, 0);

        // Permanent delete
        store.permanently_delete_document(&doc.id).await.unwrap();
        let result = store.get_document(&doc.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_search() {
        let store = make_store();
        let user_id = make_user_id();

        let m1 = DocumentMetadata::new("Rust Book".to_string(), user_id);
        store
            .create_document(
                m1,
                DocumentContent::markdown("Rust programming language".to_string()),
            )
            .await
            .unwrap();

        let m2 = DocumentMetadata::new("Python Book".to_string(), user_id);
        store
            .create_document(
                m2,
                DocumentContent::markdown("Python scripting".to_string()),
            )
            .await
            .unwrap();

        let results = store.search_documents("Rust", 1, 10).await.unwrap();
        assert_eq!(results.total, 1);
    }

    #[tokio::test]
    async fn test_tags() {
        let store = make_store();
        let user_id = make_user_id();

        let mut m1 = DocumentMetadata::new("D1".to_string(), user_id);
        m1.tags = vec!["a".to_string(), "b".to_string()];
        store
            .create_document(m1, DocumentContent::markdown("c".to_string()))
            .await
            .unwrap();

        let mut m2 = DocumentMetadata::new("D2".to_string(), user_id);
        m2.tags = vec!["b".to_string(), "c".to_string()];
        store
            .create_document(m2, DocumentContent::markdown("c".to_string()))
            .await
            .unwrap();

        let tags = store.get_all_tags().await.unwrap();
        assert_eq!(tags.len(), 3); // a, b, c

        let by_tag = store.get_documents_by_tag("b", 1, 10).await.unwrap();
        assert_eq!(by_tag.total, 2);
    }

    #[tokio::test]
    async fn test_slug_uniqueness() {
        let store = make_store();
        let user_id = make_user_id();

        let m1 = DocumentMetadata::new("Same Title".to_string(), user_id);
        store
            .create_document(m1, DocumentContent::markdown("c1".to_string()))
            .await
            .unwrap();

        let m2 = DocumentMetadata::new("Same Title".to_string(), user_id);
        let result = store
            .create_document(m2, DocumentContent::markdown("c2".to_string()))
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            StorageError::ConstraintViolation { field, .. } => assert_eq!(field, "slug"),
            other => panic!("Expected ConstraintViolation, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_is_available() {
        let store = make_store();
        assert!(store.is_available().await.unwrap());
    }

    #[tokio::test]
    async fn test_list_summary() {
        let store = make_store();
        let user_id = make_user_id();

        let m = DocumentMetadata::new("S".to_string(), user_id);
        store
            .create_document(m, DocumentContent::markdown("one two three".to_string()))
            .await
            .unwrap();

        let summary = store.get_list_summary().await.unwrap();
        assert_eq!(summary.total_documents, 1);
        assert_eq!(summary.total_word_count, 3);
    }

    #[tokio::test]
    async fn test_pagination() {
        let store = make_store();
        let user_id = make_user_id();

        for i in 0..5 {
            let m = DocumentMetadata::new(format!("D{}", i), user_id);
            store
                .create_document(m, DocumentContent::markdown("c".to_string()))
                .await
                .unwrap();
        }

        let page1 = store
            .list_documents(ListParams {
                page: 1,
                page_size: 2,
                ..ListParams::default()
            })
            .await
            .unwrap();
        assert_eq!(page1.total, 5);
        assert_eq!(page1.items.len(), 2);
    }
}
