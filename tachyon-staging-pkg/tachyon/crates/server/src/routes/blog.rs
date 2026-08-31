//! Blog API routes.
//!
//! CRUD endpoints for managing blog posts and RSS feed generation.

use crate::error::ServerError;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tracing::info;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlogPostRow {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub content: String,
    pub description: Option<String>,
    pub author: String,
    pub cover_image: Option<String>,
    pub published: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BlogPostResponse {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub content: String,
    pub description: Option<String>,
    pub author: String,
    pub tags: Vec<String>,
    pub cover_image: Option<String>,
    pub published: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateBlogPostRequest {
    pub title: String,
    pub content: String,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub cover_image: Option<String>,
    #[serde(default = "default_true")]
    pub published: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateBlogPostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub cover_image: Option<Option<String>>,
    pub published: Option<bool>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct BlogListQuery {
    pub tag: Option<String>,
    pub author: Option<String>,
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BlogListResponse {
    pub posts: Vec<BlogPostResponse>,
    pub total: i64,
    pub page: usize,
    pub per_page: usize,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct NewsletterSubscribeRequest {
    pub email: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct NewsletterSubscribeResponse {
    pub message: String,
}

// ============================================================================
// State
// ============================================================================

#[derive(Clone)]
pub struct BlogState {
    pub pool: tachyon_database::DatabasePool,
}

impl BlogState {
    pub fn new(pool: tachyon_database::DatabasePool) -> Self {
        Self { pool }
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn row_to_response(row: BlogPostRow, tags: Vec<String>) -> BlogPostResponse {
    BlogPostResponse {
        id: row.id,
        slug: row.slug,
        title: row.title,
        content: row.content,
        description: row.description,
        author: row.author,
        tags,
        cover_image: row.cover_image,
        published: row.published,
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.to_rfc3339(),
    }
}

// ============================================================================
// Handlers
// ============================================================================

/// List blog posts with optional filtering.
///
/// `GET /api/v1/blog/posts`
#[utoipa::path(
    get,
    path = "/blog/posts",
    params(
        ("tag" = Option<String>, Query, description = "Filter by tag"),
        ("author" = Option<String>, Query, description = "Filter by author"),
        ("page" = Option<usize>, Query, description = "Page number (default: 1)"),
        ("limit" = Option<usize>, Query, description = "Posts per page (default: 20)"),
    ),
    responses(
        (status = 200, description = "Blog post list", body = BlogListResponse),
    ),
    tag = "blog",
)]
pub async fn list_blog_posts(
    State(state): State<BlogState>,
    Query(params): Query<BlogListQuery>,
) -> Result<axum::Json<BlogListResponse>, ServerError> {
    info!("Listing blog posts");

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.limit.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as i64;
    let limit = per_page as i64;

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    // Count total
    let count_sql = if let Some(ref tag) = params.tag {
        sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(DISTINCT bp.id) FROM blog_posts bp
               JOIN blog_post_tags bpt ON bp.id = bpt.post_id
               WHERE bpt.tag = $1 AND bp.deleted_at IS NULL"#,
        )
        .bind(tag)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| ServerError::database(e.to_string()))?
    } else if let Some(ref author) = params.author {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM blog_posts WHERE author = $1 AND deleted_at IS NULL",
        )
        .bind(author)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| ServerError::database(e.to_string()))?
    } else {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM blog_posts WHERE deleted_at IS NULL")
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| ServerError::database(e.to_string()))?
    };

    // Fetch posts
    let rows: Vec<BlogPostRow> = if let Some(ref tag) = params.tag {
        sqlx::query_as::<_, BlogPostRow>(
            r#"SELECT DISTINCT bp.id, bp.slug, bp.title, bp.content, bp.description,
                      bp.author, bp.cover_image, bp.published, bp.created_at, bp.updated_at
               FROM blog_posts bp
               JOIN blog_post_tags bpt ON bp.id = bpt.post_id
               WHERE bpt.tag = $1 AND bp.deleted_at IS NULL
               ORDER BY bp.created_at DESC
               LIMIT $2 OFFSET $3"#,
        )
        .bind(tag)
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| ServerError::database(e.to_string()))?
    } else if let Some(ref author) = params.author {
        sqlx::query_as::<_, BlogPostRow>(
            r#"SELECT id, slug, title, content, description, author, cover_image, published,
                      created_at, updated_at
               FROM blog_posts
               WHERE author = $1 AND deleted_at IS NULL
               ORDER BY created_at DESC
               LIMIT $2 OFFSET $3"#,
        )
        .bind(author)
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| ServerError::database(e.to_string()))?
    } else {
        sqlx::query_as::<_, BlogPostRow>(
            r#"SELECT id, slug, title, content, description, author, cover_image, published,
                      created_at, updated_at
               FROM blog_posts
               WHERE deleted_at IS NULL
               ORDER BY created_at DESC
               LIMIT $1 OFFSET $2"#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| ServerError::database(e.to_string()))?
    };

    // Fetch tags for each post
    let mut posts = Vec::new();
    for row in rows {
        let tags: Vec<String> =
            sqlx::query_scalar::<_, String>("SELECT tag FROM blog_post_tags WHERE post_id = $1")
                .bind(&row.id)
                .fetch_all(&mut *conn)
                .await
                .unwrap_or_default();

        posts.push(row_to_response(row, tags));
    }

    Ok(axum::Json(BlogListResponse {
        posts,
        total: count_sql,
        page,
        per_page,
    }))
}

