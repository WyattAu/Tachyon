//! GraphQL type definitions for Tachyon.

use async_graphql::{Context, Object, Result, SimpleObject, ID};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, SimpleObject)]
pub struct Document {
    pub id: ID,
    pub title: String,
    pub content: String,
    pub slug: Option<String>,
    pub repository_id: Option<ID>,
    pub space_id: Option<ID>,
    pub author_id: ID,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
    pub is_published: bool,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct User {
    pub id: ID,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct Space {
    pub id: ID,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<ID>,
    pub owner_id: ID,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub document_count: i64,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct SearchResult {
    pub id: ID,
    pub title: String,
    pub snippet: String,
    pub score: f64,
    pub tags: Vec<String>,
    pub space_id: Option<ID>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct SearchResults {
    pub results: Vec<SearchResult>,
    pub total: u64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct Team {
    pub id: ID,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: ID,
    pub member_count: i64,
    pub created_at: DateTime<Utc>,
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn document(&self, _ctx: &Context<'_>, _id: ID) -> Result<Option<Document>> {
        Err(async_graphql::Error::new(
            "Not yet implemented: requires database connection",
        ))
    }

    async fn user(&self, _ctx: &Context<'_>, _id: ID) -> Result<Option<User>> {
        Err(async_graphql::Error::new(
            "Not yet implemented: requires database connection",
        ))
    }

    async fn me(&self, _ctx: &Context<'_>) -> Result<Option<User>> {
        Err(async_graphql::Error::new(
            "Not yet implemented: requires database connection",
        ))
    }

    async fn search(
        &self,
        _ctx: &Context<'_>,
        _query: String,
        #[graphql(default = 1)] _page: i32,
        #[graphql(default = 20)] _per_page: i32,
        _tags: Option<Vec<String>>,
    ) -> Result<SearchResults> {
        Err(async_graphql::Error::new(
            "Not yet implemented: requires database connection",
        ))
    }

    async fn spaces(
        &self,
        _ctx: &Context<'_>,
        #[graphql(default = 1)] _page: i32,
        #[graphql(default = 20)] _per_page: i32,
    ) -> Result<Vec<Space>> {
        Err(async_graphql::Error::new(
            "Not yet implemented: requires database connection",
        ))
    }

    async fn teams(&self, _ctx: &Context<'_>) -> Result<Vec<Team>> {
        Err(async_graphql::Error::new(
            "Not yet implemented: requires database connection",
        ))
    }

    async fn health(&self) -> &str {
        "ok"
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_document(
        &self,
        _ctx: &Context<'_>,
        _title: String,
        _content: String,
        _space_id: Option<ID>,
        _tags: Option<Vec<String>>,
    ) -> Result<Document> {
        Err(async_graphql::Error::new(
            "Not yet implemented: requires database connection",
        ))
    }

    async fn update_document(
        &self,
        _ctx: &Context<'_>,
        _id: ID,
        _title: Option<String>,
        _content: Option<String>,
        _tags: Option<Vec<String>>,
    ) -> Result<Document> {
        Err(async_graphql::Error::new(
            "Not yet implemented: requires database connection",
        ))
    }

    async fn delete_document(&self, _ctx: &Context<'_>, _id: ID) -> Result<bool> {
        Err(async_graphql::Error::new(
            "Not yet implemented: requires database connection",
        ))
    }

    async fn create_space(
        &self,
        _ctx: &Context<'_>,
        _name: String,
        _description: Option<String>,
        _parent_id: Option<ID>,
    ) -> Result<Space> {
        Err(async_graphql::Error::new(
            "Not yet implemented: requires database connection",
        ))
    }

    async fn update_profile(
        &self,
        _ctx: &Context<'_>,
        _display_name: Option<String>,
        _avatar_url: Option<String>,
    ) -> Result<User> {
        Err(async_graphql::Error::new(
            "Not yet implemented: requires database connection",
        ))
    }
}
