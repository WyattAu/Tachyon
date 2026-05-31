//! GraphQL type definitions and resolvers for Tachyon.

use async_graphql::{Context, Object, Result, SimpleObject, ID};
use chrono::{DateTime, Utc};
use tachyon_database::{
    DatabasePool, DocumentRepository, SearchFilters, SearchRepository, SpaceRepository,
    TeamRepository, UserRepository,
};

use crate::graphql::schema::GraphqlAuthContext;

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

fn map_document_metadata_to_gql(m: tachyon_database::DocumentMetadata) -> Document {
    let tags = m.parse_tags().unwrap_or_default();
    Document {
        id: ID::from(&m.id),
        title: m.title,
        content: m.content.unwrap_or_default(),
        slug: m.slug,
        repository_id: m.project_id.as_ref().map(|s| ID::from(s.as_str())),
        space_id: None,
        author_id: ID::from(&m.author_id),
        tags,
        created_at: m.created_at,
        updated_at: m.updated_at,
        version: m.edit_count,
        is_published: m.status == "published",
    }
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

fn map_user_to_gql(u: tachyon_core::types::user::User) -> User {
    User {
        id: ID::from(u.id.to_string()),
        username: u.username,
        email: u.email.unwrap_or_default(),
        display_name: if u.display_name.is_empty() {
            None
        } else {
            Some(u.display_name)
        },
        avatar_url: None,
        role: u.permissions.role.to_string(),
        created_at: u.created_at,
        last_login: None,
        is_active: u.is_active.unwrap_or(true),
    }
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

fn map_space_to_gql(s: tachyon_database::Space, doc_count: i64) -> Space {
    Space {
        id: ID::from(&s.id),
        name: s.name,
        description: s.description,
        parent_id: s.parent_id.map(ID::from),
        owner_id: ID::from(&s.owner_id),
        is_public: s.visibility == "public",
        created_at: s.created_at,
        updated_at: s.updated_at,
        document_count: doc_count,
    }
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

fn map_team_to_gql(t: tachyon_database::Team) -> Team {
    Team {
        id: ID::from(&t.id),
        name: t.name,
        description: t.description,
        owner_id: ID::from(&t.owner_id),
        member_count: 0,
        created_at: t.created_at,
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn document(&self, ctx: &Context<'_>, id: ID) -> Result<Option<Document>> {
        let pool = ctx.data::<DatabasePool>()?;
        let repo = DocumentRepository::new(pool.clone());
        let doc_id = tachyon_core::id::DocumentId::parse_str(id.as_str())
            .map_err(|e| async_graphql::Error::new(format!("Invalid document ID: {}", e)))?;
        match repo.get_by_id(&doc_id).await {
            Ok(m) => Ok(Some(map_document_metadata_to_gql(m))),
            Err(tachyon_database::DatabaseError::NotFound { .. }) => Ok(None),
            Err(e) => Err(async_graphql::Error::new(e.to_string())),
        }
    }

    async fn user(&self, ctx: &Context<'_>, id: ID) -> Result<Option<User>> {
        let pool = ctx.data::<DatabasePool>()?;
        let repo = UserRepository::new(pool.clone());
        let user_id = tachyon_core::id::UserId::parse_str(id.as_str())
            .map_err(|e| async_graphql::Error::new(format!("Invalid user ID: {}", e)))?;
        match repo.get_by_id(&user_id).await {
            Ok(u) => Ok(Some(map_user_to_gql(u))),
            Err(tachyon_database::DatabaseError::NotFound { .. }) => Ok(None),
            Err(e) => Err(async_graphql::Error::new(e.to_string())),
        }
    }

    async fn me(&self, ctx: &Context<'_>) -> Result<Option<User>> {
        let auth = ctx.data::<GraphqlAuthContext>();
        let pool = ctx.data::<DatabasePool>()?;

        match auth {
            Ok(auth_ctx) => {
                let repo = UserRepository::new(pool.clone());
                let user_id = tachyon_core::id::UserId::parse_str(&auth_ctx.user_id)
                    .map_err(|e| async_graphql::Error::new(format!("Invalid user ID: {}", e)))?;
                match repo.get_by_id(&user_id).await {
                    Ok(u) => Ok(Some(map_user_to_gql(u))),
                    Err(tachyon_database::DatabaseError::NotFound { .. }) => Ok(None),
                    Err(e) => Err(async_graphql::Error::new(e.to_string())),
                }
            }
            Err(_) => Ok(None),
        }
    }

    async fn search(
        &self,
        ctx: &Context<'_>,
        query: String,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] per_page: i32,
        tags: Option<Vec<String>>,
    ) -> Result<SearchResults> {
        let pool = ctx.data::<DatabasePool>()?;
        let repo = SearchRepository::new(pool.clone());
        let filters = SearchFilters {
            tags,
            ..Default::default()
        };
        let response = repo
            .search(&query, &filters, page as i64, per_page as i64)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let results: Vec<SearchResult> = response
            .results
            .into_iter()
            .map(|r| {
                let doc = &r.document;
                SearchResult {
                    id: ID::from(&doc.id),
                    title: doc.title.clone(),
                    snippet: r.headline.unwrap_or_default(),
                    score: r.rank,
                    tags: doc.parse_tags().unwrap_or_default(),
                    space_id: doc.project_id.as_ref().map(|s| ID::from(s.as_str())),
                    updated_at: doc.updated_at,
                }
            })
            .collect();

        Ok(SearchResults {
            total: response.total as u64,
            page,
            per_page,
            results,
        })
    }

    async fn spaces(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] per_page: i32,
    ) -> Result<Vec<Space>> {
        let pool = ctx.data::<DatabasePool>()?;
        let repo = SpaceRepository::new(pool.clone());
        let limit = per_page as i64;
        let offset = ((page.max(1) - 1) as i64) * limit;
        let spaces = repo
            .list(None, None, None, None, Some(limit), Some(offset))
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let space_ids: Vec<String> = spaces.iter().map(|s| s.id.clone()).collect();
        let doc_counts = repo
            .count_documents_batch(&space_ids)
            .await
            .unwrap_or_default();

        Ok(spaces
            .into_iter()
            .map(|s| {
                let count = doc_counts.get(&s.id).copied().unwrap_or(0);
                map_space_to_gql(s, count)
            })
            .collect())
    }

    async fn teams(&self, ctx: &Context<'_>) -> Result<Vec<Team>> {
        let pool = ctx.data::<DatabasePool>()?;
        let repo = TeamRepository::new(pool.clone());
        let teams = repo
            .list_by_owner("")
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(teams.into_iter().map(map_team_to_gql).collect())
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
        ctx: &Context<'_>,
        title: String,
        content: String,
        space_id: Option<ID>,
        tags: Option<Vec<String>>,
    ) -> Result<Document> {
        let pool = ctx.data::<DatabasePool>()?;
        let repo = DocumentRepository::new(pool.clone());

        let doc_id = tachyon_core::generate_document_id();
        let auth: GraphqlAuthContext = ctx
            .data::<GraphqlAuthContext>()
            .map(|a| a.clone())
            .unwrap_or(GraphqlAuthContext {
                user_id: tachyon_core::generate_user_id().to_string(),
                role: "guest".to_string(),
                permissions: vec![],
                team_id: None,
            });
        let author_id = auth.user_id;
        let now = chrono::Utc::now();
        let tags_json =
            serde_json::to_string(&tags.unwrap_or_default()).unwrap_or_else(|_| "[]".to_string());

        let metadata = tachyon_database::DocumentMetadata {
            id: doc_id.to_string(),
            title: title.clone(),
            slug: None,
            author_id: author_id.to_string(),
            description: None,
            tags: tags_json,
            frontmatter: None,
            project_id: space_id.map(|id| id.to_string()),
            visibility: "private".to_string(),
            status: "draft".to_string(),
            content_type: "markdown".to_string(),
            word_count: content.split_whitespace().count() as i32,
            character_count: content.len() as i32,
            read_count: 0,
            edit_count: 1,
            content: Some(content.clone()),
            html: None,
            created_at: now,
            updated_at: now,
            published_at: None,
            content_hash: None,
            conflict_detected: None,
        };

        repo.create(metadata)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let created = repo
            .get_by_id(&doc_id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(map_document_metadata_to_gql(created))
    }

    async fn update_document(
        &self,
        ctx: &Context<'_>,
        id: ID,
        title: Option<String>,
        content: Option<String>,
        tags: Option<Vec<String>>,
    ) -> Result<Document> {
        let pool = ctx.data::<DatabasePool>()?;
        let repo = DocumentRepository::new(pool.clone());

        let doc_id = tachyon_core::id::DocumentId::parse_str(id.as_str())
            .map_err(|e| async_graphql::Error::new(format!("Invalid document ID: {}", e)))?;

        let mut metadata = repo
            .get_by_id(&doc_id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        if let Some(t) = title {
            metadata.title = t;
        }
        if let Some(c) = content {
            metadata.content = Some(c);
        }
        if let Some(t) = tags {
            metadata.tags = serde_json::to_string(&t).unwrap_or_else(|_| "[]".to_string());
        }
        metadata.updated_at = chrono::Utc::now();

        repo.update(metadata)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let updated = repo
            .get_by_id(&doc_id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(map_document_metadata_to_gql(updated))
    }

    async fn delete_document(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        let pool = ctx.data::<DatabasePool>()?;
        let repo = DocumentRepository::new(pool.clone());

        let doc_id = tachyon_core::id::DocumentId::parse_str(id.as_str())
            .map_err(|e| async_graphql::Error::new(format!("Invalid document ID: {}", e)))?;

        match repo.delete(&doc_id).await {
            Ok(()) => Ok(true),
            Err(tachyon_database::DatabaseError::NotFound { .. }) => Ok(false),
            Err(e) => Err(async_graphql::Error::new(e.to_string())),
        }
    }

    async fn create_space(
        &self,
        ctx: &Context<'_>,
        name: String,
        description: Option<String>,
        parent_id: Option<ID>,
    ) -> Result<Space> {
        let pool = ctx.data::<DatabasePool>()?;
        let repo = SpaceRepository::new(pool.clone());

        let auth: GraphqlAuthContext = ctx
            .data::<GraphqlAuthContext>()
            .map(|a| a.clone())
            .unwrap_or(GraphqlAuthContext {
                user_id: "00000000-0000-0000-0000-000000000000".to_string(),
                role: "guest".to_string(),
                permissions: vec![],
                team_id: None,
            });
        let owner_id = auth.user_id;
        let req = tachyon_database::CreateSpaceRequest {
            name,
            description,
            icon: None,
            color: None,
            parent_id: parent_id.map(|id| id.to_string()),
            visibility: None,
        };

        let space = repo
            .create(&owner_id, req)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let doc_count = repo.count_documents(&space.id).await.unwrap_or(0);
        Ok(map_space_to_gql(space, doc_count))
    }

    async fn update_profile(
        &self,
        _ctx: &Context<'_>,
        _display_name: Option<String>,
        _avatar_url: Option<String>,
    ) -> Result<User> {
        Err(async_graphql::Error::new(
            "update_profile requires authentication context",
        ))
    }
}
