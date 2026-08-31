//! Citation graph queries and metrics.
//!
//! Tracks academic-style citations between documents and computes
//! impact metrics like citation count, most-referenced documents.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgRow, Row};

use crate::schema::DatabasePool;

/// A citation edge between two documents.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Citation {
    pub source_id: String,
    pub target_id: String,
    pub context: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Citation metrics for a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationMetrics {
    pub document_id: String,
    pub citation_count: i64,
    pub reference_count: i64,
    pub unique_citers: i64,
}

/// Aggregated citation stats across the corpus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationStats {
    pub total_citations: i64,
    pub total_documents_cited: i64,
    pub average_citations_per_document: f64,
    pub most_cited: Vec<(String, String, i64)>,
}

/// Add a citation edge.
pub async fn add_citation(
    pool: &DatabasePool,
    source_id: &str,
    target_id: &str,
    context: Option<&str>,
) -> Result<(), sqlx::Error> {
    let mut conn = pool.acquire().await?;
    sqlx::query(
        r#"INSERT INTO document_citations (source_id, target_id, context)
           VALUES ($1, $2, $3)
           ON CONFLICT (source_id, target_id) DO NOTHING"#,
    )
    .bind(source_id)
    .bind(target_id)
    .bind(context)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Remove a citation edge.
pub async fn remove_citation(
    pool: &DatabasePool,
    source_id: &str,
    target_id: &str,
) -> Result<(), sqlx::Error> {
    let mut conn = pool.acquire().await?;
    sqlx::query("DELETE FROM document_citations WHERE source_id = $1 AND target_id = $2")
        .bind(source_id)
        .bind(target_id)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Get citations for a document (documents this document cites).
pub async fn get_references(
    pool: &DatabasePool,
    document_id: &str,
) -> Result<Vec<Citation>, sqlx::Error> {
    let mut conn = pool.acquire().await?;
    let rows = sqlx::query_as::<_, Citation>(
        r#"SELECT dc.source_id, dc.target_id, dc.context, dc.created_at
           FROM document_citations dc
           LEFT JOIN documents d ON dc.target_id = d.id
           WHERE dc.source_id = $1
           ORDER BY dc.created_at DESC"#,
    )
    .bind(document_id)
    .fetch_all(&mut *conn)
    .await?;
    Ok(rows)
}

/// Get citations of a document (documents that cite this document).
pub async fn get_citations(
    pool: &DatabasePool,
    document_id: &str,
) -> Result<Vec<Citation>, sqlx::Error> {
    let mut conn = pool.acquire().await?;
    let rows = sqlx::query_as::<_, Citation>(
        r#"SELECT dc.source_id, dc.target_id, dc.context, dc.created_at
           FROM document_citations dc
           WHERE dc.target_id = $1
           ORDER BY dc.created_at DESC"#,
    )
    .bind(document_id)
    .fetch_all(&mut *conn)
    .await?;
    Ok(rows)
}

/// Get citation metrics for a specific document.
pub async fn get_document_metrics(
    pool: &DatabasePool,
    document_id: &str,
) -> Result<CitationMetrics, sqlx::Error> {
    let mut conn = pool.acquire().await?;
    let row: PgRow = sqlx::query(
        r#"SELECT
             $1::text as document_id,
             (SELECT COUNT(*) FROM document_citations WHERE target_id = $1) as citation_count,
             (SELECT COUNT(*) FROM document_citations WHERE source_id = $1) as reference_count,
             (SELECT COUNT(DISTINCT source_id) FROM document_citations WHERE target_id = $1) as unique_citers"#,
    )
    .bind(document_id)
    .fetch_one(&mut *conn)
    .await?;

    Ok(CitationMetrics {
        document_id: row.get("document_id"),
        citation_count: row.get("citation_count"),
        reference_count: row.get("reference_count"),
        unique_citers: row.get("unique_citers"),
    })
}

/// Get corpus-wide citation statistics.
pub async fn get_corpus_stats(pool: &DatabasePool) -> Result<CitationStats, sqlx::Error> {
    let mut conn = pool.acquire().await?;
    let row: PgRow = sqlx::query(
        r#"SELECT
             COUNT(*) as total_citations,
             COUNT(DISTINCT target_id) as total_documents_cited"#,
    )
    .fetch_one(&mut *conn)
    .await?;

    let total_citations: i64 = row.get("total_citations");
    let total_documents_cited: i64 = row.get("total_documents_cited");
    let total_docs: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM documents WHERE deleted_at IS NULL")
            .fetch_one(&mut *conn)
            .await?;
    let avg = if total_docs > 0 {
        total_citations as f64 / total_docs as f64
    } else {
        0.0
    };

    let most_cited_rows: Vec<PgRow> = sqlx::query(
        r#"SELECT dc.target_id, COALESCE(d.title, 'Untitled') as title, COUNT(*) as count
           FROM document_citations dc
           LEFT JOIN documents d ON dc.target_id = d.id
           GROUP BY dc.target_id, d.title
           ORDER BY count DESC
           LIMIT 10"#,
    )
    .fetch_all(&mut *conn)
    .await?;

    let most_cited = most_cited_rows
        .iter()
        .map(|r| (r.get("target_id"), r.get("title"), r.get::<_, i64>("count")))
        .collect();

    Ok(CitationStats {
        total_citations,
        total_documents_cited,
        average_citations_per_document: avg,
        most_cited,
    })
}

#[cfg(test)]
mod tests {}