/// Get a single blog post by slug.
///
/// `GET /api/v1/blog/posts/:slug`
#[utoipa::path(
    get,
    path = "/blog/posts/{slug}",
    params(
        ("slug" = String, Path, description = "Blog post slug"),
    ),
    responses(
        (status = 200, description = "Blog post", body = BlogPostResponse),
        (status = 404, description = "Not found"),
    ),
    tag = "blog",
)]
pub async fn get_blog_post(
    Path(slug): Path<String>,
    State(state): State<BlogState>,
) -> Result<axum::Json<BlogPostResponse>, ServerError> {
    info!("Getting blog post: {}", slug);

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    let row: BlogPostRow = sqlx::query_as::<_, BlogPostRow>(
        r#"SELECT id, slug, title, content, description, author, cover_image, published,
                  created_at, updated_at
           FROM blog_posts
           WHERE slug = $1 AND deleted_at IS NULL"#,
    )
    .bind(&slug)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?
    .ok_or_else(|| ServerError::not_found("Blog post", &slug))?;

    let tags: Vec<String> =
        sqlx::query_scalar::<_, String>("SELECT tag FROM blog_post_tags WHERE post_id = $1")
            .bind(&row.id)
            .fetch_all(&mut *conn)
            .await
            .unwrap_or_default();

    Ok(axum::Json(row_to_response(row, tags)))
}

