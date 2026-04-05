// Repository CRUD Operations
// Document and repository management (PostgreSQL)

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use crate::types::*;
use sqlx::{query, query_as, Row};
use tachyon_core::id::{DocumentId, RepositoryId};
use tracing::{debug, info, instrument};

/// Document SELECT statement with UUID casting for PostgreSQL compatibility
const DOCUMENT_SELECT_SQL: &str = r#"
    SELECT id::text, title, slug, author_id::text, description, tags::text as tags, frontmatter::text as frontmatter,
           repository_id::text, visibility, status, content_type,
           word_count, character_count, read_count, edit_count,
           created_at, updated_at, published_at
    FROM documents
"#;

/// Document repository for CRUD operations
#[derive(Clone)]
pub struct DocumentRepository {
    pool: DatabasePool,
}
