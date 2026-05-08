// Indexing Module
// Document indexing with Tantivy search engine

use crate::error::{SearchError, SearchResult};
use crate::types::{FieldDefinition, FieldType, IndexConfig};
#[cfg(test)]
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tantivy::{schema::*, Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument};

/// Index manager for Tantivy-based document indexing
///
/// Manages document lifecycle including indexing, deletion, and query execution.
#[derive(Clone)]
pub struct IndexManager {
    /// Tantivy index
    index: Arc<Index>,
    /// Schema for document structure
    schema: Schema,
    /// Field mappings for custom fields
    field_mappings: HashMap<String, Field>,
    /// Index configuration
    config: IndexConfig,
    /// Default field names
    default_fields: DefaultFields,
    /// Shared index writer, lazily created
    writer: Arc<Mutex<Option<IndexWriter>>>,
}

/// Default field names used in the index
#[derive(Clone)]
struct DefaultFields {
    id: String,
    title: String,
    content: String,
    author_id: String,
    repository_id: String,
    tags: String,
    created_at: String,
    updated_at: String,
}

impl DefaultFields {
    fn new() -> Self {
        Self {
            id: "id".to_string(),
            title: "title".to_string(),
            content: "content".to_string(),
            author_id: "author_id".to_string(),
            repository_id: "repository_id".to_string(),
            tags: "tags".to_string(),
            created_at: "created_at".to_string(),
            updated_at: "updated_at".to_string(),
        }
    }
}

impl IndexManager {
    /// Create a new index manager with default configuration
    ///
    /// # Arguments
    /// * `index_path` - Path to index directory
    ///
    /// # Returns
    /// Result containing initialized IndexManager or error
    ///
    /// # Errors
    /// Returns error if index creation fails
    pub async fn new(index_path: PathBuf) -> SearchResult<Self> {
        Self::with_config(index_path, IndexConfig::new("default")).await
    }

    /// Create a new index manager with custom configuration
    ///
    /// # Arguments
    /// * `index_path` - Path to index directory
    /// * `config` - Index configuration
    ///
    /// # Returns
    /// Result containing initialized IndexManager or error
    ///
    /// # Errors
    /// Returns error if index creation fails
    pub async fn with_config(index_path: PathBuf, config: IndexConfig) -> SearchResult<Self> {
        let default_fields = DefaultFields::new();

        // Create schema builder
        let mut schema_builder = Schema::builder();

        // Add default fields
        let _id_field = schema_builder.add_text_field(&default_fields.id, STRING | STORED);
        let _title_field = schema_builder.add_text_field(&default_fields.title, TEXT | STORED);
        let _content_field = schema_builder.add_text_field(&default_fields.content, TEXT);
        let _author_id_field =
            schema_builder.add_text_field(&default_fields.author_id, STRING | STORED);
        let _repository_id_field =
            schema_builder.add_text_field(&default_fields.repository_id, STRING | STORED);
        let _tags_field = schema_builder.add_text_field(&default_fields.tags, TEXT | STORED);
        let _created_at_field = schema_builder.add_date_field(&default_fields.created_at, STORED);
        let _updated_at_field = schema_builder.add_date_field(&default_fields.updated_at, STORED);

        // Add custom fields from configuration
        let mut field_mappings = HashMap::new();
        for field_def in &config.fields {
            let field = Self::add_field_to_schema(&mut schema_builder, field_def)?;
            field_mappings.insert(field_def.name.clone(), field);
        }

        // Build schema
        let schema = schema_builder.build();

        // Create or open index
        let index = Index::builder()
            .schema(schema.clone())
            .create_in_dir(&index_path)
            .map_err(|e| {
                SearchError::index(
                    "INDEX_CREATE_ERROR",
                    format!("Failed to create index: {}", e),
                )
            })?;

        Ok(Self {
            index: Arc::new(index),
            schema,
            field_mappings,
            config,
            default_fields,
            writer: Arc::new(Mutex::new(None)),
        })
    }

    /// Open an existing index at the given path
    pub async fn open(index_path: PathBuf) -> SearchResult<Self> {
        let index = Index::open_in_dir(&index_path).map_err(|e| {
            SearchError::index("INDEX_OPEN_ERROR", format!("Failed to open index: {}", e))
        })?;
        let schema = index.schema();
        let config =
            IndexConfig::new("tachyon").with_index_path(index_path.to_string_lossy().to_string());
        let default_fields = DefaultFields::new();
        let field_mappings = HashMap::new();
        Ok(Self {
            index: Arc::new(index),
            schema,
            field_mappings,
            config,
            default_fields,
            writer: Arc::new(Mutex::new(None)),
        })
    }

