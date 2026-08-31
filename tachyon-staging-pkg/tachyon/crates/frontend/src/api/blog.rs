//! Blog API client for frontend.

use crate::api::{ApiClient, ApiError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogPost {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogListResponse {
    pub posts: Vec<BlogPost>,
    pub total: i64,
    pub page: usize,
    pub per_page: usize,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct UpdateBlogPostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub cover_image: Option<Option<String>>,
    pub published: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct NewsletterSubscribeRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct NewsletterSubscribeResponse {
    pub message: String,
}

impl ApiClient {
    /// List blog posts with optional filtering and pagination.
    pub async fn list_blog_posts(
        &self,
        tag: Option<&str>,
        page: Option<usize>,
    ) -> Result<BlogListResponse, ApiError> {
        let mut url = format!("{}/blog/posts", self.base_url);
        let mut params = Vec::new();
        if let Some(t) = tag {
            params.push(format!("tag={}", t));
        }
        if let Some(p) = page {
            params.push(format!("page={}", p));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }
        self.get(&url).await
    }

    /// Get a single blog post by slug.
    pub async fn get_blog_post(&self, slug: &str) -> Result<BlogPost, ApiError> {
        let url = format!("{}/blog/posts/{}", self.base_url, slug);
        self.get(&url).await
    }

    /// Create a new blog post.
    pub async fn create_blog_post(
        &self,
        req: &CreateBlogPostRequest,
    ) -> Result<BlogPost, ApiError> {
        let url = format!("{}/blog/posts", self.base_url);
        self.post(&url, req).await
    }

    /// Update an existing blog post.
    pub async fn update_blog_post(
        &self,
        slug: &str,
        req: &UpdateBlogPostRequest,
    ) -> Result<BlogPost, ApiError> {
        let url = format!("{}/blog/posts/{}", self.base_url, slug);
        self.put(&url, req).await
    }

    /// Delete a blog post.
    pub async fn delete_blog_post(&self, slug: &str) -> Result<(), ApiError> {
        let url = format!("{}/blog/posts/{}", self.base_url, slug);
        self.delete(&url).await
    }

    /// Subscribe to the blog newsletter.
    pub async fn subscribe_newsletter(
        &self,
        email: &str,
    ) -> Result<NewsletterSubscribeResponse, ApiError> {
        let url = format!("{}/blog/subscribe", self.base_url);
        let req = NewsletterSubscribeRequest {
            email: email.to_string(),
        };
        self.post(&url, &req).await
    }
}
