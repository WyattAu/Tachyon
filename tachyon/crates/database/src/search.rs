// Full-Text Search
// PostgreSQL tsvector-based search with faceted filtering

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use crate::types::DocumentMetadata;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query, Row};
use tracing::{debug, info, instrument};

#[cfg(feature = "staging")]
const DOCUMENT_SELECT_SQL: &str = r#"
    SELECT 
        id::text as id,
        title,
        slug,
        author_id::text as author_id,
        description,
        tags::text as tags,
        frontmatter::text as frontmatter,
        project_id::text as project_id,
        visibility,
        status,
        content_type,
        word_count,
        character_count,
        read_count,
        edit_count,
        content,
        html,
        created_at,
        updated_at,
        published_at
    FROM documents
"#;

fn row_to_document_metadata(row: sqlx::postgres::PgRow) -> DatabaseResult<DocumentMetadata> {
    Ok(DocumentMetadata {
        id: row.get("id"),
        title: row.get("title"),
        slug: row.get("slug"),
        author_id: row.get("author_id"),
        description: row.get("description"),
        tags: row.get("tags"),
        frontmatter: row.try_get("frontmatter").unwrap_or(None),
        project_id: row.try_get("project_id").unwrap_or(None),
        visibility: row.get("visibility"),
        status: row.get("status"),
        content_type: row.try_get("content_type").unwrap_or_default(),
        word_count: row.try_get("word_count").unwrap_or(0),
        character_count: row.try_get("character_count").unwrap_or(0),
        read_count: row.try_get("read_count").unwrap_or(0),
        edit_count: row.try_get("edit_count").unwrap_or(0),
        content: row.try_get("content").unwrap_or(None),
        html: row.try_get("html").unwrap_or(None),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        published_at: row.try_get("published_at").unwrap_or(None),
        content_hash: row.try_get("content_hash").unwrap_or(None),
        conflict_detected: row.try_get("conflict_detected").unwrap_or(None),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub content_type: Option<String>,
    pub status: Option<String>,
    pub visibility: Option<String>,
    pub project_id: Option<String>,
    pub author_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self {
            content_type: None,
            status: None,
            visibility: None,
            project_id: None,
            author_id: None,
            tags: None,
            date_from: None,
            date_to: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document: DocumentMetadata,
    pub rank: f64,
    pub headline: Option<String>,
    pub highlights: Vec<SearchHighlight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHighlight {
    pub field: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub facets: SearchFacets,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFacets {
    pub content_types: Vec<FacetCount>,
    pub statuses: Vec<FacetCount>,
    pub visibilities: Vec<FacetCount>,
    pub tags: Vec<FacetCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetCount {
    pub value: String,
    pub count: i64,
}

#[derive(Clone)]
pub struct SearchRepository {
    pool: DatabasePool,
}

impl SearchRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn search(
        &self,
        query_text: &str,
        filters: &SearchFilters,
        page: i64,
        page_size: i64,
    ) -> DatabaseResult<SearchResponse> {
        let offset = (page - 1) * page_size;
        let mut conditions: Vec<String> = vec![];
        let mut param_count = 2;

        if let Some(ref _content_type) = filters.content_type {
            conditions.push(format!("content_type = ${}", param_count));
            param_count += 1;
        }
        if let Some(ref _status) = filters.status {
            conditions.push(format!("status = ${}", param_count));
            param_count += 1;
        }
        if let Some(ref _visibility) = filters.visibility {
            conditions.push(format!("visibility = ${}", param_count));
            param_count += 1;
        }
        if let Some(ref _project_id) = filters.project_id {
            conditions.push(format!("project_id = ${}::uuid", param_count));
            param_count += 1;
        }
        if let Some(ref _author_id) = filters.author_id {
            conditions.push(format!("author_id = ${}::uuid", param_count));
            param_count += 1;
        }
        if let Some(ref tags) = filters.tags {
            for _ in tags {
                conditions.push(format!("tags::jsonb ? ${}", param_count));
                param_count += 1;
            }
        }
        if let Some(_date_from) = filters.date_from {
            conditions.push(format!("created_at >= ${}", param_count));
            param_count += 1;
        }
        if let Some(_date_to) = filters.date_to {
            conditions.push(format!("created_at <= ${}", param_count));
            param_count += 1;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("AND {}", conditions.join(" AND "))
        };

        let search_sql = format!(
            r#"
            SELECT 
                d.id::text as id,
                d.title,
                d.slug,
                d.author_id::text as author_id,
                d.description,
                d.tags::text as tags,
                d.frontmatter::text as frontmatter,
                d.project_id::text as project_id,
                d.visibility,
                d.status,
                d.content_type,
                d.word_count,
                d.character_count,
                d.read_count,
                d.edit_count,
                d.created_at,
                d.updated_at,
                d.published_at,
                ts_rank(d.search_vector, websearch_to_tsquery('english', $1))::float8 as rank,
                ts_headline('english', d.title || ' ' || COALESCE(d.description, ''), websearch_to_tsquery('english', $1), 'StartSel=<mark>, StopSel=</mark>, MaxWords=35, MinWords=15') as headline
            FROM documents d
            WHERE d.search_vector @@ websearch_to_tsquery('english', $1)
            {}
            ORDER BY rank DESC, d.updated_at DESC
            LIMIT ${} OFFSET ${}
            "#,
            where_clause, param_count, param_count + 1
        );

        let count_sql = format!(
            r#"
            SELECT COUNT(*) as total
            FROM documents d
            WHERE d.search_vector @@ websearch_to_tsquery('english', $1)
            {}
            "#,
            where_clause
        );

        let mut conn = self.pool.acquire().await?;

        let mut query_builder = sqlx::query(&search_sql).bind(query_text);

        if let Some(ref content_type) = filters.content_type {
            query_builder = query_builder.bind(content_type);
        }
        if let Some(ref status) = filters.status {
            query_builder = query_builder.bind(status);
        }
        if let Some(ref visibility) = filters.visibility {
            query_builder = query_builder.bind(visibility);
        }
        if let Some(ref project_id) = filters.project_id {
            query_builder = query_builder.bind(project_id);
        }
        if let Some(ref author_id) = filters.author_id {
            query_builder = query_builder.bind(author_id);
        }
        if let Some(ref tags) = filters.tags {
            for tag in tags {
                query_builder = query_builder.bind(tag);
            }
        }
        if let Some(date_from) = filters.date_from {
            query_builder = query_builder.bind(date_from);
        }
        if let Some(date_to) = filters.date_to {
            query_builder = query_builder.bind(date_to);
        }

        query_builder = query_builder.bind(page_size).bind(offset);

        let rows = query_builder
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let mut count_builder = sqlx::query(&count_sql).bind(query_text);

        if let Some(ref content_type) = filters.content_type {
            count_builder = count_builder.bind(content_type);
        }
        if let Some(ref status) = filters.status {
            count_builder = count_builder.bind(status);
        }
        if let Some(ref visibility) = filters.visibility {
            count_builder = count_builder.bind(visibility);
        }
        if let Some(ref project_id) = filters.project_id {
            count_builder = count_builder.bind(project_id);
        }
        if let Some(ref author_id) = filters.author_id {
            count_builder = count_builder.bind(author_id);
        }
        if let Some(ref tags) = filters.tags {
            for tag in tags {
                count_builder = count_builder.bind(tag);
            }
        }
        if let Some(date_from) = filters.date_from {
            count_builder = count_builder.bind(date_from);
        }
        if let Some(date_to) = filters.date_to {
            count_builder = count_builder.bind(date_to);
        }

        let count_row = count_builder
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let total: i64 = count_row.get("total");

        let results: Vec<SearchResult> = rows
            .into_iter()
            .map(|row| {
                let rank: f64 = row.get("rank");
                let headline: Option<String> = row.get("headline");
                let doc = row_to_document_metadata(row)?;

                Ok(SearchResult {
                    document: doc,
                    rank,
                    headline,
                    highlights: vec![],
                })
            })
            .collect::<DatabaseResult<Vec<_>>>()?;

        let facets = self.get_facets(query_text, filters).await?;

        Ok(SearchResponse {
            results,
            total,
            page,
            page_size,
            facets,
        })
    }

    async fn get_facets(
        &self,
        query_text: &str,
        _filters: &SearchFilters,
    ) -> DatabaseResult<SearchFacets> {
        let mut conn = self.pool.acquire().await?;

        let base_where = "WHERE search_vector @@ websearch_to_tsquery('english', $1)";

        let content_types_sql = format!(
            "SELECT content_type as value, COUNT(*) as count FROM documents {} GROUP BY content_type ORDER BY count DESC LIMIT 10",
            base_where
        );
        let statuses_sql = format!(
            "SELECT status as value, COUNT(*) as count FROM documents {} GROUP BY status ORDER BY count DESC",
            base_where
        );
        let visibilities_sql = format!(
            "SELECT visibility as value, COUNT(*) as count FROM documents {} GROUP BY visibility ORDER BY count DESC",
            base_where
        );

        let content_types: Vec<FacetCount> = query(&content_types_sql)
            .bind(query_text)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            .into_iter()
            .map(|row| FacetCount {
                value: row.get("value"),
                count: row.get("count"),
            })
            .collect();

        let statuses: Vec<FacetCount> = query(&statuses_sql)
            .bind(query_text)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            .into_iter()
            .map(|row| FacetCount {
                value: row.get("value"),
                count: row.get("count"),
            })
            .collect();

        let visibilities: Vec<FacetCount> = query(&visibilities_sql)
            .bind(query_text)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            .into_iter()
            .map(|row| FacetCount {
                value: row.get("value"),
                count: row.get("count"),
            })
            .collect();

        let tags_sql = format!(
            "SELECT DISTINCT jsonb_array_elements_text(tags) as value, COUNT(*) as count FROM documents {} GROUP BY value ORDER BY count DESC LIMIT 20",
            base_where
        );
        let tags: Vec<FacetCount> = query(&tags_sql)
            .bind(query_text)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            .into_iter()
            .map(|row| FacetCount {
                value: row.get("value"),
                count: row.get("count"),
            })
            .collect();

        Ok(SearchFacets {
            content_types,
            statuses,
            visibilities,
            tags,
        })
    }

    #[instrument(skip(self))]
    pub async fn global_search(
        &self,
        query_text: &str,
        filters: &SearchFilters,
        page: i64,
        page_size: i64,
    ) -> DatabaseResult<GlobalSearchResponse> {
        let doc_response = self.search(query_text, filters, page, page_size).await?;

        let projects = self.search_projects(query_text, page_size).await?;

        Ok(GlobalSearchResponse {
            documents: doc_response,
            projects,
        })
    }

    async fn search_projects(
        &self,
        query_text: &str,
        limit: i64,
    ) -> DatabaseResult<Vec<ProjectSearchResult>> {
        let sql = r#"
            SELECT 
                id::text as id,
                name,
                slug,
                description,
                project_type,
                status,
                ts_rank(
                    setweight(to_tsvector('english', name), 'A') ||
                    setweight(to_tsvector('english', COALESCE(description, '')), 'B'),
                    websearch_to_tsquery('english', $1)
                )::float8 as rank
            FROM projects
            WHERE 
                setweight(to_tsvector('english', name), 'A') ||
                setweight(to_tsvector('english', COALESCE(description, '')), 'B')
                @@ websearch_to_tsquery('english', $1)
            ORDER BY rank DESC
            LIMIT $2
        "#;

        let mut conn = self.pool.acquire().await?;
        let rows = query(sql)
            .bind(query_text)
            .bind(limit)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let results: Vec<ProjectSearchResult> = rows
            .into_iter()
            .map(|row| ProjectSearchResult {
                id: row.get("id"),
                name: row.get("name"),
                slug: row.get("slug"),
                description: row.get("description"),
                project_type: row.get("project_type"),
                status: row.get("status"),
                rank: row.get("rank"),
            })
            .collect();

        Ok(results)
    }

    #[instrument(skip(self))]
    pub async fn update_search_index(
        &self,
        document_id: &str,
        title: &str,
        description: Option<&str>,
        content: &str,
        tags: &[String],
    ) -> DatabaseResult<()> {
        let tags_str = tags.join(" ");

        let sql = r#"
            UPDATE documents SET
                search_vector = 
                    setweight(to_tsvector('english', COALESCE($2, '')), 'A') ||
                    setweight(to_tsvector('english', COALESCE($3, '')), 'B') ||
                    setweight(to_tsvector('english', $4), 'C') ||
                    setweight(to_tsvector('english', $5), 'B')
            WHERE id = $1::uuid
        "#;

        let mut conn = self.pool.acquire().await?;
        query(sql)
            .bind(document_id)
            .bind(title)
            .bind(description)
            .bind(content)
            .bind(&tags_str)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        debug!("Search index updated for document: {}", document_id);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn rebuild_search_index(&self) -> DatabaseResult<u64> {
        let sql = r#"
            UPDATE documents SET
                search_vector = 
                    setweight(to_tsvector('english', COALESCE(title, '')), 'A') ||
                    setweight(to_tsvector('english', COALESCE(description, '')), 'B') ||
                    setweight(to_tsvector('english', COALESCE(content, '')), 'C') ||
                    setweight(to_tsvector('english', COALESCE(
                        (SELECT string_agg(jsonb_array_elements_text(tags), ' ') FROM documents d2 WHERE d2.id = documents.id),
                        ''
                    )), 'B')
            WHERE search_vector IS NULL OR search_vector = ''
        "#;

        let mut conn = self.pool.acquire().await?;
        let result = query(sql)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!("Rebuilt search index for {} documents", result.rows_affected());
        Ok(result.rows_affected())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSearchResult {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub project_type: String,
    pub status: String,
    pub rank: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSearchResponse {
    pub documents: SearchResponse,
    pub projects: Vec<ProjectSearchResult>,
}