    /// Add a field to the schema based on field definition
    ///
    /// # Arguments
    /// * `schema_builder` - Schema builder
    /// * `field_def` - Field definition
    ///
    /// # Returns
    /// Result containing the created field or error
    fn add_field_to_schema(
        schema_builder: &mut SchemaBuilder,
        field_def: &FieldDefinition,
    ) -> SearchResult<Field> {
        let field = match &field_def.field_type {
            FieldType::Text => schema_builder.add_text_field(&field_def.name, TEXT | STORED),
            FieldType::String => schema_builder.add_text_field(&field_def.name, STRING | STORED),
            FieldType::Integer => schema_builder.add_i64_field(&field_def.name, STORED),
            FieldType::Boolean => schema_builder.add_bool_field(&field_def.name, STORED),
            FieldType::DateTime => schema_builder.add_date_field(&field_def.name, STORED),
        };
        Ok(field)
    }

    /// Execute an operation with the shared index writer, lazily creating it on first use.
    fn with_writer<R>(
        &self,
        f: impl FnOnce(&mut IndexWriter) -> SearchResult<R>,
    ) -> SearchResult<R> {
        let mut guard = self
            .writer
            .lock()
            .map_err(|_| SearchError::internal("Index writer lock poisoned"))?;
        if guard.is_none() {
            let writer = self.index.writer(50_000_000).map_err(|e| {
                SearchError::index("WRITER_ERROR", format!("Failed to create writer: {}", e))
            })?;
            *guard = Some(writer);
        }
        let writer = guard
            .as_mut()
            .ok_or_else(|| SearchError::internal("Index writer not initialized"))?;
        f(writer)
    }

    /// Index a document
    ///
    /// # Arguments
    /// * `document` - Document to index
    ///
    /// # Returns
    /// Result indicating success or error
    ///
    /// # Errors
    /// Returns error if indexing fails
    pub async fn index_document(
        &self,
        document: &crate::types::SearchDocument,
    ) -> SearchResult<()> {
        self.with_writer(|writer| {
            self.index_document_to_writer(writer, document)?;
            writer
                .commit()
                .map(|_| ())
                .map_err(|e| SearchError::index("COMMIT_ERROR", format!("Failed to commit: {}", e)))
        })
    }