/// Create a new blog post.
///
/// `POST /api/v1/blog/posts`
#[utoipa::path(
    post,
    path = "/blog/posts",
    request_body = CreateBlogPostRequest,
    responses(
        (status = 201, description = "Blog post created", body = BlogPostResponse),
        (status = 400, description = "Validation error"),
    ),
    tag = "blog",
    security(("bearer_auth" = [])),
)]
pub async fn create_blog_post(
    State(state): State<BlogState>,
    axum::extract::Extension(auth): axum::extract::Extension<crate::middleware::AuthContext>,
    Json(req): Json<CreateBlogPostRequest>,
) -> Result<(StatusCode, Json<BlogPostResponse>), ServerError> {
    info!("Creating blog post: {}", req.title);

    if req.title.trim().is_empty() {
        return Err(ServerError::bad_request("Title is required"));
    }
    if req.content.trim().is_empty() {
        return Err(ServerError::bad_request("Content is required"));
    }

    let slug = tachyon_ssg::slug::slugify(&req.title);
    let id = uuid::Uuid::new_v4().to_string();
    let author = auth.user_id.clone();

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    let row: BlogPostRow = sqlx::query_as::<_, BlogPostRow>(
        r#"INSERT INTO blog_posts (id, slug, title, content, description, author, cover_image, published)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
           RETURNING id, slug, title, content, description, author, cover_image, published, created_at, updated_at"#,
    )
    .bind(&id)
    .bind(&slug)
    .bind(&req.title)
    .bind(&req.content)
    .bind(&req.description)
    .bind(&author)
    .bind(&req.cover_image)
    .bind(req.published)
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    // Insert tags
    for tag in &req.tags {
        sqlx::query(
            "INSERT INTO blog_post_tags (post_id, tag) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(&row.id)
        .bind(tag)
        .execute(&mut *conn)
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;
    }

    Ok((StatusCode::CREATED, Json(row_to_response(row, req.tags))))
}

/// Update an existing blog post.
///
/// `PUT /api/v1/blog/posts/:slug`
#[utoipa::path(
    put,
    path = "/blog/posts/{slug}",
    params(
        ("slug" = String, Path, description = "Blog post slug"),
    ),
    request_body = UpdateBlogPostRequest,
    responses(
        (status = 200, description = "Blog post updated", body = BlogPostResponse),
        (status = 404, description = "Not found"),
    ),
    tag = "blog",
    security(("bearer_auth" = [])),
)]
pub async fn update_blog_post(
    Path(slug): Path<String>,
    State(state): State<BlogState>,
    Json(req): Json<UpdateBlogPostRequest>,
) -> Result<Json<BlogPostResponse>, ServerError> {
    info!("Updating blog post: {}", slug);

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    // Check existence
    let existing: BlogPostRow = sqlx::query_as::<_, BlogPostRow>(
        "SELECT id, slug, title, content, description, author, cover_image, published, created_at, updated_at FROM blog_posts WHERE slug = $1 AND deleted_at IS NULL",
    )
    .bind(&slug)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?
    .ok_or_else(|| ServerError::not_found("Blog post", &slug))?;

    let new_title = req.title.unwrap_or(existing.title);
    let new_content = req.content.unwrap_or(existing.content);
    let new_description = req.description.or(existing.description);
    let new_cover_image = req.cover_image.unwrap_or(existing.cover_image);
    let new_published = req.published.unwrap_or(existing.published);

    let row: BlogPostRow = sqlx::query_as::<_, BlogPostRow>(
        r#"UPDATE blog_posts
           SET title = $1, content = $2, description = $3, cover_image = $4, published = $5, updated_at = NOW()
           WHERE slug = $6 AND deleted_at IS NULL
           RETURNING id, slug, title, content, description, author, cover_image, published, created_at, updated_at"#,
    )
    .bind(&new_title)
    .bind(&new_content)
    .bind(&new_description)
    .bind(&new_cover_image)
    .bind(new_published)
    .bind(&slug)
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    // Update tags if provided
    if let Some(new_tags) = req.tags {
        sqlx::query("DELETE FROM blog_post_tags WHERE post_id = $1")
            .bind(&row.id)
            .execute(&mut *conn)
            .await
            .map_err(|e| ServerError::database(e.to_string()))?;

        for tag in &new_tags {
            sqlx::query(
                "INSERT INTO blog_post_tags (post_id, tag) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(&row.id)
            .bind(tag)
            .execute(&mut *conn)
            .await
            .map_err(|e| ServerError::database(e.to_string()))?;
        }

        Ok(Json(row_to_response(row, new_tags)))
    } else {
        let tags: Vec<String> =
            sqlx::query_scalar::<_, String>("SELECT tag FROM blog_post_tags WHERE post_id = $1")
                .bind(&row.id)
                .fetch_all(&mut *conn)
                .await
                .unwrap_or_default();

        Ok(Json(row_to_response(row, tags)))
    }
}

/// Delete a blog post (soft delete).
///
/// `DELETE /api/v1/blog/posts/:slug`
#[utoipa::path(
    delete,
    path = "/blog/posts/{slug}",
    params(
        ("slug" = String, Path, description = "Blog post slug"),
    ),
    responses(
        (status = 204, description = "Blog post deleted"),
        (status = 404, description = "Not found"),
    ),
    tag = "blog",
    security(("bearer_auth" = [])),
)]
pub async fn delete_blog_post(
    Path(slug): Path<String>,
    State(state): State<BlogState>,
) -> Result<StatusCode, ServerError> {
    info!("Deleting blog post: {}", slug);

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    let result = sqlx::query(
        "UPDATE blog_posts SET deleted_at = NOW() WHERE slug = $1 AND deleted_at IS NULL",
    )
    .bind(&slug)
    .execute(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerError::not_found("Blog post", &slug));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Get RSS feed for blog posts.
///
/// `GET /api/v1/blog/feed`
pub async fn blog_feed(State(state): State<BlogState>) -> Result<impl IntoResponse, ServerError> {
    info!("Generating blog RSS feed");

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    let rows: Vec<BlogPostRow> = sqlx::query_as::<_, BlogPostRow>(
        r#"SELECT id, slug, title, content, description, author, cover_image, published,
                  created_at, updated_at
           FROM blog_posts
           WHERE deleted_at IS NULL AND published = true
           ORDER BY created_at DESC
           LIMIT 50"#,
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    let base_url =
        std::env::var("TACHYON_BASE_URL").unwrap_or_else(|_| "https://tachyon.app".to_string());

    let mut items = String::new();
    for row in &rows {
        let tags: Vec<String> =
            sqlx::query_scalar::<_, String>("SELECT tag FROM blog_post_tags WHERE post_id = $1")
                .bind(&row.id)
                .fetch_all(&mut *conn)
                .await
                .unwrap_or_default();

        let description = row.description.as_deref().unwrap_or("No description");
        let categories: String = tags
            .iter()
            .map(|t| format!("\n      <category>{}</category>", t))
            .collect();

        items.push_str(&format!(
            r#"
    <item>
      <title>{}</title>
      <link>{}/blog/{}.html</link>
      <description>{}</description>
      <pubDate>{}</pubDate>
      <guid isPermaLink="true">{}/blog/{}.html</guid>
      <author>{}</author>{}
    </item>"#,
            row.title,
            base_url,
            row.slug,
            description,
            row.created_at.to_rfc2822(),
            base_url,
            row.slug,
            row.author,
            categories,
        ));
    }

    let rss = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>Tachyon Blog</title>
    <link>{}/blog.html</link>
    <description>Latest posts from the Tachyon team</description>
    <language>en</language>
    <lastBuildDate>{}</lastBuildDate>
    <atom:link href="{}/api/v1/blog/feed" rel="self" type="application/rss+xml"/>
    <generator>Tachyon Server</generator>{}
  </channel>
</rss>"#,
        base_url,
        chrono::Utc::now().to_rfc3339(),
        base_url,
        items,
    );

    Ok((
        StatusCode::OK,
        [(
            axum::http::header::CONTENT_TYPE,
            "application/rss+xml; charset=utf-8",
        )],
        rss,
    )
        .into_response())
}

/// Subscribe to the blog newsletter.
///
/// `POST /api/v1/blog/subscribe`
pub async fn subscribe_newsletter(
    State(state): State<BlogState>,
    Json(req): Json<NewsletterSubscribeRequest>,
) -> Result<Json<NewsletterSubscribeResponse>, ServerError> {
    info!("Newsletter subscription: {}", req.email);

    if !req.email.contains('@') || !req.email.contains('.') {
        return Err(ServerError::bad_request("Invalid email address"));
    }

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| ServerError::database(e.to_string()))?;

    let result = sqlx::query(
        r#"INSERT INTO blog_subscribers (email) VALUES ($1)
           ON CONFLICT (email) DO UPDATE SET active = true, updated_at = NOW()
           RETURNING id"#,
    )
    .bind(&req.email)
    .execute(&mut *conn)
    .await
    .map_err(|e| ServerError::database(e.to_string()))?;

    if result.rows_affected() > 0 {
        Ok(Json(NewsletterSubscribeResponse {
            message: "Successfully subscribed to the newsletter".to_string(),
        }))
    } else {
        Err(ServerError::internal("Failed to subscribe"))
    }
}

