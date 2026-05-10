use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use tachyon_core::Id;
use tachyon_database::{DatabasePool, DocumentMetadata, DocumentRepository, OnboardingRepository};
use tracing::info;

#[derive(Clone)]
pub struct OnboardingState {
    pub pool: DatabasePool,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct OnboardingStatusResponse {
    pub completed: bool,
    pub steps: Vec<tachyon_database::OnboardingStep>,
    pub current_step: usize,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CompleteStepRequest {
    pub step_id: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CompleteStepResponse {
    pub success: bool,
    pub step_id: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SampleContentResponse {
    pub created: usize,
    pub skipped: usize,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SuggestionsResponse {
    pub suggested_tags: Vec<String>,
    pub suggested_templates: Vec<TemplateSuggestion>,
    pub tips: Vec<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TemplateSuggestion {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

/// Get the current user's onboarding status.
///
/// `GET /api/v1/onboarding/status`
///
/// Returns which steps are completed and the current step index.
#[utoipa::path(
    get,
    path = "/onboarding/status",
    responses(
        (status = 200, description = "Onboarding status", body = OnboardingStatusResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "onboarding",
    security(("bearer_auth" = [])),
)]
pub async fn get_onboarding_status(
    State(state): State<OnboardingState>,
) -> Result<Json<OnboardingStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    let repo = OnboardingRepository::new(state.pool.clone());
    let status = repo
        .get_onboarding_status("current_user")
        .await
        .map_err(|e| {
            server_error(
                "QUERY_ERROR",
                &format!("Failed to get onboarding status: {}", e),
            )
        })?;

    Ok(Json(OnboardingStatusResponse {
        completed: status.completed,
        steps: status.steps,
        current_step: status.current_step,
    }))
}

/// Mark an onboarding step as completed.
///
/// `POST /api/v1/onboarding/complete`
#[utoipa::path(
    post,
    path = "/onboarding/complete",
    request_body(content = CompleteStepRequest, description = "Step completion request"),
    responses(
        (status = 200, description = "Step completed", body = CompleteStepResponse),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "onboarding",
    security(("bearer_auth" = [])),
)]
pub async fn complete_step(
    State(state): State<OnboardingState>,
    Json(body): Json<CompleteStepRequest>,
) -> Result<Json<CompleteStepResponse>, (StatusCode, Json<ErrorResponse>)> {
    let repo = OnboardingRepository::new(state.pool.clone());
    repo.complete_step("current_user", &body.step_id)
        .await
        .map_err(|e| {
            if format!("{}", e).contains("Validation") {
                bad_request("VALIDATION_ERROR", &format!("{}", e))
            } else {
                server_error("UPDATE_ERROR", &format!("Failed to complete step: {}", e))
            }
        })?;

    info!("Onboarding step completed: {}", body.step_id);
    Ok(Json(CompleteStepResponse {
        success: true,
        step_id: body.step_id,
    }))
}

/// Create sample documents for onboarding.
///
/// `POST /api/v1/onboarding/sample-content`
///
/// Creates up to 5 sample documents (Welcome, Getting Started, Markdown Guide,
/// Knowledge Graph, Keyboard Shortcuts) if the user has fewer than 3 documents.
#[utoipa::path(
    post,
    path = "/onboarding/sample-content",
    responses(
        (status = 200, description = "Sample content created", body = SampleContentResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "onboarding",
    security(("bearer_auth" = [])),
)]
pub async fn create_sample_content(
    State(state): State<OnboardingState>,
) -> Result<Json<SampleContentResponse>, (StatusCode, Json<ErrorResponse>)> {
    let repo = OnboardingRepository::new(state.pool.clone());
    let doc_count = repo
        .get_user_document_count("current_user")
        .await
        .map_err(|e| server_error("QUERY_ERROR", &format!("Failed to count documents: {}", e)))?;

    if doc_count >= 3 {
        return Ok(Json(SampleContentResponse {
            created: 0,
            skipped: 0,
        }));
    }

    let samples = build_sample_documents();
    let doc_repo = DocumentRepository::new(state.pool.clone());
    let mut created = 0usize;

    for sample in &samples {
        let result = doc_repo.create(sample.clone()).await;
        match result {
            Ok(()) => created += 1,
            Err(e) if format!("{}", e).contains("duplicate") => {}
            Err(e) => {
                return Err(server_error(
                    "CREATE_ERROR",
                    &format!("Failed to create sample document: {}", e),
                ))
            }
        }
    }

    info!("Created {} sample documents", created);
    Ok(Json(SampleContentResponse {
        created,
        skipped: samples.len() - created,
    }))
}

/// Get onboarding suggestions.
///
/// `GET /api/v1/onboarding/suggestions`
///
/// Returns suggested tags, templates, and tips for new users.
#[utoipa::path(
    get,
    path = "/onboarding/suggestions",
    responses(
        (status = 200, description = "Onboarding suggestions", body = SuggestionsResponse),
    ),
    tag = "onboarding",
    security(("bearer_auth" = [])),
)]
pub async fn get_suggestions(
    State(state): State<OnboardingState>,
) -> Result<Json<SuggestionsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let _ = &state;

    Ok(Json(SuggestionsResponse {
        suggested_tags: vec![
            "getting-started".to_string(),
            "documentation".to_string(),
            "notes".to_string(),
            "project".to_string(),
            "reference".to_string(),
            "meeting-notes".to_string(),
            "daily-journal".to_string(),
            "research".to_string(),
        ],
        suggested_templates: vec![
            TemplateSuggestion {
                id: "meeting-notes".to_string(),
                name: "Meeting Notes".to_string(),
                description: "Structured template for meeting notes with attendees, agenda, and action items".to_string(),
            },
            TemplateSuggestion {
                id: "project-brief".to_string(),
                name: "Project Brief".to_string(),
                description: "Comprehensive project overview with goals, timeline, and stakeholders".to_string(),
            },
            TemplateSuggestion {
                id: "daily-journal".to_string(),
                name: "Daily Journal".to_string(),
                description: "Daily reflection template with gratitude, goals, and learnings".to_string(),
            },
            TemplateSuggestion {
                id: "technical-spec".to_string(),
                name: "Technical Specification".to_string(),
                description: "Detailed technical specification with requirements, architecture, and implementation plan".to_string(),
            },
        ],
        tips: vec![
            "Use [[wiki-links]] to connect related documents and build your knowledge graph".to_string(),
            "Add tags to your documents to make them easy to find with search".to_string(),
            "Use code blocks with language identifiers for syntax highlighting".to_string(),
            "Create spaces to organize documents into logical groups".to_string(),
            "Invite team members to collaborate on documents in real-time".to_string(),
        ],
    }))
}

pub fn create_onboarding_router() -> axum::Router<OnboardingState> {
    axum::Router::new()
        .route(
            "/onboarding/status",
            axum::routing::get(get_onboarding_status),
        )
        .route("/onboarding/complete", axum::routing::post(complete_step))
        .route(
            "/onboarding/sample-content",
            axum::routing::post(create_sample_content),
        )
        .route(
            "/onboarding/suggestions",
            axum::routing::get(get_suggestions),
        )
}

fn build_sample_documents() -> Vec<DocumentMetadata> {
    let now = chrono::Utc::now();
    vec![
        DocumentMetadata {
            id: Id::new().to_string(),
            title: "Welcome to Tachyon".to_string(),
            slug: Some("welcome-to-tachyon".to_string()),
            author_id: "current_user".to_string(),
            description: Some("Your personal knowledge management system — learn what Tachyon can do for you".to_string()),
            tags: serde_json::to_string(&["getting-started", "welcome"]).unwrap(),
            frontmatter: serde_json::to_string(&serde_json::json!({
                "title": "Welcome to Tachyon",
                "description": "Your personal knowledge management system",
                "created": now.to_rfc3339(),
                "tags": ["getting-started", "welcome"],
                "category": "guide"
            })).ok(),
            project_id: None,
            visibility: "private".to_string(),
            status: "published".to_string(),
            content_type: "markdown".to_string(),
            word_count: 280,
            character_count: 1800,
            read_count: 0,
            edit_count: 1,
            content: Some(build_welcome_content()),
            html: None,
            created_at: now,
            updated_at: now,
            published_at: Some(now),
            content_hash: None,
            conflict_detected: None,
        },
        DocumentMetadata {
            id: Id::new().to_string(),
            title: "Getting Started".to_string(),
            slug: Some("getting-started".to_string()),
            author_id: "current_user".to_string(),
            description: Some("A quick start guide to help you create your first document and explore Tachyon's features".to_string()),
            tags: serde_json::to_string(&["getting-started", "tutorial"]).unwrap(),
            frontmatter: serde_json::to_string(&serde_json::json!({
                "title": "Getting Started",
                "description": "Quick start guide for new users",
                "tags": ["getting-started", "tutorial"],
                "category": "guide"
            })).ok(),
            project_id: None,
            visibility: "private".to_string(),
            status: "published".to_string(),
            content_type: "markdown".to_string(),
            word_count: 350,
            character_count: 2200,
            read_count: 0,
            edit_count: 1,
            content: Some(build_getting_started_content()),
            html: None,
            created_at: now,
            updated_at: now,
            published_at: Some(now),
            content_hash: None,
            conflict_detected: None,
        },
        DocumentMetadata {
            id: Id::new().to_string(),
            title: "Markdown Guide".to_string(),
            slug: Some("markdown-guide".to_string()),
            author_id: "current_user".to_string(),
            description: Some("Learn all the markdown features supported by Tachyon for rich document editing".to_string()),
            tags: serde_json::to_string(&["reference", "markdown"]).unwrap(),
            frontmatter: serde_json::to_string(&serde_json::json!({
                "title": "Markdown Guide",
                "description": "Supported markdown features in Tachyon",
                "tags": ["reference", "markdown"],
                "category": "reference"
            })).ok(),
            project_id: None,
            visibility: "private".to_string(),
            status: "published".to_string(),
            content_type: "markdown".to_string(),
            word_count: 420,
            character_count: 2700,
            read_count: 0,
            edit_count: 1,
            content: Some(build_markdown_guide_content()),
            html: None,
            created_at: now,
            updated_at: now,
            published_at: Some(now),
            content_hash: None,
            conflict_detected: None,
        },
        DocumentMetadata {
            id: Id::new().to_string(),
            title: "Knowledge Graph".to_string(),
            slug: Some("knowledge-graph".to_string()),
            author_id: "current_user".to_string(),
            description: Some("Understand how Tachyon's knowledge graph connects your documents and ideas".to_string()),
            tags: serde_json::to_string(&["features", "knowledge-graph"]).unwrap(),
            frontmatter: serde_json::to_string(&serde_json::json!({
                "title": "Knowledge Graph",
                "description": "How the knowledge graph connects your ideas",
                "tags": ["features", "knowledge-graph"],
                "category": "guide"
            })).ok(),
            project_id: None,
            visibility: "private".to_string(),
            status: "published".to_string(),
            content_type: "markdown".to_string(),
            word_count: 300,
            character_count: 2000,
            read_count: 0,
            edit_count: 1,
            content: Some(build_knowledge_graph_content()),
            html: None,
            created_at: now,
            updated_at: now,
            published_at: Some(now),
            content_hash: None,
            conflict_detected: None,
        },
        DocumentMetadata {
            id: Id::new().to_string(),
            title: "Keyboard Shortcuts".to_string(),
            slug: Some("keyboard-shortcuts".to_string()),
            author_id: "current_user".to_string(),
            description: Some("Reference card for all keyboard shortcuts available in Tachyon".to_string()),
            tags: serde_json::to_string(&["reference", "shortcuts"]).unwrap(),
            frontmatter: serde_json::to_string(&serde_json::json!({
                "title": "Keyboard Shortcuts",
                "description": "All available keyboard shortcuts",
                "tags": ["reference", "shortcuts"],
                "category": "reference"
            })).ok(),
            project_id: None,
            visibility: "private".to_string(),
            status: "published".to_string(),
            content_type: "markdown".to_string(),
            word_count: 250,
            character_count: 1600,
            read_count: 0,
            edit_count: 1,
            content: Some(build_keyboard_shortcuts_content()),
            html: None,
            created_at: now,
            updated_at: now,
            published_at: Some(now),
            content_hash: None,
            conflict_detected: None,
        },
    ]
}

fn build_welcome_content() -> String {
    r#"# Welcome to Tachyon

Tachyon is your personal knowledge management system designed to help you capture, connect, and discover ideas.

## What is Tachyon?

Tachyon provides a powerful platform for organizing your knowledge:

- **Rich Markdown Editing** — Write documents using familiar markdown syntax
- **Knowledge Graph** — Automatically connect related ideas using wiki-links
- **Real-time Collaboration** — Work together with your team on shared documents
- **Full-text Search** — Find anything instantly across all your documents
- **Spaces** — Organize documents into logical collections

## Quick Links

- [[Getting Started]] — Create your first document
- [[Markdown Guide]] — Learn supported markdown features
- [[Knowledge Graph]] — Understand how documents connect
- [[Keyboard Shortcuts]] — Speed up your workflow

## Your Next Steps

1. **Create a document** — Click "New Document" in the sidebar
2. **Add tags** — Categorize your documents for easy discovery
3. **Use wiki-links** — Type `[[` to link to other documents
4. **Invite your team** — Share your knowledge with collaborators

> "The best way to get started is to start writing." — Just begin.

Welcome aboard!
"#.to_string()
}

fn build_getting_started_content() -> String {
    r#"# Getting Started

This guide walks you through the basics of Tachyon so you can start capturing your ideas right away.

## Creating Your First Document

1. Click the **New Document** button in the sidebar
2. Give your document a title
3. Start writing in the markdown editor
4. Your document saves automatically as you type

## Writing in Markdown

Tachyon uses standard markdown with some enhancements:

```markdown
# Heading 1
## Heading 2
### Heading 3

**Bold text** and *italic text*

- Unordered list item
- Another item

1. Ordered list
2. Second item

> A blockquote for important notes

[Link text](https://example.com)
```

See the [[Markdown Guide]] for the full reference.

## Linking Documents

The most powerful feature is **wiki-links**. Type `[[` and start typing the name of another document:

```
This connects to [[Getting Started]]
```

These links create the edges in your [[Knowledge Graph]], making it easy to navigate between related ideas.

## Organizing with Tags

Add tags to categorize your documents:

```markdown
---
tags: [project, meeting, q1]
---
```

Tags make your documents searchable and help you discover patterns across your knowledge base.

## Spaces

Create **spaces** to group related documents:

- **Work** — Projects, meeting notes, specs
- **Personal** — Journal, reading notes, ideas
- **Research** — Papers, references, experiments

## What's Next?

- Check out the [[Markdown Guide]] for advanced formatting
- Learn about the [[Knowledge Graph]] feature
- Review [[Keyboard Shortcuts]] to work faster
"#.to_string()
}

fn build_markdown_guide_content() -> String {
    r#"# Markdown Guide

Tachyon supports a rich set of markdown features for document authoring.

## Text Formatting

| Syntax | Result |
|--------|--------|
| `**bold**` | **bold** |
| `*italic*` | *italic* |
| `~~strikethrough~~` | ~~strikethrough~~ |
| `` `inline code` `` | `inline code` |
| `[link](url)` | [link](url) |

## Headings

```markdown
# Heading 1
## Heading 2
### Heading 3
#### Heading 4
```

## Lists

Unordered lists:

- Item one
- Item two
  - Nested item
  - Another nested item
- Item three

Ordered lists:

1. First step
2. Second step
3. Third step

## Code Blocks

With syntax highlighting:

```rust
fn main() {
    println!("Hello, Tachyon!");
}
```

```python
def greet(name):
    return f"Hello, {name}!"
```

```javascript
const greeting = (name) => `Hello, ${name}!`;
```

## Blockquotes

> This is a blockquote.
> It can span multiple lines.
>
> > Nested blockquotes are also supported.

## Tables

| Feature | Status | Notes |
|---------|--------|-------|
| Headers | Supported | H1 through H4 |
| Lists | Supported | Ordered and unordered |
| Code blocks | Supported | With syntax highlighting |
| Tables | Supported | GFM-style tables |
| Wiki-links | Supported | `[[document-name]]` |

## Wiki-Links

Link to other documents in your knowledge base:

```markdown
See [[Getting Started]] for a quick introduction.
The [[Knowledge Graph]] connects all your ideas.
```

## Frontmatter

Add metadata to your documents:

```yaml
---
title: My Document
description: A helpful description
tags: [tag1, tag2, tag3]
category: guide
created: 2026-01-01
---
```

## Horizontal Rules

---

Separate sections with three or more hyphens.

See [[Getting Started]] for more on document creation.
"#
    .to_string()
}

fn build_knowledge_graph_content() -> String {
    r#"# Knowledge Graph

The knowledge graph is one of Tachyon's most powerful features. It automatically maps the connections between your documents, helping you discover relationships and navigate your knowledge.

## How It Works

Every time you create a **wiki-link** between documents, Tachyon adds an edge to the graph:

```
[[Getting Started]] → [[Markdown Guide]]
[[Getting Started]] → [[Knowledge Graph]]
[[Markdown Guide]] → [[Keyboard Shortcuts]]
```

These connections form a web of knowledge that grows as you write.

## Benefits

- **Discover relationships** — Find unexpected connections between ideas
- **Navigate contextually** — Jump between related documents effortlessly
- **Visualize your knowledge** — See the big picture of what you know
- **Identify gaps** — Spot isolated documents that need connections

## Building Your Graph

### 1. Write and Link

As you write, use `[[wiki-links]]` to reference related concepts:

```markdown
The [[Getting Started]] guide covers the basics.
For advanced formatting, see the [[Markdown Guide]].
```

### 2. Use Tags Consistently

Tags create implicit connections. Documents with similar tags appear near each other in the graph.

### 3. Review and Refine

Periodically review your graph to:

- Find orphan documents (no connections)
- Strengthen important connections
- Discover clusters of related content

## Graph Visualization

Tachyon renders your knowledge graph as an interactive visualization where:

- **Nodes** represent documents (sized by connection count)
- **Edges** represent wiki-links (colored by relationship type)
- **Clusters** reveal groups of related content

## Tips for a Healthy Knowledge Graph

| Practice | Why |
|----------|-----|
| Link generously | More connections = richer graph |
| Use descriptive titles | Makes links more meaningful |
| Tag consistently | Helps with clustering |
| Review weekly | Keeps your graph organized |

Start building your graph by linking from [[Getting Started]].
"#.to_string()
}

fn build_keyboard_shortcuts_content() -> String {
    r#"# Keyboard Shortcuts

Speed up your workflow with these keyboard shortcuts.

## Editor Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+B` | Bold |
| `Ctrl+I` | Italic |
| `Ctrl+K` | Insert link |
| `Ctrl+Shift+K` | Insert wiki-link |
| `Ctrl+`` ` | Toggle code block |
| `Ctrl+Shift+H` | Toggle heading level |
| `Ctrl+Shift+U` | Toggle unordered list |
| `Ctrl+Shift+O` | Toggle ordered list |
| `Ctrl+Shift+Q` | Toggle blockquote |
| `Ctrl+Shift+T` | Insert table |

## Navigation

| Shortcut | Action |
|----------|--------|
| `Ctrl+P` | Quick document search |
| `Ctrl+Shift+F` | Global search |
| `Ctrl+G` | Go to knowledge graph |
| `Ctrl+Shift+N` | New document |
| `Ctrl+S` | Save (also auto-saves) |
| `Ctrl+O` | Open document sidebar |
| `Ctrl+\` | Toggle sidebar |

## Document Management

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+D` | Duplicate document |
| `Ctrl+Shift+M` | Move document |
| `Ctrl+Shift+L` | Manage tags |
| `Delete` | Move to trash |

## Collaboration

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+C` | Toggle comment panel |
| `@username` | Mention a collaborator |
| `Ctrl+Enter` | Submit comment |

## General

| Shortcut | Action |
|----------|--------|
| `Ctrl+/` | Toggle command palette |
| `Ctrl+,` | Open settings |
| `Ctrl+?` | Show keyboard shortcuts |
| `Esc` | Close panels / modals |

> Tip: Press `Ctrl+/` to open the command palette for quick access to any action.

For more on getting started, see [[Getting Started]].
"#
    .to_string()
}

fn server_error(code: &str, message: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code: code.to_string(),
            message: message.to_string(),
        }),
    )
}

fn bad_request(code: &str, message: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: code.to_string(),
            message: message.to_string(),
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_documents_have_valid_content() {
        let docs = build_sample_documents();
        assert_eq!(docs.len(), 5);
        assert!(docs[0]
            .content
            .as_ref()
            .unwrap()
            .contains("Welcome to Tachyon"));
        assert!(docs[1]
            .content
            .as_ref()
            .unwrap()
            .contains("Getting Started"));
        assert!(docs[2].content.as_ref().unwrap().contains("Markdown Guide"));
        assert!(docs[3]
            .content
            .as_ref()
            .unwrap()
            .contains("Knowledge Graph"));
        assert!(docs[4]
            .content
            .as_ref()
            .unwrap()
            .contains("Keyboard Shortcuts"));
    }

    #[test]
    fn test_sample_documents_have_tags() {
        let docs = build_sample_documents();
        for doc in &docs {
            let tags: Vec<String> = serde_json::from_str(&doc.tags).unwrap();
            assert!(!tags.is_empty(), "Document {} has no tags", doc.title);
        }
    }

    #[test]
    fn test_sample_documents_have_frontmatter() {
        let docs = build_sample_documents();
        for doc in &docs {
            assert!(
                doc.frontmatter.is_some(),
                "Document {} missing frontmatter",
                doc.title
            );
        }
    }

    #[test]
    fn test_sample_documents_have_wiki_links() {
        let docs = build_sample_documents();
        let all_content: String = docs
            .iter()
            .filter_map(|d| d.content.as_ref())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        assert!(all_content.contains("[[Getting Started]]"));
        assert!(all_content.contains("[[Markdown Guide]]"));
    }

    #[test]
    fn test_onboarding_status_response_serialization() {
        let resp = OnboardingStatusResponse {
            completed: false,
            steps: vec![tachyon_database::OnboardingStep {
                id: "create_first_document".to_string(),
                name: "Create Your First Document".to_string(),
                completed: false,
            }],
            current_step: 0,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("create_first_document"));
    }

    #[test]
    fn test_complete_step_request_deserialization() {
        let json = r#"{"step_id":"create_first_document"}"#;
        let req: CompleteStepRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.step_id, "create_first_document");
    }

    #[test]
    fn test_sample_content_response_serialization() {
        let resp = SampleContentResponse {
            created: 3,
            skipped: 2,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"created\":3"));
        assert!(json.contains("\"skipped\":2"));
    }

    #[test]
    fn test_suggestions_response_serialization() {
        let resp = SuggestionsResponse {
            suggested_tags: vec!["rust".to_string()],
            suggested_templates: vec![TemplateSuggestion {
                id: "test".to_string(),
                name: "Test".to_string(),
                description: "A test template".to_string(),
            }],
            tips: vec!["Use wiki-links".to_string()],
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("rust"));
        assert!(json.contains("wiki-links"));
    }

    #[test]
    fn test_sample_documents_have_correct_status() {
        let docs = build_sample_documents();
        for doc in &docs {
            assert_eq!(
                doc.status, "published",
                "Document {} should be published",
                doc.title
            );
        }
    }

    #[test]
    fn test_sample_documents_have_correct_author() {
        let docs = build_sample_documents();
        for doc in &docs {
            assert_eq!(doc.author_id, "current_user");
        }
    }

    #[test]
    fn test_sample_documents_have_correct_visibility() {
        let docs = build_sample_documents();
        for doc in &docs {
            assert_eq!(doc.visibility, "private");
        }
    }

    #[test]
    fn test_sample_documents_have_correct_content_type() {
        let docs = build_sample_documents();
        for doc in &docs {
            assert_eq!(doc.content_type, "markdown");
        }
    }

    #[test]
    fn test_onboarding_status_all_steps_completed() {
        let resp = OnboardingStatusResponse {
            completed: true,
            steps: vec![
                tachyon_database::OnboardingStep {
                    id: "step1".to_string(),
                    name: "Step 1".to_string(),
                    completed: true,
                },
                tachyon_database::OnboardingStep {
                    id: "step2".to_string(),
                    name: "Step 2".to_string(),
                    completed: true,
                },
            ],
            current_step: 2,
        };
        assert!(resp.completed);
        assert_eq!(resp.current_step, 2);
    }

    #[test]
    fn test_complete_step_response_serialization() {
        let resp = CompleteStepResponse {
            success: true,
            step_id: "create_first_document".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("create_first_document"));
    }

    #[test]
    fn test_template_suggestion_serialization() {
        let tmpl = TemplateSuggestion {
            id: "meeting-notes".to_string(),
            name: "Meeting Notes".to_string(),
            description: "Structured template".to_string(),
        };
        let json = serde_json::to_string(&tmpl).unwrap();
        assert!(json.contains("meeting-notes"));
        assert!(json.contains("Meeting Notes"));
    }

    #[test]
    fn test_sample_documents_have_unique_ids() {
        let docs = build_sample_documents();
        let ids: Vec<&str> = docs.iter().map(|d| d.id.as_str()).collect();
        let unique_ids: std::collections::HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique_ids.len(), "Document IDs should be unique");
    }

    #[test]
    fn test_sample_documents_have_unique_slugs() {
        let docs = build_sample_documents();
        let slugs: Vec<&str> = docs.iter().filter_map(|d| d.slug.as_deref()).collect();
        let unique_slugs: std::collections::HashSet<&str> = slugs.iter().copied().collect();
        assert_eq!(
            slugs.len(),
            unique_slugs.len(),
            "Document slugs should be unique"
        );
    }

    #[test]
    fn test_build_welcome_content_contains_links() {
        let content = build_welcome_content();
        assert!(content.contains("[[Getting Started]]"));
        assert!(content.contains("[[Markdown Guide]]"));
        assert!(content.contains("[[Knowledge Graph]]"));
        assert!(content.contains("[[Keyboard Shortcuts]]"));
    }

    #[test]
    fn test_build_getting_started_content_has_sections() {
        let content = build_getting_started_content();
        assert!(content.contains("## Creating Your First Document"));
        assert!(content.contains("## Writing in Markdown"));
        assert!(content.contains("## Linking Documents"));
        assert!(content.contains("## Organizing with Tags"));
    }

    #[test]
    fn test_build_markdown_guide_has_examples() {
        let content = build_markdown_guide_content();
        assert!(content.contains("## Text Formatting"));
        assert!(content.contains("## Code Blocks"));
        assert!(content.contains("## Wiki-Links"));
        assert!(content.contains("## Frontmatter"));
    }

    #[test]
    fn test_suggestions_has_expected_tags() {
        let suggestions = SuggestionsResponse {
            suggested_tags: vec!["test-tag".to_string()],
            suggested_templates: vec![],
            tips: vec![],
        };
        assert!(suggestions.suggested_tags.contains(&"test-tag".to_string()));
    }

    #[test]
    fn test_complete_step_request_empty_step_id_fails_deserialization() {
        let json = r#"{"step_id":""}"#;
        let req: Result<CompleteStepRequest, _> = serde_json::from_str(json);
        assert!(req.is_ok());
    }
}