    /// Add a field value to a document
    ///
    /// # Arguments
    /// * `doc` - Document to add value to
    /// * `field` - Field to add value for
    /// * `value` - Value to add
    fn add_field_value_to_doc(
        doc: &mut TantivyDocument,
        field: Field,
        value: &serde_json::Value,
    ) -> SearchResult<()> {
        match value {
            serde_json::Value::String(s) => {
                doc.add_text(field, s);
            }
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    doc.add_i64(field, i);
                } else if let Some(f) = n.as_f64() {
                    doc.add_f64(field, f);
                }
            }
            serde_json::Value::Bool(b) => {
                doc.add_bool(field, *b);
            }
            serde_json::Value::Null => {}
            _ => {
                return Err(SearchError::index(
                    "UNSUPPORTED_FIELD_TYPE",
                    format!("Unsupported value type for field: {:?}", value),
                ));
            }
        }
        Ok(())
    }

    /// Index multiple documents in bulk
    ///
    /// # Arguments
    /// * `documents` - Vector of documents to index
    ///
    /// # Returns
    /// Result containing number of indexed documents or error
    ///
    /// # Errors
    /// Returns error if bulk indexing fails
    pub async fn batch_index(
        &self,
        documents: &[crate::types::SearchDocument],
    ) -> SearchResult<usize> {
        self.with_writer(|writer| {
            for document in documents {
                self.index_document_to_writer(writer, document)?;
            }

            writer.commit().map_err(|e| {
                SearchError::index("COMMIT_ERROR", format!("Failed to commit: {}", e))
            })?;

            Ok(documents.len())
        })
    }

    /// Index a document using an existing writer
    ///
    /// # Arguments
    /// * `writer` - Index writer to use
    /// * `document` - Document to index
    fn index_document_to_writer(
        &self,
        writer: &mut IndexWriter,
        document: &crate::types::SearchDocument,
    ) -> SearchResult<()> {
        let schema = self.index.schema();
        let id_field = schema.get_field(&self.default_fields.id).map_err(|e| {
            SearchError::index("FIELD_ERROR", format!("Failed to get id field: {}", e))
        })?;
        let title_field = schema.get_field(&self.default_fields.title).map_err(|e| {
            SearchError::index("FIELD_ERROR", format!("Failed to get title field: {}", e))
        })?;
        let content_field = schema
            .get_field(&self.default_fields.content)
            .map_err(|e| {
                SearchError::index("FIELD_ERROR", format!("Failed to get content field: {}", e))
            })?;
        let author_id_field = schema
            .get_field(&self.default_fields.author_id)
            .map_err(|e| {
                SearchError::index(
                    "FIELD_ERROR",
                    format!("Failed to get author_id field: {}", e),
                )
            })?;
        let repository_id_field = schema
            .get_field(&self.default_fields.repository_id)
            .map_err(|e| {
                SearchError::index(
                    "FIELD_ERROR",
                    format!("Failed to get repository_id field: {}", e),
                )
            })?;
        let tags_field = schema.get_field(&self.default_fields.tags).map_err(|e| {
            SearchError::index("FIELD_ERROR", format!("Failed to get tags field: {}", e))
        })?;
        let created_at_field = schema
            .get_field(&self.default_fields.created_at)
            .map_err(|e| {
                SearchError::index(
                    "FIELD_ERROR",
                    format!("Failed to get created_at field: {}", e),
                )
            })?;
        let updated_at_field = schema
            .get_field(&self.default_fields.updated_at)
            .map_err(|e| {
                SearchError::index(
                    "FIELD_ERROR",
                    format!("Failed to get updated_at field: {}", e),
                )
            })?;

        let mut doc = TantivyDocument::default();

        doc.add_text(id_field, document.id.to_string());
        doc.add_text(title_field, &document.title);
        doc.add_text(content_field, &document.content);
        doc.add_text(author_id_field, document.author_id.to_string());

        if let Some(repository_id) = &document.repository_id {
            doc.add_text(repository_id_field, repository_id.to_string());
        }

        let tags_str = document.tags.join(" ");
        if !tags_str.is_empty() {
            doc.add_text(tags_field, &tags_str);
        }

        doc.add_date(
            created_at_field,
            tantivy::DateTime::from_timestamp_micros(document.created_at.timestamp_micros()),
        );
        doc.add_date(
            updated_at_field,
            tantivy::DateTime::from_timestamp_micros(document.updated_at.timestamp_micros()),
        );

        // Add custom field values
        for (field_name, field) in &self.field_mappings {
            if let Some(value) = document.custom_fields.get(field_name) {
                Self::add_field_value_to_doc(&mut doc, *field, value)?;
            }
        }

        writer.add_document(doc).map_err(|e| {
            SearchError::index(
                "ADD_DOCUMENT_ERROR",
                format!("Failed to add document: {}", e),
            )
        })?;

        Ok(())
    }

    /// Delete a document from the index
    ///
    /// # Arguments
    /// * `document_id` - ID of document to delete
    ///
    /// # Returns
    /// Result indicating success or error
    ///
    /// # Errors
    /// Returns error if deletion fails
    pub async fn delete_document(&self, document_id: &str) -> SearchResult<()> {
        self.with_writer(|writer| {
            let schema = self.index.schema();
            let id_field = schema.get_field(&self.default_fields.id).map_err(|e| {
                SearchError::index("FIELD_ERROR", format!("Failed to get id field: {}", e))
            })?;

            let term = tantivy::Term::from_field_text(id_field, document_id);
            writer.delete_term(term);

            writer
                .commit()
                .map(|_| ())
                .map_err(|e| SearchError::index("COMMIT_ERROR", format!("Failed to commit: {}", e)))
        })
    }

    /// Clear all documents from the index
    ///
    /// # Returns
    /// Result indicating success or error
    ///
    /// # Errors
    /// Returns error if clearing fails
    pub async fn clear_index(&self) -> SearchResult<()> {
        self.with_writer(|writer| {
            let _ = writer.delete_all_documents();

            writer
                .commit()
                .map(|_| ())
                .map_err(|e| SearchError::index("COMMIT_ERROR", format!("Failed to commit: {}", e)))
        })
    }

    /// Reload the index reader
    ///
    /// # Returns
    /// Result indicating success or error
    ///
    /// # Errors
    /// Returns error if reload fails
    pub async fn reload(&self) -> SearchResult<()> {
        let reader = self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .map_err(|e| {
                SearchError::index("READER_ERROR", format!("Failed to create reader: {}", e))
            })?;

        let _ = reader.searcher();

        Ok(())
    }

    /// Get the index reader
    ///
    /// # Returns
    /// Result containing the index reader or error
    ///
    /// # Errors
    /// Returns error if reader creation fails
    pub fn reader(&self) -> SearchResult<IndexReader> {
        self.index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .map_err(|e| {
                SearchError::index("READER_ERROR", format!("Failed to create reader: {}", e))
            })
    }

    /// Get the index schema
    ///
    /// # Returns
    /// Reference to the index schema
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    /// Get the index
    ///
    /// # Returns
    /// Reference to the index
    pub fn index(&self) -> &Index {
        &self.index
    }

    /// Get the index configuration
    ///
    /// # Returns
    /// Reference to the index configuration
    pub fn config(&self) -> &IndexConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tachyon_core::id::{DocumentId, RepositoryId, UserId};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_index_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().to_path_buf();

        let index_manager = IndexManager::new(index_path).await;
        assert!(index_manager.is_ok());
    }

    #[tokio::test]
    async fn test_document_indexing() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().to_path_buf();

        let index_manager = IndexManager::new(index_path).await.unwrap();

        let document = crate::types::SearchDocument {
            id: DocumentId::new(),
            title: "Test Document".to_string(),
            content: "This is a test document for indexing.".to_string(),
            author_id: UserId::new(),
            repository_id: Some(RepositoryId::new()),
            tags: vec!["test".to_string(), "document".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            custom_fields: BTreeMap::new(),
        };

        let result = index_manager.index_document(&document).await;
        assert!(result.is_ok());
    }
}