// ============================================================================
// Router
// ============================================================================

pub fn create_blog_router() -> axum::Router<BlogState> {
    use axum::routing::{get, post};

    axum::Router::new()
        .route("/blog/posts", get(list_blog_posts).post(create_blog_post))
        .route(
            "/blog/posts/{slug}",
            get(get_blog_post)
                .put(update_blog_post)
                .delete(delete_blog_post),
        )
        .route("/blog/feed", get(blog_feed))
        .route("/blog/subscribe", post(subscribe_newsletter))
}

// ============================================================================
// Migration: ensure tables exist
// ============================================================================

/// SQL migration to create blog tables. Run this at startup or via migration system.
pub const BLOG_MIGRATION: &str = r#"
CREATE TABLE IF NOT EXISTS blog_posts (
    id TEXT PRIMARY KEY,
    slug TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    description TEXT,
    author TEXT NOT NULL,
    cover_image TEXT,
    published BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_blog_posts_slug ON blog_posts(slug);
CREATE INDEX IF NOT EXISTS idx_blog_posts_author ON blog_posts(author);
CREATE INDEX IF NOT EXISTS idx_blog_posts_created_at ON blog_posts(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_blog_posts_deleted_at ON blog_posts(deleted_at);

CREATE TABLE IF NOT EXISTS blog_post_tags (
    post_id TEXT NOT NULL REFERENCES blog_posts(id) ON DELETE CASCADE,
    tag TEXT NOT NULL,
    PRIMARY KEY (post_id, tag)
);

CREATE INDEX IF NOT EXISTS idx_blog_post_tags_tag ON blog_post_tags(tag);

CREATE TABLE IF NOT EXISTS blog_subscribers (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    email TEXT UNIQUE NOT NULL,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_blog_post_request_validation() {
        let req = CreateBlogPostRequest {
            title: "  ".to_string(),
            content: "content".to_string(),
            description: None,
            tags: vec![],
            cover_image: None,
            published: true,
        };
        assert!(req.title.trim().is_empty());
    }

    #[test]
    fn test_blog_post_response_serialization() {
        let resp = BlogPostResponse {
            id: "test-id".to_string(),
            slug: "test-post".to_string(),
            title: "Test Post".to_string(),
            content: "Content".to_string(),
            description: Some("Desc".to_string()),
            author: "Author".to_string(),
            tags: vec!["rust".to_string()],
            cover_image: None,
            published: true,
            created_at: "2025-01-15T10:00:00Z".to_string(),
            updated_at: "2025-01-15T10:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("Test Post"));
        assert!(json.contains("rust"));
    }

    #[test]
    fn test_blog_list_response_serialization() {
        let resp = BlogListResponse {
            posts: vec![],
            total: 0,
            page: 1,
            per_page: 20,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("total"));
        assert!(json.contains("page"));
    }
}
