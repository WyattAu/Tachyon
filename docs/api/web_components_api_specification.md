# TACHYON: WEB COMPONENTS API SPECIFICATION

**Document ID:** TACHYON-API-012-V1.0
**Date:** February 2026
**Status:** Proposed
**Classification:** API Specification Document
**Compliance Level:** ISO/IEC 26514:2021, IEEE 829-2008

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [Component Design Principles](#2-component-design-principles)
3. [Document Components](#3-document-components)
4. [Repository Components](#4-repository-components)
5. [Search Components](#5-search-components)
6. [User Components](#6-user-components)
7. [Component Communication](#7-component-communication)
8. [Component State Management](#8-component-state-management)
9. [Component Security](#9-component-security)
10. [Component Performance](#10-component-performance)
11. [References](#11-references)

---

## 1. INTRODUCTION

### 1.1. Document Purpose

This specification defines the comprehensive Web Components API for the Tachyon toolchain web frontend. The specification provides detailed interface definitions, component contracts, and implementation guidelines for all web components built using the Leptos framework with WebAssembly (WASM) compilation.

The Web Components API serves as the foundation for building reactive, type-safe, and performant user interfaces that operate across both browser-based and desktop deployment modes. This specification ensures consistency, maintainability, and adherence to architectural principles established in [ADR-004](../.specs/02_adrs/004_leptos_for_web_frontend.md) and [ADR-005](../.specs/02_adrs/005_bun_for_javascript_runtime.md).

### 1.2. Scope

This specification covers:

- **Component Interface Definitions:** TypeScript and Rust interfaces for all web components
- **Component Communication Patterns:** Props, callbacks, and event handling mechanisms
- **State Management Contracts:** Signal-based reactive state management patterns
- **Security Requirements:** Authentication, authorization, and input validation
- **Performance Specifications:** Rendering optimization and bundle size constraints
- **Accessibility Standards:** WCAG 2.1 AA compliance requirements

Out of scope:
- Server-side API endpoints (covered in Server API Specification)
- Desktop-specific Tauri APIs (covered in Desktop API Specification)
- Core rendering engine internals (covered in System Architecture)

### 1.3. Technology Stack

The Web Components API is built on the following technology stack:

| Component | Technology | Version | Purpose |
|-----------|-------------|----------|---------|
| **Frontend Framework** | Leptos | v0.8.15 | Reactive web framework with fine-grained reactivity |
| **SSR Integration** | leptos_axum | v0.8.7 | Server-side rendering with Axum integration |
| **Routing** | leptos_router | v0.8.11 | Client-side routing with server fallback |
| **Runtime** | Bun | Latest stable | JavaScript runtime and build tooling |
| **Build Tool** | Vite | v7.3.1 | Optimized bundle generation |
| **CSS Framework** | TailwindCSS | v4.1.18 | Utility-first CSS framework |
| **WASM Target** | wasm32-unknown-unknown | - | Browser WebAssembly compilation |

### 1.4. Component Architecture Overview

The Tachyon web frontend implements a hierarchical component architecture following the Single Responsibility Principle and High Cohesion, Low Coupling design principles.

```
Application Root
├── Layout Components
│   ├── AppShell
│   ├── Sidebar
│   ├── Header
│   └── MainContent
├── Feature Components
│   ├── DocumentEditor
│   ├── DocumentViewer
│   ├── RepositoryList
│   ├── SearchInterface
│   └── UserProfile
└── Utility Components
    ├── Modal
    ├── Notification
    ├── LoadingIndicator
    └── ErrorBoundary
```

### 1.5. Component Lifecycle

All components in the Tachyon web frontend follow a standardized lifecycle:

1. **Initialization:** Component props are received and signals are created
2. **Mounting:** Component is rendered to the DOM (SSR or CSR)
3. **Update:** Reactive signals trigger targeted DOM updates
4. **Unmounting:** Component is removed from DOM and resources are cleaned up

### 1.6. Type Safety Guarantees

The Web Components API enforces strict type safety through:

- **Rust Type System:** Compile-time type checking for WASM components
- **TypeScript Interfaces:** Type-safe component props and callbacks
- **Generated Type Definitions:** Automatic .d.ts generation from Rust components
- **Runtime Validation:** Input validation at component boundaries

---

## 2. COMPONENT DESIGN PRINCIPLES

### 2.1. Fundamental Design Principles

The Tachyon web components adhere to the following fundamental design principles:

#### 2.1.1. Single Responsibility Principle

Each component shall have a single, well-defined responsibility. Components should not combine unrelated functionality.

**Example:**
- ✅ **Valid:** `DocumentEditor` handles document editing operations
- ❌ **Invalid:** `DocumentEditor` also handles user authentication

#### 2.1.2. High Cohesion, Low Coupling

Components should exhibit high internal cohesion (related functionality grouped together) and low coupling (minimal dependencies on other components).

**Cohesion Guidelines:**
- Group related state and operations within a component
- Extract unrelated functionality into separate components
- Use composition to combine simple components into complex ones

**Coupling Guidelines:**
- Prefer props over direct component references
- Use callback props for parent-child communication
- Implement event emitters for cross-component communication
- Avoid direct DOM manipulation across component boundaries

#### 2.1.3. Composition over Inheritance

Component reuse should be achieved through composition rather than inheritance. Components should be designed as building blocks that can be combined to create complex UIs.

**Composition Pattern:**
```rust
#[component]
fn DocumentCard(
    document: Document,
    on_edit: Callback<DocumentId>,
    on_delete: Callback<DocumentId>,
) -> impl IntoView {
    view! {
        <div class="document-card">
            <DocumentTitle title=document.title />
            <DocumentSummary summary=document.summary />
            <DocumentActions
                document_id=document.id
                on_edit=on_edit
                on_delete=on_delete
            />
        </div>
    }
}
```

#### 2.1.4. Explicit Dependencies

All component dependencies must be explicitly declared through props. Implicit dependencies (global state, external services) should be minimized and clearly documented.

**Explicit Dependency Declaration:**
```rust
#[component]
fn DocumentEditor(
    document: ReadSignal<Document>,
    on_save: Callback<Document>,
    api_client: Rc<ApiClient>,
    // Dependencies are explicit in props
) -> impl IntoView {
    // Component implementation
}
```

### 2.2. Component Interface Contracts

Every component must define a clear interface contract specifying:

#### 2.2.1. Props Interface

Props define the inputs to a component and must be fully typed with documentation.

**Props Interface Template:**
```rust
/// Props for the DocumentEditor component.
///
/// # Fields
///
/// * `document` - The document being edited (reactive signal)
/// * `on_save` - Callback invoked when document is saved
/// * `on_cancel` - Callback invoked when editing is cancelled
/// * `readonly` - Whether the editor is in read-only mode
///
/// # Constraints
///
/// * `document` must not be None when component is mounted
/// * `on_save` callback is required unless `readonly` is true
#[derive(Clone, Props)]
pub struct DocumentEditorProps {
    /// The document being edited (reactive signal)
    pub document: ReadSignal<Option<Document>>,
    
    /// Callback invoked when document is saved
    #[prop(optional)]
    pub on_save: Option<Callback<Document>>,
    
    /// Callback invoked when editing is cancelled
    #[prop(optional)]
    pub on_cancel: Option<Callback<()>>,
    
    /// Whether the editor is in read-only mode
    #[prop(default = false)]
    pub readonly: bool,
}
```

#### 2.2.2. Component Behavior Contract

Each component must document its behavior including:

- **Pre-conditions:** Conditions that must be true for the component to function correctly
- **Post-conditions:** Conditions that are guaranteed to be true after component operations
- **Invariants:** Conditions that remain true throughout the component's lifecycle
- **Side Effects:** Observable effects of component operations

**Behavior Contract Example:**
```rust
/// DocumentEditor Component Behavior Contract
///
/// # Pre-conditions
///
/// * The `document` signal must contain a valid Document when mounted
/// * The API client must be initialized and authenticated
///
/// # Post-conditions
///
/// * After successful save, the document signal contains the updated document
/// * After save failure, the document signal remains unchanged
///
/// # Invariants
///
/// * The editor content matches the document signal content
/// * The save button is disabled when content is unchanged
///
/// # Side Effects
///
/// * Invokes `on_save` callback with updated document
/// * Triggers API call to persist document changes
/// * Displays notification on save success or failure
```

#### 2.2.3. Error Handling Contract

Components must define clear error handling contracts specifying:

- **Error Types:** Types of errors the component can encounter
- **Error Propagation:** How errors are propagated to parent components
- **Error Recovery:** How errors can be recovered from
- **User Communication:** How errors are communicated to users

**Error Handling Contract Template:**
```rust
/// DocumentEditor Error Handling Contract
///
/// # Error Types
///
/// * `ValidationError` - Invalid document content
/// * `NetworkError` - API communication failure
/// * `ConflictError` - Concurrent modification detected
///
/// # Error Propagation
///
/// * Validation errors are displayed inline within the editor
/// * Network errors trigger error notification and retry option
/// * Conflict errors display conflict resolution modal
///
/// # Error Recovery
///
/// * Validation errors are resolved by user correcting content
/// * Network errors can be retried automatically or manually
/// * Conflict errors require user to resolve or discard changes
///
/// # User Communication
///
/// * Error messages are clear, actionable, and non-technical
/// * Error notifications include suggested actions
/// * Error states are visually distinct from normal states
```

### 2.3. Component State Management

Components must use Leptos signals for reactive state management following these principles:

#### 2.3.1. Signal Classification

Signals should be classified based on their scope and lifetime:

| Signal Type | Scope | Lifetime | Example |
|--------------|--------|-----------|----------|
| **Local Signal** | Component internal | Component lifecycle | Editor cursor position |
| **Prop Signal** | Derived from props | Prop lifetime | Formatted document title |
| **Global Signal** | Application-wide | Application lifecycle | User authentication state |
| **Cached Signal** | Shared across components | Cache TTL | Document content cache |

#### 2.3.2. Signal Creation Patterns

**Local Signal Creation:**
```rust
#[component]
fn DocumentEditor(document: ReadSignal<Document>) -> impl IntoView {
    // Local signal for editor state
    let (content, set_content) = create_signal(document.get().content);
    let (cursor_position, set_cursor_position) = create_signal(0);
    
    view! {
        // Component implementation
    }
}
```

**Derived Signal Creation:**
```rust
#[component]
fn DocumentCard(document: ReadSignal<Document>) -> impl IntoView {
    // Derived signal from prop
    let formatted_date = create_memo(move |_| {
        document.get().created_at.format("%Y-%m-%d")
    });
    
    view! {
        <div class="document-card">
            <span>{formatted_date}</span>
        </div>
    }
}
```

#### 2.3.3. Signal Synchronization

When multiple components need to share state, use one of the following patterns:

**Pattern 1: Lift State Up**
```rust
// Parent component owns state
#[component]
fn DocumentList() -> impl IntoView {
    let (selected_document, set_selected_document) = create_signal(None);
    
    view! {
        <div>
            <DocumentListItems
                selected_document=selected_document
                on_select=set_selected_document
            />
            <DocumentViewer document=selected_document />
        </div>
    }
}
```

**Pattern 2: Global Store**
```rust
// Global store for application state
#[derive(Clone)]
pub struct ApplicationStore {
    pub session: Signal<Option<Session>>,
    pub theme: Signal<Theme>,
    pub notifications: Signal<Vec<Notification>>,
}

// Components access global store via context
#[component]
fn App() -> impl IntoView {
    let store = ApplicationStore::new();
    provide_context(store);
    
    view! {
        <Router>
            <Routes>
                <Route path="/" view=Home />
                <Route path="/documents" view=DocumentList />
            </Routes>
        </Router>
    }
}
```

### 2.4. Component Performance Principles

Components must be designed for optimal performance:

#### 2.4.1. Fine-Grained Reactivity

Leverage Leptos's fine-grained reactivity to minimize DOM updates:

```rust
// ✅ Good: Fine-grained reactivity
#[component]
fn DocumentStats(document: ReadSignal<Document>) -> impl IntoView {
    let word_count = create_memo(move |_| {
        document.get().content.split_whitespace().count()
    });
    
    view! {
        <span>{word_count} words</span>
    }
}

// ❌ Bad: Coarse-grained reactivity
#[component]
fn DocumentStats(document: ReadSignal<Document>) -> impl IntoView {
    view! {
        <span>{document.get().content.split_whitespace().count()} words</span>
    }
}
```

#### 2.4.2. Lazy Rendering

Use lazy rendering for expensive components:

```rust
#[component]
fn DocumentList(documents: ReadSignal<Vec<Document>>) -> impl IntoView {
    view! {
        <div class="document-list">
            {move || {
                documents.get()
                    .into_iter()
                    .map(|doc| view! { <DocumentCard document=doc /> })
                    .collect_view()
            }}
        </div>
    }
}
```

#### 2.4.3. Memoization

Memoize expensive computations:

```rust
#[component]
fn DocumentPreview(document: ReadSignal<Document>) -> impl IntoView {
    let preview = create_memo(move |_| {
        let doc = document.get();
        // Expensive computation
        generate_preview(&doc.content, 200)
    });
    
    view! {
        <div class="preview">{preview}</div>
    }
}
```

### 2.5. Component Accessibility Principles

All components must adhere to WCAG 2.1 AA accessibility standards:

#### 2.5.1. Semantic HTML

Use semantic HTML elements for proper screen reader interpretation:

```rust
// ✅ Good: Semantic HTML
view! {
    <article class="document">
        <header>
            <h1>{document.title}</h1>
        </header>
        <main>
            <div inner_html=document.content />
        </main>
        <footer>
            <p>By {document.author}</p>
        </footer>
    </article>
}

// ❌ Bad: Non-semantic HTML
view! {
    <div class="document">
        <div class="title">{document.title}</div>
        <div class="content" inner_html=document.content />
        <div class="author">By {document.author}</div>
    </div>
}
```

#### 2.5.2. ARIA Attributes

Provide ARIA attributes for interactive elements:

```rust
view! {
    <button
        aria_label="Save document"
        aria_pressed=save_in_progress
        disabled=save_in_progress
        on:click=on_save
    >
        {move || if save_in_progress.get() { "Saving..." } else { "Save" }}
    </button>
}
```

#### 2.5.3. Keyboard Navigation

Ensure full keyboard support:

```rust
view! {
    <div
        role="listbox"
        tabindex="0"
        on:keydown=handle_keyboard
        aria_label="Document list"
    >
        {documents}
    </div>
}
```

### 2.6. Component Security Principles

Components must implement security best practices:

#### 2.6.1. Input Sanitization

Sanitize all user inputs to prevent XSS attacks:

```rust
view! {
    <div class="document-content">
        // Leptos automatically escapes HTML in text nodes
        <p>{document.title}</p>
        
        // For HTML content, use sanitization
        <div inner_html=sanitize_html(&document.content) />
    </div>
}
```

#### 2.6.2. Content Security Policy

Implement Content Security Policy headers:

```rust
// In server configuration
pub fn csp_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'wasm-unsafe-eval'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: https:; \
             connect-src 'self' wss://"
        ),
    );
    headers
}
```

#### 2.6.3. CSRF Protection

Implement CSRF tokens for state-changing operations:

```rust
#[component]
fn DocumentForm(document: Document) -> impl IntoView {
    let csrf_token = use_context::<CsrfToken>();
    
    view! {
        <form method="post" action="/documents/save">
            <input type="hidden" name="csrf_token" value=csrf_token.get() />
            <!-- Form fields -->
        </form>
    }
}
```

---

---

## 3. DOCUMENT COMPONENTS

Document components provide functionality for viewing, editing, and managing documents within the Tachyon system. These components implement requirements from [REQ-WEB-016 through REQ-WEB-020](../.specs/04_future_state/reqs/web_requirements.md).

### 3.1. DocumentViewer Component

The DocumentViewer component displays rendered document content with proper formatting and styling.

#### 3.1.1. Component Interface

**TypeScript Interface:**
```typescript
/**
 * Props for DocumentViewer component.
 *
 * @property document - The document to display (reactive signal)
 * @property on_edit - Callback invoked when edit button is clicked
 * @property on_share - Callback invoked when share button is clicked
 * @property readonly - Whether viewer is in read-only mode
 * @property show_metadata - Whether to display document metadata
 *
 * @constraint document must not be None when component is mounted
 * @constraint on_edit callback is optional and only invoked when readonly is false
 */
export interface DocumentViewerProps {
    document: ReadSignal<Document | null>;
    on_edit?: Callback<DocumentId>;
    on_share?: Callback<DocumentId>;
    readonly?: boolean;
    show_metadata?: boolean;
}
```

**Rust Component Definition:**
```rust
/// Props for DocumentViewer component.
///
/// # Fields
///
/// * `document` - The document to display (reactive signal)
/// * `on_edit` - Callback invoked when edit button is clicked
/// * `on_share` - Callback invoked when share button is clicked
/// * `readonly` - Whether viewer is in read-only mode
/// * `show_metadata` - Whether to display document metadata
///
/// # Constraints
///
/// * `document` must not be None when component is mounted
/// * `on_edit` callback is optional and only invoked when `readonly` is false
#[derive(Clone, Props)]
pub struct DocumentViewerProps {
    /// The document to display (reactive signal)
    pub document: ReadSignal<Option<Document>>,
    
    /// Callback invoked when edit button is clicked
    #[prop(optional)]
    pub on_edit: Option<Callback<DocumentId>>,
    
    /// Callback invoked when share button is clicked
    #[prop(optional)]
    pub on_share: Option<Callback<DocumentId>>,
    
    /// Whether viewer is in read-only mode
    #[prop(default = false)]
    pub readonly: bool,
    
    /// Whether to display document metadata
    #[prop(default = true)]
    pub show_metadata: bool,
}

/// DocumentViewer component displays rendered document content.
///
/// # Pre-conditions
///
/// * The `document` signal must contain a valid Document when mounted
/// * Document content must be valid Markdown
///
/// # Post-conditions
///
/// * Document content is rendered with proper formatting
/// * Metadata is displayed if `show_metadata` is true
///
/// # Invariants
///
/// * The rendered content matches the document signal content
/// * Edit button is hidden when `readonly` is true
///
/// # Side Effects
///
/// * Invokes `on_edit` callback when edit button is clicked
/// * Invokes `on_share` callback when share button is clicked
#[component]
pub fn DocumentViewer(props: DocumentViewerProps) -> impl IntoView {
    let document = props.document;
    let readonly = props.readonly;
    
    // Derived signal for formatted content
    let rendered_content = create_memo(move |_| {
        document.get()
            .as_ref()
            .map(|doc| render_markdown(&doc.content))
            .unwrap_or_default()
    });
    
    // Derived signal for metadata display
    let show_metadata = props.show_metadata && document.get().is_some();
    
    view! {
        <article class="document-viewer" aria_label="Document viewer">
            <Show
                when=move || document.get().is_some()
                fallback=|| view! { <div class="empty-state">"No document selected"</div> }
            >
                {move || {
                    document.get().map(|doc| view! {
                        <header class="document-header">
                            <h1 class="document-title">{doc.title.clone()}</h1>
                            <Show when=move || show_metadata>
                                <div class="document-metadata">
                                    <span class="metadata-item">
                                        "By " {doc.author.clone()}
                                    </span>
                                    <span class="metadata-item">
                                        {doc.created_at.format("%Y-%m-%d")}
                                    </span>
                                    <Show when=move || !doc.tags.is_empty()>
                                        <span class="metadata-item tags">
                                            {doc.tags.iter()
                                                .map(|tag| view! {
                                                    <span class="tag">{tag.clone()}</span>
                                                })
                                                .collect_view()}
                                        </span>
                                    </Show>
                                </div>
                            </Show>
                        </header>
                        <main class="document-content" inner_html=rendered_content />
                        <Show when=move || !readonly>
                            <footer class="document-actions">
                                <Show
                                    when=move || props.on_edit.is_some()
                                    fallback=|| view! { <div></div> }
                                >
                                    <button
                                        class="btn-primary"
                                        aria_label="Edit document"
                                        on:click=move |_| {
                                            if let Some(ref on_edit) = props.on_edit {
                                                on_edit.call(doc.id.clone());
                                            }
                                        }
                                    >
                                        "Edit"
                                    </button>
                                </Show>
                                <Show
                                    when=move || props.on_share.is_some()
                                    fallback=|| view! { <div></div> }
                                >
                                    <button
                                        class="btn-secondary"
                                        aria_label="Share document"
                                        on:click=move |_| {
                                            if let Some(ref on_share) = props.on_share {
                                                on_share.call(doc.id.clone());
                                            }
                                        }
                                    >
                                        "Share"
                                    </button>
                                </Show>
                            </footer>
                        </Show>
                    })}
                }
            </Show>
        </article>
    }
}
```

#### 3.1.2. Component Behavior

**Loading State:**
- Displays loading indicator while document is being fetched
- Shows skeleton UI for improved perceived performance

**Error State:**
- Displays user-friendly error message when document fails to load
- Provides retry option for transient errors

**Empty State:**
- Displays "No document selected" message when no document is loaded
- Provides guidance for selecting a document

**Interactive Features:**
- Edit button triggers `on_edit` callback with document ID
- Share button triggers `on_share` callback with document ID
- Both buttons are disabled when `readonly` is true

### 3.2. DocumentEditor Component

The DocumentEditor component provides a Markdown editor with live preview and auto-save functionality.

#### 3.2.1. Component Interface

**TypeScript Interface:**
```typescript
/**
 * Props for DocumentEditor component.
 *
 * @property document - The document being edited (reactive signal)
 * @property on_save - Callback invoked when document is saved
 * @property on_cancel - Callback invoked when editing is cancelled
 * @property auto_save_interval - Auto-save interval in milliseconds
 * @property show_preview - Whether to show live preview
 *
 * @constraint document must not be None when component is mounted
 * @constraint on_save callback is required
 * @constraint auto_save_interval must be between 1000 and 60000
 */
export interface DocumentEditorProps {
    document: ReadSignal<Document>;
    on_save: Callback<Document>;
    on_cancel?: Callback<()>;
    auto_save_interval?: number;
    show_preview?: boolean;
}
```

**Rust Component Definition:**
```rust
/// Props for DocumentEditor component.
///
/// # Fields
///
/// * `document` - The document being edited (reactive signal)
/// * `on_save` - Callback invoked when document is saved
/// * `on_cancel` - Callback invoked when editing is cancelled
/// * `auto_save_interval` - Auto-save interval in milliseconds
/// * `show_preview` - Whether to show live preview
///
/// # Constraints
///
/// * `document` must not be None when component is mounted
/// * `on_save` callback is required
/// * `auto_save_interval` must be between 1000 and 60000
#[derive(Clone, Props)]
pub struct DocumentEditorProps {
    /// The document being edited (reactive signal)
    pub document: ReadSignal<Document>,
    
    /// Callback invoked when document is saved
    pub on_save: Callback<Document>,
    
    /// Callback invoked when editing is cancelled
    #[prop(optional)]
    pub on_cancel: Option<Callback<()>>,
    
    /// Auto-save interval in milliseconds
    #[prop(default = 30000)]
    pub auto_save_interval: u64,
    
    /// Whether to show live preview
    #[prop(default = true)]
    pub show_preview: bool,
}

/// DocumentEditor component provides Markdown editing with live preview.
///
/// # Pre-conditions
///
/// * The `document` signal must contain a valid Document when mounted
/// * The API client must be initialized and authenticated
///
/// # Post-conditions
///
/// * After successful save, document signal contains updated document
/// * After save failure, document signal remains unchanged
///
/// # Invariants
///
/// * The editor content matches the document signal content
/// * The save button is disabled when content is unchanged
/// * Auto-save is disabled when content is unchanged
///
/// # Side Effects
///
/// * Invokes `on_save` callback with updated document
/// * Triggers API call to persist document changes
/// * Displays notification on save success or failure
#[component]
pub fn DocumentEditor(props: DocumentEditorProps) -> impl IntoView {
    let document = props.document;
    let on_save = props.on_save.clone();
    let auto_save_interval = props.auto_save_interval;
    let show_preview = props.show_preview;
    
    // Local signal for editor content
    let (content, set_content) = create_signal(document.get().content);
    let (title, set_title) = create_signal(document.get().title);
    let (tags, set_tags) = create_signal(document.get().tags.join(", "));
    
    // Local signal for save state
    let (is_saving, set_is_saving) = create_signal(false);
    let (has_unsaved_changes, set_has_unsaved_changes) = create_signal(false);
    
    // Derived signal for live preview
    let preview = create_memo(move |_| {
        render_markdown(&content.get())
    });
    
    // Derived signal for save button state
    let can_save = create_memo(move |_| {
        has_unsaved_changes.get() && !is_saving.get()
    });
    
    // Auto-save effect
    create_effect(move |_| {
        let content_value = content.get();
        let document_content = document.get().content;
        
        set_has_unsaved_changes.set(content_value != document_content);
    });
    
    // Auto-save timer
    create_effect(move |_| {
        if has_unsaved_changes.get() && !is_saving.get() {
            set_is_saving.set(true);
            
            // Trigger save after debounce
            set_timeout_with_handle(
                move || {
                    let updated_doc = Document {
                        id: document.get().id.clone(),
                        title: title.get(),
                        content: content.get(),
                        tags: tags.get()
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect(),
                        author: document.get().author.clone(),
                        created_at: document.get().created_at,
                        updated_at: Utc::now(),
                    };
                    
                    on_save.call(updated_doc);
                    set_has_unsaved_changes.set(false);
                    set_is_saving.set(false);
                },
                Duration::from_millis(auto_save_interval),
            );
        }
    });
    
    // Handle save button click
    let handle_save = move |_| {
        set_is_saving.set(true);
        
        let updated_doc = Document {
            id: document.get().id.clone(),
            title: title.get(),
            content: content.get(),
            tags: tags.get()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            author: document.get().author.clone(),
            created_at: document.get().created_at,
            updated_at: Utc::now(),
        };
        
        on_save.call(updated_doc);
        set_has_unsaved_changes.set(false);
        set_is_saving.set(false);
    };
    
    // Handle cancel button click
    let handle_cancel = move |_| {
        if let Some(ref on_cancel) = props.on_cancel {
            on_cancel.call(());
        }
    };
    
    view! {
        <div class="document-editor">
            <div class="editor-toolbar">
                <input
                    type="text"
                    class="editor-title-input"
                    placeholder="Document title"
                    value=title
                    on:input=move |e| {
                        set_title.set(event_target_value(&e));
                        set_has_unsaved_changes.set(true);
                    }
                    aria_label="Document title"
                />
                <div class="editor-actions">
                    <Show
                        when=move || props.on_cancel.is_some()
                        fallback=|| view! { <div></div> }
                    >
                        <button
                            class="btn-secondary"
                            aria_label="Cancel editing"
                            disabled=is_saving
                            on:click=handle_cancel
                        >
                            "Cancel"
                        </button>
                    </Show>
                    <button
                        class="btn-primary"
                        aria_label="Save document"
                        disabled=move || !can_save()
                        on:click=handle_save
                    >
                        {move || if is_saving.get() { "Saving..." } else { "Save" }}
                    </button>
                </div>
            </div>
            <div class="editor-container">
                <div class="editor-pane">
                    <textarea
                        class="editor-textarea"
                        placeholder="Write your document in Markdown..."
                        value=content
                        on:input=move |e| {
                            set_content.set(event_target_value(&e));
                            set_has_unsaved_changes.set(true);
                        }
                        aria_label="Document content"
                    />
                </div>
                <Show when=move || show_preview>
                    <div class="preview-pane">
                        <div class="preview-content" inner_html=preview />
                    </div>
                </Show>
            </div>
            <div class="editor-footer">
                <input
                    type="text"
                    class="editor-tags-input"
                    placeholder="Tags (comma-separated)"
                    value=tags
                    on:input=move |e| {
                        set_tags.set(event_target_value(&e));
                        set_has_unsaved_changes.set(true);
                    }
                    aria_label="Document tags"
                />
            </div>
        </div>
    }
}
```

#### 3.2.2. Component Features

**Editor Features:**
- Content-editable textarea for Markdown editing
- Real-time syntax highlighting (via WASM module)
- Live preview of rendered content
- Auto-save with configurable interval
- Manual save button
- Cancel button for discarding changes

**Formatting Toolbar:**
- Bold, italic, strikethrough
- Headings (H1-H6)
- Lists (ordered, unordered)
- Code blocks with language selection
- Links and images
- Tables

**Mobile Optimization:**
- Floating toolbar above virtual keyboard
- Debounced syntax highlighting to prevent cursor jump
- Touch-optimized interactions
- Responsive layout (single column on mobile)

### 3.3. DocumentList Component

The DocumentList component displays a list of documents with filtering and sorting capabilities.

#### 3.3.1. Component Interface

**TypeScript Interface:**
```typescript
/**
 * Props for DocumentList component.
 *
 * @property documents - The list of documents to display (reactive signal)
 * @property on_select - Callback invoked when a document is selected
 * @property on_delete - Callback invoked when a document is deleted
 * @property filter - Filter criteria for documents
 * @property sort_by - Sort field for documents
 * @property sort_order - Sort order (asc or desc)
 */
export interface DocumentListProps {
    documents: ReadSignal<Document[]>;
    on_select?: Callback<DocumentId>;
    on_delete?: Callback<DocumentId>;
    filter?: DocumentFilter;
    sort_by?: SortField;
    sort_order?: SortOrder;
}

export type DocumentFilter = {
    search?: string;
    tags?: string[];
    author?: string;
    date_range?: [Date, Date];
};

export type SortField = 'title' | 'created_at' | 'updated_at' | 'author';
export type SortOrder = 'asc' | 'desc';
```

**Rust Component Definition:**
```rust
/// Props for DocumentList component.
///
/// # Fields
///
/// * `documents` - The list of documents to display (reactive signal)
/// * `on_select` - Callback invoked when a document is selected
/// * `on_delete` - Callback invoked when a document is deleted
/// * `filter` - Filter criteria for documents
/// * `sort_by` - Sort field for documents
/// * `sort_order` - Sort order (asc or desc)
#[derive(Clone, Props)]
pub struct DocumentListProps {
    /// The list of documents to display (reactive signal)
    pub documents: ReadSignal<Vec<Document>>,
    
    /// Callback invoked when a document is selected
    #[prop(optional)]
    pub on_select: Option<Callback<DocumentId>>,
    
    /// Callback invoked when a document is deleted
    #[prop(optional)]
    pub on_delete: Option<Callback<DocumentId>>,
    
    /// Filter criteria for documents
    #[prop(optional)]
    pub filter: Option<DocumentFilter>,
    
    /// Sort field for documents
    #[prop(default = SortField::UpdatedAt)]
    pub sort_by: SortField,
    
    /// Sort order (asc or desc)
    #[prop(default = SortOrder::Desc)]
    pub sort_order: SortOrder,
}

/// DocumentList component displays a list of documents with filtering and sorting.
///
/// # Pre-conditions
///
/// * The `documents` signal must contain a valid vector of Documents
///
/// # Post-conditions
///
/// * Documents are filtered and sorted according to props
/// * Selected document is highlighted
///
/// # Invariants
///
/// * The displayed documents match the filtered and sorted documents signal
/// * The list updates reactively when documents signal changes
///
/// # Side Effects
///
/// * Invokes `on_select` callback when a document is clicked
/// * Invokes `on_delete` callback when delete button is clicked
#[component]
pub fn DocumentList(props: DocumentListProps) -> impl IntoView {
    let documents = props.documents;
    let filter = props.filter;
    let sort_by = props.sort_by;
    let sort_order = props.sort_order;
    
    // Local signal for selected document
    let (selected_document_id, set_selected_document_id) = create_signal(None::<DocumentId>);
    
    // Derived signal for filtered documents
    let filtered_documents = create_memo(move |_| {
        let docs = documents.get();
        let filter = filter.clone();
        
        docs.into_iter()
            .filter(|doc| {
                if let Some(ref filter) = filter {
                    // Apply search filter
                    if let Some(ref search) = filter.search {
                        if !doc.title.to_lowercase().contains(&search.to_lowercase())
                            && !doc.content.to_lowercase().contains(&search.to_lowercase()) {
                            return false;
                        }
                    }
                    
                    // Apply tags filter
                    if !filter.tags.is_empty() {
                        let doc_tags: std::collections::HashSet<_> =
                            doc.tags.iter().cloned().collect();
                        if !filter.tags.iter().all(|tag| doc_tags.contains(tag)) {
                            return false;
                        }
                    }
                    
                    // Apply author filter
                    if let Some(ref author) = filter.author {
                        if doc.author != *author {
                            return false;
                        }
                    }
                    
                    // Apply date range filter
                    if let Some((start, end)) = filter.date_range {
                        if doc.created_at < *start || doc.created_at > *end {
                            return false;
                        }
                    }
                }
                true
            })
            .collect::<Vec<_>>()
    });
    
    // Derived signal for sorted documents
    let sorted_documents = create_memo(move |_| {
        let mut docs = filtered_documents.get();
        
        docs.sort_by(|a, b| {
            let cmp = match sort_by {
                SortField::Title => a.title.cmp(&b.title),
                SortField::CreatedAt => a.created_at.cmp(&b.created_at),
                SortField::UpdatedAt => a.updated_at.cmp(&b.updated_at),
                SortField::Author => a.author.cmp(&b.author),
            };
            
            match sort_order {
                SortOrder::Asc => cmp,
                SortOrder::Desc => cmp.reverse(),
            }
        });
        
        docs
    });
    
    // Handle document selection
    let handle_select = move |doc_id: DocumentId| {
        set_selected_document_id.set(Some(doc_id.clone()));
        if let Some(ref on_select) = props.on_select {
            on_select.call(doc_id);
        }
    };
    
    // Handle document deletion
    let handle_delete = move |doc_id: DocumentId| {
        if let Some(ref on_delete) = props.on_delete {
            on_delete.call(doc_id);
        }
    };
    
    view! {
        <div class="document-list" role="list" aria_label="Document list">
            <Show
                when=move || sorted_documents.get().is_empty()
                fallback=|| view! {
                    <div class="document-list-items">
                        {move || {
                            sorted_documents.get()
                                .into_iter()
                                .map(|doc| {
                                    let doc_id = doc.id.clone();
                                    let is_selected = create_memo(move |_| {
                                        selected_document_id.get().as_ref() == Some(&doc_id)
                                    });
                                    
                                    view! {
                                        <DocumentCard
                                            document=doc
                                            selected=is_selected
                                            on_select=handle_select.clone()
                                            on_delete=handle_delete.clone()
                                        />
                                    }
                                })
                                .collect_view()
                        }}
                    </div>
                }
            >
                <div class="empty-state">
                    <p>"No documents found"</p>
                    <p class="empty-state-hint">
                        "Try adjusting your filters or create a new document"
                    </p>
                </div>
            </Show>
        </div>
    }
}
```

#### 3.3.2. DocumentCard Subcomponent

The DocumentCard component displays a single document in the list.

```rust
/// Props for DocumentCard component.
#[derive(Clone, Props)]
pub struct DocumentCardProps {
    pub document: Document,
    pub selected: bool,
    pub on_select: Callback<DocumentId>,
    pub on_delete: Callback<DocumentId>,
}

/// DocumentCard component displays a single document.
#[component]
pub fn DocumentCard(props: DocumentCardProps) -> impl IntoView {
    let document = props.document;
    let selected = props.selected;
    let on_select = props.on_select.clone();
    let on_delete = props.on_delete.clone();
    
    // Derived signal for formatted date
    let formatted_date = create_memo(move |_| {
        document.updated_at.format("%b %d, %Y")
    });
    
    // Derived signal for word count
    let word_count = create_memo(move |_| {
        document.content.split_whitespace().count()
    });
    
    // Handle card click
    let handle_click = move |_| {
        on_select.call(document.id.clone());
    };
    
    // Handle delete click (stop propagation to prevent card selection)
    let handle_delete_click = move |e: Event| {
        e.prevent_default();
        e.stop_propagation();
        on_delete.call(document.id.clone());
    };
    
    view! {
        <div
            class=format!(
                "document-card {}",
                if selected { "selected" } else { "" }
            )
            role="listitem"
            tabindex="0"
            aria_selected=selected
            on:click=handle_click
            on:keydown=move |e: KeyboardEvent| {
                if e.key() == "Enter" || e.key() == " " {
                    handle_click(());
                }
            }
        >
            <div class="document-card-header">
                <h3 class="document-card-title">{document.title.clone()}</h3>
                <button
                    class="document-card-delete"
                    aria_label="Delete document"
                    on:click=handle_delete_click
                >
                    "×"
                </button>
            </div>
            <p class="document-card-summary">
                {document.content.chars().take(100).collect::<String>()}
                {if document.content.len() > 100 { "..." } else { "" }}
            </p>
            <div class="document-card-footer">
                <span class="document-card-date">{formatted_date}</span>
                <span class="document-card-words">
                    {word_count} " words"
                </span>
            </div>
        </div>
    }
}
```

---

---

## 4. REPOSITORY COMPONENTS

Repository components provide functionality for managing Git repositories, viewing repository structure, and monitoring sync status. These components implement requirements from [REQ-WEB-005 through REQ-WEB-010](../.specs/04_future_state/reqs/web_requirements.md).

### 4.1. RepositoryList Component

The RepositoryList component displays a list of repositories with sync status indicators.

#### 4.1.1. Component Interface

**TypeScript Interface:**
```typescript
/**
 * Props for RepositoryList component.
 *
 * @property repositories - The list of repositories to display (reactive signal)
 * @property sync_status - Sync status for each repository (reactive signal)
 * @property on_select - Callback invoked when a repository is selected
 * @property on_sync - Callback invoked when sync is requested for a repository
 * @property on_add - Callback invoked when add repository is requested
 *
 * @constraint repositories signal must contain a valid vector of Repositories
 */
export interface RepositoryListProps {
    repositories: ReadSignal<Repository[]>;
    sync_status: ReadSignal<Map<RepositoryPath, SyncStatus>>;
    on_select?: Callback<RepositoryPath>;
    on_sync?: Callback<RepositoryPath>;
    on_add?: Callback<()>;
}
```

**Rust Component Definition:**
```rust
/// Props for RepositoryList component.
///
/// # Fields
///
/// * `repositories` - The list of repositories to display (reactive signal)
/// * `sync_status` - Sync status for each repository (reactive signal)
/// * `on_select` - Callback invoked when a repository is selected
/// * `on_sync` - Callback invoked when sync is requested for a repository
/// * `on_add` - Callback invoked when add repository is requested
#[derive(Clone, Props)]
pub struct RepositoryListProps {
    /// The list of repositories to display (reactive signal)
    pub repositories: ReadSignal<Vec<Repository>>,
    
    /// Sync status for each repository (reactive signal)
    pub sync_status: ReadSignal<std::collections::HashMap<RepositoryPath, SyncStatus>>,
    
    /// Callback invoked when a repository is selected
    #[prop(optional)]
    pub on_select: Option<Callback<RepositoryPath>>,
    
    /// Callback invoked when sync is requested for a repository
    #[prop(optional)]
    pub on_sync: Option<Callback<RepositoryPath>>,
    
    /// Callback invoked when add repository is requested
    #[prop(optional)]
    pub on_add: Option<Callback<()>>,
}

/// RepositoryList component displays a list of repositories with sync status.
///
/// # Pre-conditions
///
/// * The `repositories` signal must contain a valid vector of Repositories
///
/// # Post-conditions
///
/// * Repositories are displayed with sync status indicators
/// * Selected repository is highlighted
///
/// # Invariants
///
/// * The displayed repositories match the repositories signal
/// * Sync status indicators reflect the sync_status signal
///
/// # Side Effects
///
/// * Invokes `on_select` callback when a repository is clicked
/// * Invokes `on_sync` callback when sync button is clicked
/// * Invokes `on_add` callback when add button is clicked
#[component]
pub fn RepositoryList(props: RepositoryListProps) -> impl IntoView {
    let repositories = props.repositories;
    let sync_status = props.sync_status;
    
    // Local signal for selected repository
    let (selected_repository, set_selected_repository) = create_signal(None::<RepositoryPath>);
    
    // Handle repository selection
    let handle_select = move |repo_path: RepositoryPath| {
        set_selected_repository.set(Some(repo_path.clone()));
        if let Some(ref on_select) = props.on_select {
            on_select.call(repo_path);
        }
    };
    
    // Handle sync request
    let handle_sync = move |repo_path: RepositoryPath| {
        if let Some(ref on_sync) = props.on_sync {
            on_sync.call(repo_path);
        }
    };
    
    // Handle add repository
    let handle_add = move |_| {
        if let Some(ref on_add) = props.on_add {
            on_add.call(());
        }
    };
    
    view! {
        <div class="repository-list" role="list" aria_label="Repository list">
            <div class="repository-list-header">
                <h2>"Repositories"</h2>
                <Show
                    when=move || props.on_add.is_some()
                    fallback=|| view! { <div></div> }
                >
                    <button
                        class="btn-primary"
                        aria_label="Add repository"
                        on:click=handle_add
                    >
                        "+ Add Repository"
                    </button>
                </Show>
            </div>
            <Show
                when=move || repositories.get().is_empty()
                fallback=|| view! {
                    <div class="repository-list-items">
                        {move || {
                            repositories.get()
                                .into_iter()
                                .map(|repo| {
                                    let repo_path = repo.path.clone();
                                    let sync_status = create_memo(move |_| {
                                        sync_status.get().get(&repo_path).cloned()
                                            .unwrap_or(SyncStatus::Unknown)
                                    });
                                    let is_selected = create_memo(move |_| {
                                        selected_repository.get().as_ref() == Some(&repo_path)
                                    });
                                    
                                    view! {
                                        <RepositoryCard
                                            repository=repo
                                            sync_status=sync_status
                                            selected=is_selected
                                            on_select=handle_select.clone()
                                            on_sync=handle_sync.clone()
                                        />
                                    }
                                })
                                .collect_view()
                        }}
                    </div>
                }
            >
                <div class="empty-state">
                    <p>"No repositories configured"</p>
                    <p class="empty-state-hint">
                        "Add a repository to get started"
                    </p>
                </div>
            </Show>
        </div>
    }
}
```

#### 4.1.2. RepositoryCard Subcomponent

The RepositoryCard component displays a single repository with sync status.

```rust
/// Props for RepositoryCard component.
#[derive(Clone, Props)]
pub struct RepositoryCardProps {
    pub repository: Repository,
    pub sync_status: ReadSignal<SyncStatus>,
    pub selected: bool,
    pub on_select: Callback<RepositoryPath>,
    pub on_sync: Callback<RepositoryPath>,
}

/// RepositoryCard component displays a single repository.
#[component]
pub fn RepositoryCard(props: RepositoryCardProps) -> impl IntoView {
    let repository = props.repository;
    let selected = props.selected;
    let on_select = props.on_select.clone();
    let on_sync = props.on_sync.clone();
    
    // Derived signal for sync status
    let sync_status = props.sync_status;
    
    // Derived signal for formatted sync status
    let sync_status_text = create_memo(move |_| {
        match sync_status.get() {
            SyncStatus::Synced => "Synced".to_string(),
            SyncStatus::Syncing => "Syncing...".to_string(),
            SyncStatus::Error => "Sync Error".to_string(),
            SyncStatus::Unknown => "Unknown".to_string(),
        }
    });
    
    // Derived signal for sync status class
    let sync_status_class = create_memo(move |_| {
        match sync_status.get() {
            SyncStatus::Synced => "synced".to_string(),
            SyncStatus::Syncing => "syncing".to_string(),
            SyncStatus::Error => "error".to_string(),
            SyncStatus::Unknown => "unknown".to_string(),
        }
    });
    
    // Handle card click
    let handle_click = move |_| {
        on_select.call(repository.path.clone());
    };
    
    // Handle sync click (stop propagation to prevent card selection)
    let handle_sync_click = move |e: Event| {
        e.prevent_default();
        e.stop_propagation();
        on_sync.call(repository.path.clone());
    };
    
    view! {
        <div
            class=format!(
                "repository-card {}",
                if selected { "selected" } else { "" }
            )
            role="listitem"
            tabindex="0"
            aria_selected=selected
            on:click=handle_click
            on:keydown=move |e: KeyboardEvent| {
                if e.key() == "Enter" || e.key() == " " {
                    handle_click(());
                }
            }
        >
            <div class="repository-card-header">
                <h3 class="repository-card-name">{repository.name.clone()}</h3>
                <span
                    class=format!("repository-card-status {}", sync_status_class)
                    aria_label=format!("Sync status: {}", sync_status_text)
                >
                    {sync_status_text}
                </span>
            </div>
            <p class="repository-card-path">{repository.path.display()}</p>
            <div class="repository-card-footer">
                <span class="repository-card-branch">
                    "Branch: " {repository.branch.clone()}
                </span>
                <button
                    class="repository-card-sync"
                    aria_label="Sync repository"
                    on:click=handle_sync_click
                >
                    "Sync"
                </button>
            </div>
        </div>
    }
}
```

### 4.2. RepositoryTree Component

The RepositoryTree component displays a hierarchical tree view of repository structure.

#### 4.2.1. Component Interface

**TypeScript Interface:**
```typescript
/**
 * Props for RepositoryTree component.
 *
 * @property repository - The repository to display (reactive signal)
 * @property git_status - Git status for repository (reactive signal)
 * @property on_select - Callback invoked when a file/directory is selected
 * @property expanded_paths - Set of expanded paths (reactive signal)
 *
 * @constraint repository signal must contain a valid Repository
 */
export interface RepositoryTreeProps {
    repository: ReadSignal<Repository>;
    git_status: ReadSignal<Map<RepositoryPath, GitStatus>>;
    on_select?: Callback<RepositoryPath>;
    expanded_paths?: ReadSignal<Set<RepositoryPath>>;
}
```

**Rust Component Definition:**
```rust
/// Props for RepositoryTree component.
///
/// # Fields
///
/// * `repository` - The repository to display (reactive signal)
/// * `git_status` - Git status for repository (reactive signal)
/// * `on_select` - Callback invoked when a file/directory is selected
/// * `expanded_paths` - Set of expanded paths (reactive signal)
#[derive(Clone, Props)]
pub struct RepositoryTreeProps {
    /// The repository to display (reactive signal)
    pub repository: ReadSignal<Repository>,
    
    /// Git status for repository (reactive signal)
    pub git_status: ReadSignal<std::collections::HashMap<RepositoryPath, GitStatus>>,
    
    /// Callback invoked when a file/directory is selected
    #[prop(optional)]
    pub on_select: Option<Callback<RepositoryPath>>,
    
    /// Set of expanded paths (reactive signal)
    #[prop(optional)]
    pub expanded_paths: Option<ReadSignal<std::collections::HashSet<RepositoryPath>>>,
}

/// RepositoryTree component displays a hierarchical tree view of repository structure.
///
/// # Pre-conditions
///
/// * The `repository` signal must contain a valid Repository
///
/// # Post-conditions
///
/// * Repository structure is displayed as a tree
/// * Git status indicators are shown for modified files
///
/// # Invariants
///
/// * The displayed tree matches the repository structure
/// * Git status indicators reflect the git_status signal
///
/// # Side Effects
///
/// * Invokes `on_select` callback when a file/directory is clicked
/// * Updates expanded_paths when a directory is expanded/collapsed
#[component]
pub fn RepositoryTree(props: RepositoryTreeProps) -> impl IntoView {
    let repository = props.repository;
    let git_status = props.git_status;
    
    // Local signal for expanded paths
    let (expanded_paths, set_expanded_paths) = create_signal(
        props.expanded_paths
            .as_ref()
            .map(|s| s.get().clone())
            .unwrap_or_default()
    );
    
    // Local signal for selected path
    let (selected_path, set_selected_path) = create_signal(None::<RepositoryPath>);
    
    // Derived signal for repository structure
    let tree_structure = create_memo(move |_| {
        let repo = repository.get();
        build_tree_structure(&repo.path)
    });
    
    // Handle path selection
    let handle_select = move |path: RepositoryPath| {
        set_selected_path.set(Some(path.clone()));
        if let Some(ref on_select) = props.on_select {
            on_select.call(path);
        }
    };
    
    // Handle path expansion toggle
    let handle_toggle_expand = move |path: RepositoryPath| {
        set_expanded_paths.update(|paths| {
            if paths.contains(&path) {
                paths.remove(&path);
            } else {
                paths.insert(path);
            }
        });
    };
    
    // Recursively render tree nodes
    let render_tree_node = |node: TreeNode| {
        let path = node.path.clone();
        let is_expanded = create_memo(move |_| {
            expanded_paths.get().contains(&path)
        });
        let is_selected = create_memo(move |_| {
            selected_path.get().as_ref() == Some(&path)
        });
        let git_status = create_memo(move |_| {
            git_status.get().get(&path).cloned()
        });
        
        view! {
            <TreeNode
                node=node
                is_expanded=is_expanded
                is_selected=is_selected
                git_status=git_status
                on_select=handle_select.clone()
                on_toggle_expand=handle_toggle_expand.clone()
            />
        }
    };
    
    view! {
        <div class="repository-tree" role="tree" aria_label="Repository structure">
            <Show
                when=move || tree_structure.get().is_empty()
                fallback=|| view! {
                    <div class="repository-tree-nodes">
                        {move || {
                            tree_structure.get()
                                .into_iter()
                                .map(render_tree_node)
                                .collect_view()
                        }}
                    </div>
                }
            >
                <div class="empty-state">
                    <p>"Repository is empty"</p>
                </div>
            </Show>
        </div>
    }
}
```

#### 4.2.2. TreeNode Subcomponent

The TreeNode component displays a single node in the repository tree.

```rust
/// Props for TreeNode component.
#[derive(Clone, Props)]
pub struct TreeNodeProps {
    pub node: TreeNode,
    pub is_expanded: ReadSignal<bool>,
    pub is_selected: ReadSignal<bool>,
    pub git_status: ReadSignal<Option<GitStatus>>,
    pub on_select: Callback<RepositoryPath>,
    pub on_toggle_expand: Callback<RepositoryPath>,
}

/// TreeNode component displays a single node in the repository tree.
#[component]
pub fn TreeNode(props: TreeNodeProps) -> impl IntoView {
    let node = props.node;
    let is_expanded = props.is_expanded;
    let is_selected = props.is_selected;
    let git_status = props.git_status;
    let on_select = props.on_select.clone();
    let on_toggle_expand = props.on_toggle_expand.clone();
    
    // Handle node click
    let handle_click = move |_| {
        if node.node_type == NodeType::Directory {
            on_toggle_expand.call(node.path.clone());
        } else {
            on_select.call(node.path.clone());
        }
    };
    
    // Handle node double click
    let handle_double_click = move |_| {
        on_select.call(node.path.clone());
    };
    
    // Derived signal for git status indicator
    let git_status_indicator = create_memo(move |_| {
        match git_status.get() {
            Some(GitStatus::Modified) => Some("modified".to_string()),
            Some(GitStatus::Added) => Some("added".to_string()),
            Some(GitStatus::Deleted) => Some("deleted".to_string()),
            Some(GitStatus::Conflicted) => Some("conflicted".to_string()),
            None => None,
        }
    });
    
    // Derived signal for node icon
    let node_icon = create_memo(move |_| {
        match node.node_type {
            NodeType::Directory => {
                if is_expanded.get() { "folder-open".to_string() }
                else { "folder".to_string() }
            },
            NodeType::File => "file".to_string(),
        }
    });
    
    view! {
        <div
            class=format!(
                "tree-node {} {}",
                if is_selected.get() { "selected" } else { "" },
                if node.node_type == NodeType::Directory { "directory" } else { "file" }
            )
            role="treeitem"
            aria_expanded=is_expanded
            on:click=handle_click
            on:dblclick=handle_double_click
        >
            <div class="tree-node-content">
                <span class="tree-node-icon">{node_icon}</span>
                <span class="tree-node-name">{node.name.clone()}</span>
                <Show
                    when=move || git_status_indicator.get().is_some()
                    fallback=|| view! { <span></span> }
                >
                    <span
                        class=format!("tree-node-status {}", git_status_indicator.get().unwrap())
                        aria_label=format!("Git status: {}", git_status_indicator.get().unwrap())
                    >
                    "●"
                    </span>
                </Show>
            </div>
            <Show
                when=move || node.node_type == NodeType::Directory && is_expanded.get()
                fallback=|| view! { <div></div> }
            >
                <div class="tree-node-children">
                    {move || {
                        node.children
                            .into_iter()
                            .map(|child| {
                                let child_props = TreeNodeProps {
                                    node: child,
                                    is_expanded: props.is_expanded,
                                    is_selected: props.is_selected,
                                    git_status: props.git_status,
                                    on_select: props.on_select.clone(),
                                    on_toggle_expand: props.on_toggle_expand.clone(),
                                };
                                view! { <TreeNode ..child_props /> }
                            })
                            .collect_view()
                    }}
                </div>
            </Show>
        </div>
    }
}
```

### 4.3. RepositorySync Component

The RepositorySync component displays sync progress and provides sync controls.

#### 4.3.1. Component Interface

**TypeScript Interface:**
```typescript
/**
 * Props for RepositorySync component.
 *
 * @property repository - The repository being synced (reactive signal)
 * @property sync_status - Current sync status (reactive signal)
 * @property sync_progress - Sync progress (0-100) (reactive signal)
 * @property on_sync - Callback invoked when sync is requested
 * @property on_cancel - Callback invoked when sync is cancelled
 *
 * @constraint repository signal must contain a valid Repository
 */
export interface RepositorySyncProps {
    repository: ReadSignal<Repository>;
    sync_status: ReadSignal<SyncStatus>;
    sync_progress: ReadSignal<number>;
    on_sync?: Callback<()>;
    on_cancel?: Callback<()>;
}
```

**Rust Component Definition:**
```rust
/// Props for RepositorySync component.
///
/// # Fields
///
/// * `repository` - The repository being synced (reactive signal)
/// * `sync_status` - Current sync status (reactive signal)
/// * `sync_progress` - Sync progress (0-100) (reactive signal)
/// * `on_sync` - Callback invoked when sync is requested
/// * `on_cancel` - Callback invoked when sync is cancelled
#[derive(Clone, Props)]
pub struct RepositorySyncProps {
    /// The repository being synced (reactive signal)
    pub repository: ReadSignal<Repository>,
    
    /// Current sync status (reactive signal)
    pub sync_status: ReadSignal<SyncStatus>,
    
    /// Sync progress (0-100) (reactive signal)
    pub sync_progress: ReadSignal<u8>,
    
    /// Callback invoked when sync is requested
    #[prop(optional)]
    pub on_sync: Option<Callback<()>>,
    
    /// Callback invoked when sync is cancelled
    #[prop(optional)]
    pub on_cancel: Option<Callback<()>>,
}

/// RepositorySync component displays sync progress and provides sync controls.
///
/// # Pre-conditions
///
/// * The `repository` signal must contain a valid Repository
///
/// # Post-conditions
///
/// * Sync status is displayed with progress indicator
/// * Sync controls are available based on sync status
///
/// # Invariants
///
/// * The displayed sync status matches the sync_status signal
/// * The progress indicator reflects the sync_progress signal
///
/// # Side Effects
///
/// * Invokes `on_sync` callback when sync is requested
/// * Invokes `on_cancel` callback when sync is cancelled
#[component]
pub fn RepositorySync(props: RepositorySyncProps) -> impl IntoView {
    let repository = props.repository;
    let sync_status = props.sync_status;
    let sync_progress = props.sync_progress;
    
    // Derived signal for sync status text
    let sync_status_text = create_memo(move |_| {
        match sync_status.get() {
            SyncStatus::Synced => "Up to date".to_string(),
            SyncStatus::Syncing => "Syncing...".to_string(),
            SyncStatus::Error => "Sync failed".to_string(),
            SyncStatus::Unknown => "Unknown".to_string(),
        }
    });
    
    // Derived signal for can sync
    let can_sync = create_memo(move |_| {
        matches!(sync_status.get(), SyncStatus::Synced | SyncStatus::Error | SyncStatus::Unknown)
    });
    
    // Derived signal for can cancel
    let can_cancel = create_memo(move |_| {
        matches!(sync_status.get(), SyncStatus::Syncing)
    });
    
    // Handle sync request
    let handle_sync = move |_| {
        if let Some(ref on_sync) = props.on_sync {
            on_sync.call(());
        }
    };
    
    // Handle cancel request
    let handle_cancel = move |_| {
        if let Some(ref on_cancel) = props.on_cancel {
            on_cancel.call(());
        }
    };
    
    view! {
        <div class="repository-sync">
            <div class="repository-sync-header">
                <h3>"Sync Status"</h3>
                <span class="repository-sync-status">{sync_status_text}</span>
            </div>
            <Show
                when=move || matches!(sync_status.get(), SyncStatus::Syncing)
                fallback=|| view! { <div></div> }
            >
                <div class="repository-sync-progress">
                    <progress
                        class="repository-sync-progress-bar"
                        value=move || sync_progress.get() as f64
                        max=100.0
                        aria_label=format!("Sync progress: {}%", sync_progress)
                    />
                    <span class="repository-sync-progress-text">
                        {move || format!("{}%", sync_progress.get())}
                    </span>
                </div>
            </Show>
            <div class="repository-sync-controls">
                <Show
                    when=move || can_sync()
                    fallback=|| view! { <div></div> }
                >
                    <button
                        class="btn-primary"
                        aria_label="Sync repository"
                        on:click=handle_sync
                    >
                        "Sync Now"
                    </button>
                </Show>
                <Show
                    when=move || can_cancel()
                    fallback=|| view! { <div></div> }
                >
                    <button
                        class="btn-secondary"
                        aria_label="Cancel sync"
                        on:click=handle_cancel
                    >
                        "Cancel"
                    </button>
                </Show>
            </div>
            <div class="repository-sync-info">
                <p>"Repository: " {move || repository.get().name}</p>
                <p>"Branch: " {move || repository.get().branch}</p>
                <p>"Path: " {move || repository.get().path.display()}</p>
            </div>
        </div>
    }
}
```

---

---

## 5. SEARCH COMPONENTS

Search components provide functionality for searching documents, repositories, and other content within the Tachyon system. These components implement requirements from [REQ-WEB-021 through REQ-WEB-025](../.specs/04_future_state/reqs/web_requirements.md).

### 5.1. SearchInterface Component

The SearchInterface component provides a comprehensive search interface with filters and advanced search options.

#### 5.1.1. Component Interface

**TypeScript Interface:**
```typescript
/**
 * Props for SearchInterface component.
 *
 * @property query - Current search query (reactive signal)
 * @property results - Search results (reactive signal)
 * @property is_searching - Whether search is in progress (reactive signal)
 * @property on_search - Callback invoked when search is triggered
 * @property on_select - Callback invoked when a result is selected
 * @property filters - Search filters (reactive signal)
 * @property advanced_mode - Whether advanced search mode is enabled
 *
 * @constraint query signal must be a valid string
 * @constraint on_search callback is required
 */
export interface SearchInterfaceProps {
    query: ReadSignal<string>;
    results: ReadSignal<SearchResult[]>;
    is_searching: ReadSignal<boolean>;
    on_search: Callback<SearchQuery>;
    on_select?: Callback<SearchResult>;
    filters?: ReadSignal<SearchFilters>;
    advanced_mode?: boolean;
}

export interface SearchQuery {
    text: string;
    filters: SearchFilters;
    advanced_options?: AdvancedSearchOptions;
}

export interface SearchFilters {
    tags?: string[];
    author?: string;
    date_range?: [Date, Date];
    repository?: string;
}

export interface AdvancedSearchOptions {
    use_boolean_operators?: boolean;
    phrase_matching?: boolean;
    fuzzy_search?: boolean;
    case_sensitive?: boolean;
}
```

**Rust Component Definition:**
```rust
/// Props for SearchInterface component.
///
/// # Fields
///
/// * `query` - Current search query (reactive signal)
/// * `results` - Search results (reactive signal)
/// * `is_searching` - Whether search is in progress (reactive signal)
/// * `on_search` - Callback invoked when search is triggered
/// * `on_select` - Callback invoked when a result is selected
/// * `filters` - Search filters (reactive signal)
/// * `advanced_mode` - Whether advanced search mode is enabled
#[derive(Clone, Props)]
pub struct SearchInterfaceProps {
    /// Current search query (reactive signal)
    pub query: ReadSignal<String>,
    
    /// Search results (reactive signal)
    pub results: ReadSignal<Vec<SearchResult>>,
    
    /// Whether search is in progress (reactive signal)
    pub is_searching: ReadSignal<bool>,
    
    /// Callback invoked when search is triggered
    pub on_search: Callback<SearchQuery>,
    
    /// Callback invoked when a result is selected
    #[prop(optional)]
    pub on_select: Option<Callback<SearchResult>>,
    
    /// Search filters (reactive signal)
    #[prop(optional)]
    pub filters: Option<ReadSignal<SearchFilters>>,
    
    /// Whether advanced search mode is enabled
    #[prop(default = false)]
    pub advanced_mode: bool,
}

/// SearchInterface component provides a comprehensive search interface.
///
/// # Pre-conditions
///
/// * The `query` signal must be a valid string
/// * The `on_search` callback is required
///
/// # Post-conditions
///
/// * Search results are displayed with relevance ranking
/// * Filters are applied to search results
///
/// # Invariants
///
/// * The displayed results match the results signal
/// * The search input matches the query signal
///
/// # Side Effects
///
/// * Invokes `on_search` callback when search is triggered
/// * Invokes `on_select` callback when a result is selected
#[component]
pub fn SearchInterface(props: SearchInterfaceProps) -> impl IntoView {
    let query = props.query;
    let results = props.results;
    let is_searching = props.is_searching;
    let filters = props.filters;
    let advanced_mode = props.advanced_mode;
    
    // Local signal for search input
    let (search_input, set_search_input) = create_signal(query.get());
    let (show_advanced, set_show_advanced) = create_signal(advanced_mode);
    
    // Local signal for selected result
    let (selected_result, set_selected_result) = create_signal(None::<SearchResult>);
    
    // Derived signal for has results
    let has_results = create_memo(move |_| !results.get().is_empty());
    
    // Derived signal for result count
    let result_count = create_memo(move |_| results.get().len());
    
    // Derived signal for can search
    let can_search = create_memo(move |_| {
        !search_input.get().trim().is_empty()
    });
    
    // Handle search input change
    let handle_input_change = move |e: Event| {
        let value = event_target_value(&e);
        set_search_input.set(value);
    };
    
    // Handle search submit
    let handle_search = move |_| {
        let search_query = SearchQuery {
            text: search_input.get().trim().to_string(),
            filters: filters
                .as_ref()
                .map(|f| f.get().clone())
                .unwrap_or_default(),
            advanced_options: if show_advanced.get() {
                Some(AdvancedSearchOptions {
                    use_boolean_operators: true,
                    phrase_matching: false,
                    fuzzy_search: true,
                    case_sensitive: false,
                })
            } else {
                None
            },
        };
        props.on_search.call(search_query);
    };
    
    // Handle result selection
    let handle_select = move |result: SearchResult| {
        set_selected_result.set(Some(result.clone()));
        if let Some(ref on_select) = props.on_select {
            on_select.call(result);
        }
    };
    
    // Handle advanced mode toggle
    let handle_toggle_advanced = move |_| {
        set_show_advanced.update(|v| !v);
    };
    
    view! {
        <div class="search-interface">
            <div class="search-input-container">
                <div class="search-input-wrapper">
                    <input
                        type="text"
                        class="search-input"
                        placeholder="Search documents..."
                        value=search_input
                        on:input=handle_input_change
                        on:keydown=move |e: KeyboardEvent| {
                            if e.key() == "Enter" && can_search() {
                                handle_search(());
                            }
                        }
                        aria_label="Search documents"
                    />
                    <button
                        class="search-button"
                        aria_label="Search"
                        disabled=move || !can_search()
                        on:click=handle_search
                    >
                        <Show
                            when=is_searching
                            fallback=|| view! {
                                <span class="search-icon">"🔍"</span>
                            }
                        >
                            <span class="search-spinner">"⟳"</span>
                        </Show>
                    </button>
                </div>
                <button
                    class=format!(
                        "search-advanced-toggle {}",
                        if show_advanced.get() { "active" } else { "" }
                    )
                    aria_label="Toggle advanced search"
                    on:click=handle_toggle_advanced
                >
                    "Advanced"
                </button>
            </div>
            <Show
                when=move || show_advanced()
                fallback=|| view! { <div></div> }
            >
                <div class="search-filters">
                    <SearchFilters filters=filters />
                </div>
            </Show>
            <div class="search-results-container">
                <Show
                    when=move || is_searching.get()
                    fallback=|| view! { <div></div> }
                >
                    <div class="search-loading">
                        <div class="search-spinner">"⟳"</div>
                        <p>"Searching..."</p>
                    </div>
                </Show>
                <Show
                    when=move || !is_searching.get() && !has_results() && !search_input.get().trim().is_empty()
                    fallback=|| view! { <div></div> }
                >
                    <div class="search-empty">
                        <p>"No results found"</p>
                        <p class="search-empty-hint">
                            "Try adjusting your search terms or filters"
                        </p>
                    </div>
                </Show>
                <Show
                    when=move || has_results()
                    fallback=|| view! { <div></div> }
                >
                    <div class="search-results-header">
                        <p>
                            {move || format!("{} results found", result_count())}
                        </p>
                    </div>
                    <div class="search-results" role="list" aria_label="Search results">
                        {move || {
                            results.get()
                                .into_iter()
                                .map(|result| {
                                    let result_clone = result.clone();
                                    let is_selected = create_memo(move |_| {
                                        selected_result.get().as_ref() == Some(&result_clone)
                                    });
                                    
                                    view! {
                                        <SearchResultCard
                                            result=result
                                            selected=is_selected
                                            on_select=handle_select.clone()
                                        />
                                    }
                                })
                                .collect_view()
                        }}
                    </div>
                </Show>
            </div>
        </div>
    }
}
```

#### 5.1.2. SearchFilters Component

The SearchFilters component provides filter controls for search queries.

```rust
/// Props for SearchFilters component.
#[derive(Clone, Props)]
pub struct SearchFiltersProps {
    pub filters: ReadSignal<SearchFilters>,
}

/// SearchFilters component provides filter controls.
#[component]
pub fn SearchFilters(props: SearchFiltersProps) -> impl IntoView {
    let filters = props.filters;
    
    // Local signals for filter values
    let (tag_input, set_tag_input) = create_signal(String::new());
    let (author_input, set_author_input) = create_signal(String::new());
    let (repository_input, set_repository_input) = create_signal(String::new());
    
    // Local signals for date range
    let (start_date, set_start_date) = create_signal(None::<NaiveDate>);
    let (end_date, set_end_date) = create_signal(None::<NaiveDate>);
    
    // Handle tag add
    let handle_add_tag = move |_| {
        let tag = tag_input.get().trim().to_string();
        if !tag.is_empty() {
            filters.update(|f| {
                f.tags.get_or_insert_with(Vec::new).push(tag);
            });
            set_tag_input.set(String::new());
        }
    };
    
    // Handle tag remove
    let handle_remove_tag = move |tag: String| {
        filters.update(|f| {
            if let Some(ref mut tags) = f.tags {
                tags.retain(|t| t != &tag);
            }
        });
    };
    
    // Handle author change
    let handle_author_change = move |e: Event| {
        let value = event_target_value(&e);
        filters.update(|f| {
            if value.trim().is_empty() {
                f.author = None;
            } else {
                f.author = Some(value.trim().to_string());
            }
        });
    };
    
    // Handle repository change
    let handle_repository_change = move |e: Event| {
        let value = event_target_value(&e);
        filters.update(|f| {
            if value.trim().is_empty() {
                f.repository = None;
            } else {
                f.repository = Some(value.trim().to_string());
            }
        });
    };
    
    // Handle date range change
    let handle_date_range_change = move |start: Option<NaiveDate>, end: Option<NaiveDate>| {
        filters.update(|f| {
            if start.is_some() || end.is_some() {
                f.date_range = Some((start, end));
            } else {
                f.date_range = None;
            }
        });
    };
    
    view! {
        <div class="search-filters">
            <div class="search-filter-group">
                <label class="search-filter-label">"Tags"</label>
                <div class="search-filter-inputs">
                    <input
                        type="text"
                        class="search-filter-input"
                        placeholder="Add tag..."
                        value=tag_input
                        on:input=move |e| set_tag_input.set(event_target_value(&e))
                        on:keydown=move |e: KeyboardEvent| {
                            if e.key() == "Enter" && !tag_input.get().trim().is_empty() {
                                handle_add_tag(());
                            }
                        }
                        aria_label="Add tag"
                    />
                    <button
                        class="search-filter-add"
                        aria_label="Add tag"
                        on:click=handle_add_tag
                    >
                        "+"
                    </button>
                </div>
                <div class="search-filter-tags">
                    {move || {
                        filters.get().tags
                            .as_ref()
                            .map(|tags| {
                                tags.iter()
                                    .map(|tag| {
                                        let tag_clone = tag.clone();
                                        view! {
                                            <span class="search-filter-tag">
                                                {tag_clone}
                                                <button
                                                    class="search-filter-tag-remove"
                                                    aria_label=format!("Remove tag: {}", tag)
                                                    on:click=move |_| handle_remove_tag(tag_clone.clone())
                                                >
                                                    "×"
                                                </button>
                                            </span>
                                        }
                                    })
                                    .collect_view()
                            })
                            .unwrap_or(view! { <div></div> })
                    }}
                </div>
            </div>
            <div class="search-filter-group">
                <label class="search-filter-label">"Author"</label>
                <input
                    type="text"
                    class="search-filter-input"
                    placeholder="Filter by author..."
                    value=move || filters.get().author.clone().unwrap_or_default()
                    on:input=handle_author_change
                    aria_label="Filter by author"
                />
            </div>
            <div class="search-filter-group">
                <label class="search-filter-label">"Repository"</label>
                <input
                    type="text"
                    class="search-filter-input"
                    placeholder="Filter by repository..."
                    value=move || filters.get().repository.clone().unwrap_or_default()
                    on:input=handle_repository_change
                    aria_label="Filter by repository"
                />
            </div>
            <div class="search-filter-group">
                <label class="search-filter-label">"Date Range"</label>
                <div class="search-filter-dates">
                    <input
                        type="date"
                        class="search-filter-date"
                        placeholder="Start date"
                        on:change=move |e| {
                            let value = event_target_value(&e);
                            let date = if value.is_empty() {
                                None
                            } else {
                                NaiveDate::parse_from_str(&value, "%Y-%m-%d").ok()
                            };
                            set_start_date.set(date);
                            handle_date_range_change(date, end_date.get());
                        }
                        aria_label="Start date"
                    />
                    <span>"to"</span>
                    <input
                        type="date"
                        class="search-filter-date"
                        placeholder="End date"
                        on:change=move |e| {
                            let value = event_target_value(&e);
                            let date = if value.is_empty() {
                                None
                            } else {
                                NaiveDate::parse_from_str(&value, "%Y-%m-%d").ok()
                            };
                            set_end_date.set(date);
                            handle_date_range_change(start_date.get(), date);
                        }
                        aria_label="End date"
                    />
                </div>
            </div>
        </div>
    }
}
```

### 5.2. SearchResultCard Component

The SearchResultCard component displays a single search result with highlighting.

#### 5.2.1. Component Interface

**TypeScript Interface:**
```typescript
/**
 * Props for SearchResultCard component.
 *
 * @property result - The search result to display
 * @property selected - Whether this result is selected
 * @property on_select - Callback invoked when result is selected
 *
 * @constraint result must be a valid SearchResult
 */
export interface SearchResultCardProps {
    result: SearchResult;
    selected: boolean;
    on_select: Callback<SearchResult>;
}
```

**Rust Component Definition:**
```rust
/// Props for SearchResultCard component.
#[derive(Clone, Props)]
pub struct SearchResultCardProps {
    pub result: SearchResult,
    pub selected: bool,
    pub on_select: Callback<SearchResult>,
}

/// SearchResultCard component displays a single search result.
#[component]
pub fn SearchResultCard(props: SearchResultCardProps) -> impl IntoView {
    let result = props.result;
    let selected = props.selected;
    let on_select = props.on_select.clone();
    
    // Derived signal for formatted date
    let formatted_date = create_memo(move |_| {
        result.created_at.format("%b %d, %Y")
    });
    
    // Derived signal for highlighted title
    let highlighted_title = create_memo(move |_| {
        highlight_text(&result.title, &result.highlight_terms)
    });
    
    // Derived signal for highlighted snippet
    let highlighted_snippet = create_memo(move |_| {
        highlight_text(&result.snippet, &result.highlight_terms)
    });
    
    // Handle card click
    let handle_click = move |_| {
        on_select.call(result.clone());
    };
    
    view! {
        <div
            class=format!(
                "search-result-card {}",
                if selected { "selected" } else { "" }
            )
            role="listitem"
            tabindex="0"
            aria_selected=selected
            on:click=handle_click
            on:keydown=move |e: KeyboardEvent| {
                if e.key() == "Enter" || e.key() == " " {
                    handle_click(());
                }
            }
        >
            <div class="search-result-header">
                <h3 class="search-result-title" inner_html=highlighted_title />
                <span class="search-result-score">
                    {move || format!("Relevance: {:.1}", result.score)}
                </span>
            </div>
            <p class="search-result-snippet" inner_html=highlighted_snippet />
            <div class="search-result-meta">
                <span class="search-result-path">{result.path.display()}</span>
                <span class="search-result-date">{formatted_date}</span>
                <Show when=move || !result.tags.is_empty()>
                    fallback=|| view! { <span></span> }
                >
                    <span class="search-result-tags">
                        {move || {
                            result.tags.iter()
                                .map(|tag| view! {
                                    <span class="search-result-tag">{tag.clone()}</span>
                                })
                                .collect_view()
                        }}
                    </span>
                </Show>
            </div>
        </div>
    }
}
```

### 5.3. SearchHistory Component

The SearchHistory component displays and manages search history.

#### 5.3.1. Component Interface

**TypeScript Interface:**
```typescript
/**
 * Props for SearchHistory component.
 *
 * @property history - Search history (reactive signal)
 * @property on_select - Callback invoked when a history item is selected
 * @property on_clear - Callback invoked when history is cleared
 *
 * @constraint history signal must contain a valid vector of SearchQuery
 */
export interface SearchHistoryProps {
    history: ReadSignal<SearchQuery[]>;
    on_select?: Callback<SearchQuery>;
    on_clear?: Callback<()>;
}
```

**Rust Component Definition:**
```rust
/// Props for SearchHistory component.
#[derive(Clone, Props)]
pub struct SearchHistoryProps {
    pub history: ReadSignal<Vec<SearchQuery>>,
    #[prop(optional)]
    pub on_select: Option<Callback<SearchQuery>>,
    #[prop(optional)]
    pub on_clear: Option<Callback<()>>,
}

/// SearchHistory component displays and manages search history.
#[component]
pub fn SearchHistory(props: SearchHistoryProps) -> impl IntoView {
    let history = props.history;
    
    // Handle history item selection
    let handle_select = move |query: SearchQuery| {
        if let Some(ref on_select) = props.on_select {
            on_select.call(query);
        }
    };
    
    // Handle history clear
    let handle_clear = move |_| {
        if let Some(ref on_clear) = props.on_clear {
            on_clear.call(());
        }
    };
    
    view! {
        <div class="search-history">
            <div class="search-history-header">
                <h3>"Recent Searches"</h3>
                <Show
                    when=move || props.on_clear.is_some()
                    fallback=|| view! { <div></div> }
                >
                    <button
                        class="search-history-clear"
                        aria_label="Clear search history"
                        on:click=handle_clear
                    >
                        "Clear"
                    </button>
                </Show>
            </div>
            <Show
                when=move || history.get().is_empty()
                fallback=|| view! {
                    <div class="search-history-items">
                        {move || {
                            history.get()
                                .into_iter()
                                .enumerate()
                                .map(|(index, query)| {
                                    let query_clone = query.clone();
                                    view! {
                                        <div
                                            class="search-history-item"
                                            role="button"
                                            tabindex="0"
                                            on:click=move |_| handle_select(query_clone.clone())
                                            on:keydown=move |e: KeyboardEvent| {
                                                if e.key() == "Enter" || e.key() == " " {
                                                    handle_select(query_clone.clone());
                                                }
                                            }
                                        >
                                            <span class="search-history-text">
                                                {query_clone.text}
                                            </span>
                                            <span class="search-history-time">
                                                {move || format!("{} ago", format_time_ago(index))}
                                            </span>
                                        </div>
                                    }
                                })
                                .collect_view()
                        }}
                    </div>
                }
            >
                <div class="empty-state">
                    <p>"No recent searches"</p>
                </div>
            </Show>
        </div>
    }
}
```

---

---

## 6. USER COMPONENTS

User components provide functionality for user profile management, authentication, and user preferences. These components implement requirements from [REQ-WEB-014](../.specs/04_future_state/reqs/web_requirements.md).

### 6.1. UserProfile Component

The UserProfile component displays user information and provides profile management options.

#### 6.1.1. Component Interface

**TypeScript Interface:**
```typescript
/**
 * Props for UserProfile component.
 *
 * @property user - The user to display (reactive signal)
 * @property on_edit - Callback invoked when profile edit is requested
 * @property on_settings - Callback invoked when settings are requested
 * @property on_logout - Callback invoked when logout is requested
 *
 * @constraint user signal must contain a valid User
 */
export interface UserProfileProps {
    user: ReadSignal<User>;
    on_edit?: Callback<UserId>;
    on_settings?: Callback<()>;
    on_logout?: Callback<()>;
}
```

**Rust Component Definition:**
```rust
/// Props for UserProfile component.
///
/// # Fields
///
/// * `user` - The user to display (reactive signal)
/// * `on_edit` - Callback invoked when profile edit is requested
/// * `on_settings` - Callback invoked when settings are requested
/// * `on_logout` - Callback invoked when logout is requested
#[derive(Clone, Props)]
pub struct UserProfileProps {
    /// The user to display (reactive signal)
    pub user: ReadSignal<User>,
    
    /// Callback invoked when profile edit is requested
    #[prop(optional)]
    pub on_edit: Option<Callback<UserId>>,
    
    /// Callback invoked when settings are requested
    #[prop(optional)]
    pub on_settings: Option<Callback<()>>,
    
    /// Callback invoked when logout is requested
    #[prop(optional)]
    pub on_logout: Option<Callback<()>>,
}

/// UserProfile component displays user information.
///
/// # Pre-conditions
///
/// * The `user` signal must contain a valid User
///
/// # Post-conditions
///
/// * User information is displayed with avatar and details
/// * Profile actions are available based on permissions
///
/// # Invariants
///
/// * The displayed user matches the user signal
///
/// # Side Effects
///
/// * Invokes `on_edit` callback when edit button is clicked
/// * Invokes `on_settings` callback when settings button is clicked
/// * Invokes `on_logout` callback when logout button is clicked
#[component]
pub fn UserProfile(props: UserProfileProps) -> impl IntoView {
    let user = props.user;
    let on_edit = props.on_edit.clone();
    let on_settings = props.on_settings.clone();
    let on_logout = props.on_logout.clone();
    
    // Derived signal for formatted join date
    let formatted_join_date = create_memo(move |_| {
        user.get().created_at.format("%B %Y")
    });
    
    // Handle edit click
    let handle_edit = move |_| {
        if let Some(ref on_edit) = props.on_edit {
            on_edit.call(user.get().id.clone());
        }
    };
    
    // Handle settings click
    let handle_settings = move |_| {
        if let Some(ref on_settings) = props.on_settings {
            on_settings.call(());
        }
    };
    
    // Handle logout click
    let handle_logout = move |_| {
        if let Some(ref on_logout) = props.on_logout {
            on_logout.call(());
        }
    };
    
    view! {
        <div class="user-profile">
            <div class="user-profile-header">
                <div class="user-avatar">
                    <img
                        src=move || format!("/api/users/{}/avatar", user.get().id)
                        alt=format!("Avatar for {}", user.get().username)
                        loading="lazy"
                    />
                </div>
                <div class="user-info">
                    <h2 class="user-name">{move || user.get().name.clone()}</h2>
                    <p class="user-username">@{move || user.get().username.clone()}</p>
                </div>
            </div>
            <div class="user-profile-details">
                <div class="user-detail">
                    <span class="user-detail-label">"Email:"</span>
                    <span class="user-detail-value">{move || user.get().email.clone()}</span>
                </div>
                <div class="user-detail">
                    <span class="user-detail-label">"Role:"</span>
                    <span class="user-detail-value">{move || format!("{:?}", user.get().role)}</span>
                </div>
                <div class="user-detail">
                    <span class="user-detail-label">"Member since:"</span>
                    <span class="user-detail-value">{formatted_join_date}</span>
                </div>
            </div>
            <div class="user-profile-actions">
                <Show
                    when=move || props.on_edit.is_some()
                    fallback=|| view! { <div></div> }
                >
                    <button
                        class="btn-secondary"
                        aria_label="Edit profile"
                        on:click=handle_edit
                    >
                        "Edit Profile"
                    </button>
                </Show>
                <Show
                    when=move || props.on_settings.is_some()
                    fallback=|| view! { <div></div> }
                >
                    <button
                        class="btn-secondary"
                        aria_label="Settings"
                        on:click=handle_settings
                    >
                        "Settings"
                    </button>
                </Show>
                <Show
                    when=move || props.on_logout.is_some()
                    fallback=|| view! { <div></div> }
                >
                    <button
                        class="btn-danger"
                        aria_label="Logout"
                        on:click=handle_logout
                    >
                        "Logout"
                    </button>
                </Show>
            </div>
        </div>
    }
}
```

### 6.2. UserMenu Component

The UserMenu component provides a dropdown menu for user actions.

#### 6.2.1. Component Interface

**TypeScript Interface:**
```typescript
/**
 * Props for UserMenu component.
 *
 * @property user - The current user (reactive signal)
 * @property on_profile - Callback invoked when profile is selected
 * @property on_settings - Callback invoked when settings are selected
 * @property on_logout - Callback invoked when logout is selected
 *
 * @constraint user signal must contain a valid User
 */
export interface UserMenuProps {
    user: ReadSignal<User>;
    on_profile?: Callback<()>;
    on_settings?: Callback<()>;
    on_logout?: Callback<()>;
}
```

**Rust Component Definition:**
```rust
/// Props for UserMenu component.
#[derive(Clone, Props)]
pub struct UserMenuProps {
    pub user: ReadSignal<User>,
    #[prop(optional)]
    pub on_profile: Option<Callback<()>>,
    #[prop(optional)]
    pub on_settings: Option<Callback<()>>,
    #[prop(optional)]
    pub on_logout: Option<Callback<()>>,
}

/// UserMenu component provides a dropdown menu for user actions.
#[component]
pub fn UserMenu(props: UserMenuProps) -> impl IntoView {
    let user = props.user;
    
    // Local signal for menu visibility
    let (menu_visible, set_menu_visible) = create_signal(false);
    
    // Handle menu toggle
    let handle_toggle = move |_| {
        set_menu_visible.update(|v| !v);
    };
    
    // Handle profile click
    let handle_profile = move |_| {
        set_menu_visible.set(false);
        if let Some(ref on_profile) = props.on_profile {
            on_profile.call(());
        }
    };
    
    // Handle settings click
    let handle_settings = move |_| {
        set_menu_visible.set(false);
        if let Some(ref on_settings) = props.on_settings {
            on_settings.call(());
        }
    };
    
    // Handle logout click
    let handle_logout = move |_| {
        set_menu_visible.set(false);
        if let Some(ref on_logout) = props.on_logout {
            on_logout.call(());
        }
    };
    
    // Close menu when clicking outside
    create_effect(move |_| {
        if menu_visible.get() {
            let set_menu_visible = set_menu_visible.clone();
            let listener = Closure::wrap(move |_: Event| {
                set_menu_visible.set(false);
            });
            
            window().add_event_listener_with_callback(
                "click",
                &listener,
            );
            
            // Cleanup listener on unmount
            on_cleanup(move || {
                window().remove_event_listener_with_callback(
                    "click",
                    &listener,
                );
            })
        }
    });
    
    view! {
        <div class="user-menu">
            <button
                class="user-menu-trigger"
                aria_label="User menu"
                aria_expanded=menu_visible
                on:click=handle_toggle
            >
                <img
                    class="user-menu-avatar"
                    src=move || format!("/api/users/{}/avatar", user.get().id)
                    alt=format!("Avatar for {}", user.get().username)
                />
                <span class="user-menu-username">{move || user.get().username.clone()}</span>
                <span class="user-menu-arrow">"▼"</span>
            </button>
            <Show
                when=menu_visible
                fallback=|| view! { <div></div> }
            >
                <div class="user-menu-dropdown">
                    <div class="user-menu-header">
                        <p class="user-menu-name">{move || user.get().name.clone()}</p>
                        <p class="user-menu-email">{move || user.get().email.clone()}</p>
                    </div>
                    <ul class="user-menu-items" role="menu">
                        <Show
                            when=move || props.on_profile.is_some()
                            fallback=|| view! { <li></li> }
                        >
                            <li role="none">
                                <button
                                    class="user-menu-item"
                                    role="menuitem"
                                    on:click=handle_profile
                                >
                                    "Profile"
                                </button>
                            </li>
                        </Show>
                        <Show
                            when=move || props.on_settings.is_some()
                            fallback=|| view! { <li></li> }
                        >
                            <li role="none">
                                <button
                                    class="user-menu-item"
                                    role="menuitem"
                                    on:click=handle_settings
                                >
                                    "Settings"
                                </button>
                            </li>
                        </Show>
                        <li class="user-menu-divider" role="separator"></li>
                        <Show
                            when=move || props.on_logout.is_some()
                            fallback=|| view! { <li></li> }
                        >
                            <li role="none">
                                <button
                                    class="user-menu-item user-menu-item-danger"
                                    role="menuitem"
                                    on:click=handle_logout
                                >
                                    "Logout"
                                </button>
                            </li>
                        </Show>
                    </ul>
                </div>
            </Show>
        </div>
    }
}
```

### 6.3. UserSettings Component

The UserSettings component provides user settings management interface.

#### 6.3.1. Component Interface

**TypeScript Interface:**
```typescript
/**
 * Props for UserSettings component.
 *
 * @property user - The current user (reactive signal)
 * @property settings - User settings (reactive signal)
 * @property on_save - Callback invoked when settings are saved
 * @property on_cancel - Callback invoked when editing is cancelled
 *
 * @constraint user signal must contain a valid User
 * @constraint settings signal must contain a valid UserSettings
 * @constraint on_save callback is required
 */
export interface UserSettingsProps {
    user: ReadSignal<User>;
    settings: ReadSignal<UserSettings>;
    on_save: Callback<UserSettings>;
    on_cancel?: Callback<()>;
}
```

**Rust Component Definition:**
```rust
/// Props for UserSettings component.
#[derive(Clone, Props)]
pub struct UserSettingsProps {
    pub user: ReadSignal<User>,
    pub settings: ReadSignal<UserSettings>,
    pub on_save: Callback<UserSettings>,
    #[prop(optional)]
    pub on_cancel: Option<Callback<()>>,
}

/// UserSettings component provides user settings management.
#[component]
pub fn UserSettings(props: UserSettingsProps) -> impl IntoView {
    let settings = props.settings;
    let on_save = props.on_save.clone();
    
    // Local signals for settings values
    let (theme, set_theme) = create_signal(settings.get().theme.clone());
    let (language, set_language) = create_signal(settings.get().language.clone());
    let (timezone, set_timezone) = create_signal(settings.get().timezone.clone());
    let (email_notifications, set_email_notifications) = create_signal(settings.get().email_notifications);
    let (digest_frequency, set_digest_frequency) = create_signal(settings.get().digest_frequency.clone());
    
    // Local signal for has unsaved changes
    let (has_unsaved_changes, set_has_unsaved_changes) = create_signal(false);
    
    // Derived signal for can save
    let can_save = create_memo(move |_| {
        has_unsaved_changes.get()
    });
    
    // Handle settings change
    let handle_settings_change = move |_| {
        set_has_unsaved_changes.set(true);
    };
    
    // Handle save
    let handle_save = move |_| {
        let updated_settings = UserSettings {
            theme: theme.get(),
            language: language.get(),
            timezone: timezone.get(),
            email_notifications: email_notifications.get(),
            digest_frequency: digest_frequency.get(),
        };
        on_save.call(updated_settings);
        set_has_unsaved_changes.set(false);
    };
    
    // Handle cancel
    let handle_cancel = move |_| {
        if let Some(ref on_cancel) = props.on_cancel {
            on_cancel.call(());
        }
    };
    
    view! {
        <div class="user-settings">
            <div class="user-settings-header">
                <h2>"Settings"</h2>
                <div class="user-settings-actions">
                    <Show
                        when=move || props.on_cancel.is_some()
                        fallback=|| view! { <div></div> }
                    >
                        <button
                            class="btn-secondary"
                            aria_label="Cancel"
                            on:click=handle_cancel
                        >
                            "Cancel"
                        </button>
                    </Show>
                    <button
                        class="btn-primary"
                        aria_label="Save settings"
                        disabled=move || !can_save()
                        on:click=handle_save
                    >
                        "Save"
                    </button>
                </div>
            </div>
            <div class="user-settings-content">
                <div class="settings-section">
                    <h3>"Appearance"</h3>
                    <div class="settings-field">
                        <label class="settings-label" for="theme">"Theme"</label>
                        <select
                            id="theme"
                            class="settings-select"
                            value=theme
                            on:change=move |e| {
                                set_theme.set(event_target_value(&e));
                                handle_settings_change(());
                            }
                            aria_label="Theme"
                        >
                            <option value="light">"Light"</option>
                            <option value="dark">"Dark"</option>
                            <option value="auto">"Auto"</option>
                        </select>
                    </div>
                </div>
                <div class="settings-section">
                    <h3>"Language & Region"</h3>
                    <div class="settings-field">
                        <label class="settings-label" for="language">"Language"</label>
                        <select
                            id="language"
                            class="settings-select"
                            value=language
                            on:change=move |e| {
                                set_language.set(event_target_value(&e));
                                handle_settings_change(());
                            }
                            aria_label="Language"
                        >
                            <option value="en">"English"</option>
                            <option value="es">"Spanish"</option>
                            <option value="fr">"French"</option>
                            <option value="de">"German"</option>
                        </select>
                    </div>
                    <div class="settings-field">
                        <label class="settings-label" for="timezone">"Timezone"</label>
                        <select
                            id="timezone"
                            class="settings-select"
                            value=timezone
                            on:change=move |e| {
                                set_timezone.set(event_target_value(&e));
                                handle_settings_change(());
                            }
                            aria_label="Timezone"
                        >
                            <option value="UTC">"UTC"</option>
                            <option value="America/New_York">"Eastern Time"</option>
                            <option value="America/Los_Angeles">"Pacific Time"</option>
                            <option value="Europe/London">"London"</option>
                        </select>
                    </div>
                </div>
                <div class="settings-section">
                    <h3>"Notifications"</h3>
                    <div class="settings-field">
                        <label class="settings-checkbox-label">
                            <input
                                type="checkbox"
                                class="settings-checkbox"
                                checked=email_notifications
                                on:change=move |e| {
                                    set_email_notifications.set(event_target_checked(&e));
                                    handle_settings_change(());
                                }
                                aria_label="Email notifications"
                            />
                            "Email notifications"
                        </label>
                    </div>
                    <div class="settings-field">
                        <label class="settings-label" for="digest-frequency">"Digest frequency"</label>
                        <select
                            id="digest-frequency"
                            class="settings-select"
                            value=digest_frequency
                            on:change=move |e| {
                                set_digest_frequency.set(event_target_value(&e));
                                handle_settings_change(());
                            }
                            aria_label="Digest frequency"
                        >
                            <option value="never">"Never"</option>
                            <option value="daily">"Daily"</option>
                            <option value="weekly">"Weekly"</option>
                            <option value="monthly">"Monthly"</option>
                        </select>
                    </div>
                </div>
            </div>
        </div>
    }
}
```

---

---

## 7. COMPONENT COMMUNICATION

Component communication patterns define how components exchange data and trigger actions. This section specifies the communication mechanisms used throughout the Tachyon web frontend.

### 7.1. Props-Based Communication

Props-based communication is the primary mechanism for parent-to-child data flow.

#### 7.1.1. Props Interface Definition

Props define the contract between parent and child components.

**TypeScript Props Interface:**
```typescript
/**
 * Props interface for child component.
 *
 * @property data - Data passed from parent
 * @property on_action - Callback for child to notify parent
 * @property optional_prop - Optional configuration
 *
 * @constraint data must be valid when component is mounted
 */
export interface ChildComponentProps<T> {
    data: ReadSignal<T>;
    on_action: Callback<Action>;
    optional_prop?: boolean;
}
```

**Rust Props Interface:**
```rust
/// Props interface for child component.
///
/// # Fields
///
/// * `data` - Data passed from parent (reactive signal)
/// * `on_action` - Callback for child to notify parent
/// * `optional_prop` - Optional configuration
#[derive(Clone, Props)]
pub struct ChildComponentProps<T: Clone + 'static> {
    /// Data passed from parent (reactive signal)
    pub data: ReadSignal<T>,
    
    /// Callback for child to notify parent
    pub on_action: Callback<Action>,
    
    /// Optional configuration
    #[prop(default = false)]
    pub optional_prop: bool,
}
```

#### 7.1.2. Callback Pattern

Callbacks enable child-to-parent communication for actions and events.

**Callback Definition:**
```rust
/// Callback type for component communication.
pub type Callback<T> = leptos::Callback<T>;

/// Action type for component communication.
#[derive(Clone, Debug)]
pub enum Action {
    Select(DocumentId),
    Delete(DocumentId),
    Edit(DocumentId),
    Cancel,
}
```

**Callback Usage Pattern:**
```rust
#[component]
pub fn ParentComponent() -> impl IntoView {
    let (selected_document, set_selected_document) = create_signal(None::<DocumentId>);
    
    // Handle action from child
    let handle_action = move |action: Action| {
        match action {
            Action::Select(id) => set_selected_document.set(Some(id)),
            Action::Delete(id) => {
                // Handle delete
            },
            Action::Edit(id) => {
                // Handle edit
            },
            Action::Cancel => {
                // Handle cancel
            },
        }
    };
    
    view! {
        <ChildComponent
            data=selected_document
            on_action=handle_action
        />
    }
}
```

### 7.2. Context-Based Communication

Context-based communication enables sharing state across component hierarchies without prop drilling.

#### 7.2.1. Context Definition

Context provides application-wide shared state.

**Context Type Definition:**
```rust
/// Application context for shared state.
#[derive(Clone)]
pub struct ApplicationContext {
    pub session: Signal<Option<Session>>,
    pub theme: Signal<Theme>,
    pub notifications: Signal<Vec<Notification>>,
    pub api_client: Rc<ApiClient>,
}

impl ApplicationContext {
    /// Create new application context.
    pub fn new(api_client: Rc<ApiClient>) -> Self {
        Self {
            session: create_signal(None),
            theme: create_signal(Theme::Light),
            notifications: create_signal(Vec::new()),
            api_client,
        }
    }
}
```

**Context Provider Pattern:**
```rust
#[component]
pub fn App() -> impl IntoView {
    let api_client = use_context::<Rc<ApiClient>>();
    let app_context = ApplicationContext::new(api_client);
    
    provide_context(app_context);
    
    view! {
        <Router>
            <Routes>
                <Route path="/" view=Home />
                <Route path="/documents" view=DocumentList />
            </Routes>
        </Router>
    }
}
```

#### 7.2.2. Context Consumer Pattern

Components consume context through the `use_context` hook.

**Context Consumer Pattern:**
```rust
#[component]
pub fn DocumentEditor() -> impl IntoView {
    let app_context = use_context::<ApplicationContext>();
    let session = app_context.session;
    let api_client = app_context.api_client;
    
    // Use session and api_client
    let is_authenticated = create_memo(move |_| {
        session.get().is_some()
    });
    
    view! {
        <Show
            when=is_authenticated
            fallback=|| view! { <div>"Please login"</div> }
        >
            <div class="document-editor">
                <!-- Editor content -->
            </div>
        </Show>
    }
}
```

### 7.3. Event-Based Communication

Event-based communication enables cross-component communication without direct references.

#### 7.3.1. Event Bus Definition

Event bus provides publish-subscribe pattern for component communication.

**Event Bus Type Definition:**
```rust
/// Event type for component communication.
#[derive(Clone, Debug)]
pub enum AppEvent {
    DocumentCreated(Document),
    DocumentUpdated(Document),
    DocumentDeleted(DocumentId),
    UserLoggedIn(User),
    UserLoggedOut,
    Notification(Notification),
}

/// Event bus for application-wide events.
#[derive(Clone)]
pub struct EventBus {
    subscribers: Rc<RefCell<Vec<Box<dyn Fn(AppEvent)>>>>,
}

impl EventBus {
    /// Create new event bus.
    pub fn new() -> Self {
        Self {
            subscribers: Rc::new(RefCell::new(Vec::new())),
        }
    }
    
    /// Subscribe to events.
    pub fn subscribe<F>(&self, callback: F)
    where
        F: Fn(AppEvent) + 'static,
    {
        self.subscribers.borrow_mut().push(Box::new(callback));
    }
    
    /// Publish event.
    pub fn publish(&self, event: AppEvent) {
        for subscriber in self.subscribers.borrow().iter() {
            subscriber(event.clone());
        }
    }
}
```

**Event Bus Usage Pattern:**
```rust
#[component]
pub fn DocumentList() -> impl IntoView {
    let event_bus = use_context::<Rc<EventBus>>();
    let documents = create_signal(Vec::<Document>::new());
    
    // Subscribe to document events
    create_effect(move |_| {
        let event_bus = event_bus.clone();
        let set_documents = documents.clone();
        
        event_bus.subscribe(move |event| {
            match event {
                AppEvent::DocumentCreated(doc) => {
                    set_documents.update(|docs| docs.push(doc));
                }
                AppEvent::DocumentUpdated(doc) => {
                    set_documents.update(|docs| {
                        if let Some(pos) = docs.iter().position(|d| d.id == doc.id) {
                            docs[pos] = doc;
                        }
                    });
                }
                AppEvent::DocumentDeleted(id) => {
                    set_documents.update(|docs| {
                        docs.retain(|d| d.id != id);
                    });
                }
                _ => {}
            }
        });
    });
    
    view! {
        <div class="document-list">
            {move || {
                documents.get()
                    .into_iter()
                    .map(|doc| view! { <DocumentCard document=doc /> })
                    .collect_view()
            }}
        </div>
    }
}
```

### 7.4. Signal-Based Communication

Signal-based communication enables reactive data sharing across components.

#### 7.4.1. Signal Sharing Pattern

Signals can be shared across components through props or context.

**Signal Sharing Pattern:**
```rust
#[component]
pub fn ParentComponent() -> impl IntoView {
    // Create shared signal
    let (shared_value, set_shared_value) = create_signal(0);
    
    view! {
        <div>
            <ChildComponentA value=shared_value />
            <ChildComponentB value=shared_value on_change=set_shared_value />
        </div>
    }
}

#[component]
pub fn ChildComponentA(value: ReadSignal<i32>) -> impl IntoView {
    view! {
        <div>"Value: " {value}</div>
    }
}

#[component]
pub fn ChildComponentB(
    value: ReadSignal<i32>,
    on_change: Callback<i32>,
) -> impl IntoView {
    let handle_change = move |e: Event| {
        let new_value = event_target_value::<i32>(&e).unwrap();
        on_change.call(new_value);
    };
    
    view! {
        <input
            type="number"
            value=value
            on:input=handle_change
        />
    }
}
```

### 7.5. Communication Pattern Selection

Select the appropriate communication pattern based on use case:

| Pattern | Use Case | Pros | Cons |
|---------|----------|------|------|
| **Props** | Parent-to-child data flow | Type-safe, explicit | Limited to parent-child |
| **Callbacks** | Child-to-parent actions | Type-safe, explicit | Limited to child-parent |
| **Context** | Application-wide state | No prop drilling | Can cause unnecessary re-renders |
| **Events** | Cross-component communication | Decoupled, flexible | Harder to trace |
| **Signals** | Reactive data sharing | Reactive, efficient | Can cause tight coupling |

### 7.6. Communication Best Practices

Follow these best practices for component communication:

1. **Prefer Props for Parent-Child Data Flow:** Use props for passing data from parent to child components.

2. **Use Callbacks for Child-Parent Actions:** Use callbacks for child components to notify parent of actions.

3. **Use Context for Application-Wide State:** Use context for state that needs to be accessed by many components.

4. **Use Events for Cross-Component Communication:** Use events for communication between unrelated components.

5. **Avoid Direct Component References:** Avoid direct references to other components to maintain loose coupling.

6. **Document Communication Contracts:** Clearly document the props, callbacks, and events that components accept and emit.

7. **Use Type-Safe Communication:** Leverage Rust's type system to ensure type-safe communication.

8. **Handle Edge Cases:** Handle edge cases such as null values, empty collections, and missing callbacks.

### 7.7. TypeScript Interface Definitions

**Callback Type Definition:**
```typescript
/**
 * Callback type for component communication.
 *
 * @template T - The type of data passed to callback
 */
export type Callback<T> = (data: T) => void;

/**
 * Action type for component communication.
 */
export type Action =
    | { type: 'select', payload: DocumentId }
    | { type: 'delete', payload: DocumentId }
    | { type: 'edit', payload: DocumentId }
    | { type: 'cancel' };
```

**Context Type Definition:**
```typescript
/**
 * Application context for shared state.
 */
export interface ApplicationContext {
    session: Signal<User | null>;
    theme: Signal<Theme>;
    notifications: Signal<Notification[]>;
    apiClient: ApiClient;
}
```

**Event Type Definition:**
```typescript
/**
 * Event type for component communication.
 */
export type AppEvent =
    | { type: 'documentCreated', payload: Document }
    | { type: 'documentUpdated', payload: Document }
    | { type: 'documentDeleted', payload: DocumentId }
    | { type: 'userLoggedIn', payload: User }
    | { type: 'userLoggedOut' }
    | { type: 'notification', payload: Notification };
```

---

---

## 8. COMPONENT STATE MANAGEMENT

Component state management defines how components manage and synchronize their internal state. This section specifies the state management patterns used throughout the Tachyon web frontend.

### 8.1. Signal-Based State Management

Signals are the primary mechanism for reactive state management in Leptos.

#### 8.1.1. Signal Creation Patterns

**Local Signal Creation:**
```rust
#[component]
pub fn Counter() -> impl IntoView {
    // Create local signal with initial value
    let (count, set_count) = create_signal(0);
    
    let handle_increment = move |_| {
        set_count.update(|c| c + 1);
    };
    
    view! {
        <div>
            <p>"Count: " {count}</p>
            <button on:click=handle_increment>"Increment"</button>
        </div>
    }
}
```

**Derived Signal Creation:**
```rust
#[component]
pub fn DocumentStats(document: ReadSignal<Document>) -> impl IntoView {
    // Create derived signal from prop
    let word_count = create_memo(move |_| {
        document.get().content.split_whitespace().count()
    });
    
    let char_count = create_memo(move |_| {
        document.get().content.chars().count()
    });
    
    view! {
        <div>
            <p>"Words: " {word_count}</p>
            <p>"Characters: " {char_count}</p>
        </div>
    }
}
```

#### 8.1.2. Signal Update Patterns

**Direct Update:**
```rust
let (value, set_value) = create_signal(0);

// Direct update
set_value.set(42);

// Update with function
set_value.update(|v| v + 1);
```

**Conditional Update:**
```rust
let (value, set_value) = create_signal(0);

// Conditional update
set_value.update(|v| {
    if v < 100 {
        v + 1
    } else {
        v
    }
});
```

**Async Update:**
```rust
let (value, set_value) = create_signal(0);
let (loading, set_loading) = create_signal(false);

let handle_async_update = move |_| {
    set_loading.set(true);
    
    spawn_local(async move {
        // Simulate async operation
        set_timeout_with_handle(
            move || {
                set_value.update(|v| v + 1);
                set_loading.set(false);
            },
            Duration::from_millis(1000),
        );
    });
};
```

### 8.2. State Synchronization

State synchronization ensures consistency across components.

#### 8.2.1. Lift State Up Pattern

Lift state up to the nearest common ancestor.

**Lift State Up Pattern:**
```rust
#[component]
pub fn ParentComponent() -> impl IntoView {
    // Lift state up to parent
    let (selected_item, set_selected_item) = create_signal(None::<String>);
    
    view! {
        <div>
            <ChildComponentA
                selected_item=selected_item
                on_select=set_selected_item
            />
            <ChildComponentB
                selected_item=selected_item
                on_select=set_selected_item
            />
        </div>
    }
}

#[component]
pub fn ChildComponentA(
    selected_item: ReadSignal<Option<String>>,
    on_select: Callback<String>,
) -> impl IntoView {
    let handle_select = move |item: String| {
        on_select.call(item);
    };
    
    view! {
        <div>
            {["Item 1", "Item 2", "Item 3"]
                .into_iter()
                .map(|item| {
                    let item_clone = item.clone();
                    let is_selected = create_memo(move |_| {
                        selected_item.get().as_ref() == Some(&item_clone)
                    });
                    
                    view! {
                        <div
                            class=format!("item {}", if is_selected.get() { "selected" } else { "" })
                            on:click=move |_| handle_select(item_clone.clone())
                        >
                            {item}
                        </div>
                    }
                })
                .collect_view()}
        </div>
    }
}
```

#### 8.2.2. Global Store Pattern

Global store provides application-wide state management.

**Global Store Definition:**
```rust
/// Global store for application state.
#[derive(Clone)]
pub struct GlobalStore {
    pub session: Signal<Option<Session>>,
    pub theme: Signal<Theme>,
    pub notifications: Signal<Vec<Notification>>,
    pub documents: Signal<Vec<Document>>,
}

impl GlobalStore {
    /// Create new global store.
    pub fn new() -> Self {
        Self {
            session: create_signal(None),
            theme: create_signal(Theme::Light),
            notifications: create_signal(Vec::new()),
            documents: create_signal(Vec::new()),
        }
    }
}
```

**Global Store Usage Pattern:**
```rust
#[component]
pub fn App() -> impl IntoView {
    let store = GlobalStore::new();
    provide_context(store);
    
    view! {
        <Router>
            <Routes>
                <Route path="/" view=Home />
                <Route path="/documents" view=DocumentList />
            </Routes>
        </Router>
    }
}

#[component]
pub fn DocumentList() -> impl IntoView {
    let store = use_context::<GlobalStore>();
    let documents = store.documents;
    
    view! {
        <div class="document-list">
            {move || {
                documents.get()
                    .into_iter()
                    .map(|doc| view! { <DocumentCard document=doc /> })
                    .collect_view()
            }}
        </div>
    }
}
```

### 8.3. State Persistence

State persistence ensures data survives page refreshes and browser restarts.

#### 8.3.1. LocalStorage Persistence

Persist state to browser localStorage.

**LocalStorage Persistence Pattern:**
```rust
#[component]
pub fn SettingsComponent() -> impl IntoView {
    // Initialize state from localStorage
    let (theme, set_theme) = create_signal({
        window()
            .local_storage()
            .get_item("theme")
            .map(|t| t.parse::<Theme>().ok())
            .flatten()
            .unwrap_or(Theme::Light)
    });
    
    // Persist theme changes to localStorage
    create_effect(move |_| {
        let theme = theme.get();
        window()
            .local_storage()
            .set_item("theme", &theme.to_string());
    });
    
    view! {
        <div class="settings">
            <select
                value=theme
                on:change=move |e| {
                    let value = event_target_value(&e);
                    if let Ok(parsed_theme) = value.parse::<Theme>() {
                        set_theme.set(parsed_theme);
                    }
                }
            >
                <option value="light">"Light"</option>
                <option value="dark">"Dark"</option>
                <option value="auto">"Auto"</option>
            </select>
        </div>
    }
}
```

#### 8.3.2. SessionStorage Persistence

Persist temporary state to browser sessionStorage.

**SessionStorage Persistence Pattern:**
```rust
#[component]
pub fn FormComponent() -> impl IntoView {
    // Initialize state from sessionStorage
    let (form_data, set_form_data) = create_signal({
        window()
            .session_storage()
            .get_item("form_data")
            .map(|d| serde_json::from_str::<FormData>(d).ok())
            .flatten()
            .unwrap_or_default()
    });
    
    // Persist form changes to sessionStorage
    create_effect(move |_| {
        let data = form_data.get();
        if let Ok(json) = serde_json::to_string(&data) {
            window()
                .session_storage()
                .set_item("form_data", &json);
        }
    });
    
    view! {
        <form class="form">
            <input
                type="text"
                value=move || form_data.get().name.clone()
                on:input=move |e| {
                    let value = event_target_value(&e);
                    set_form_data.update(|d| d.name = value);
                }
            />
        </form>
    }
}
```

### 8.4. State Synchronization with Server

State synchronization ensures consistency between client and server.

#### 8.4.1. WebSocket Synchronization

Use WebSocket for real-time state synchronization.

**WebSocket Synchronization Pattern:**
```rust
#[component]
pub fn DocumentEditor(document: ReadSignal<Document>) -> impl IntoView {
    let (content, set_content) = create_signal(document.get().content.clone());
    let (sync_status, set_sync_status) = create_signal(SyncStatus::Synced);
    
    // Initialize WebSocket connection
    let ws_client = use_context::<Rc<WebSocketClient>>();
    
    // Subscribe to WebSocket messages
    create_effect(move |_| {
        let ws_client = ws_client.clone();
        let set_content = set_content.clone();
        let set_sync_status = set_sync_status.clone();
        
        let subscription = ws_client.subscribe(move |message| {
            match message {
                WebSocketMessage::DocumentUpdate { content, .. } => {
                    set_content.set(content);
                    set_sync_status.set(SyncStatus::Synced);
                }
                WebSocketMessage::SyncError => {
                    set_sync_status.set(SyncStatus::Error);
                }
                _ => {}
            }
        });
        
        // Cleanup subscription on unmount
        on_cleanup(move || {
            subscription.unsubscribe();
        })
    });
    
    // Handle content changes
    let handle_content_change = move |e: Event| {
        let new_content = event_target_value(&e);
        set_content.set(new_content.clone());
        set_sync_status.set(SyncStatus::Syncing);
        
        // Send update to server via WebSocket
        ws_client.send(WebSocketMessage::UpdateDocument {
            document_id: document.get().id.clone(),
            content: new_content,
        });
    };
    
    view! {
        <div class="document-editor">
            <div class="sync-status">
                {move || match sync_status.get() {
                    SyncStatus::Synced => "Synced".to_string(),
                    SyncStatus::Syncing => "Syncing...".to_string(),
                    SyncStatus::Error => "Sync Error".to_string(),
                }}
            </div>
            <textarea
                value=content
                on:input=handle_content_change
            />
        </div>
    }
}
```

#### 8.4.2. HTTP Synchronization

Use HTTP for periodic state synchronization.

**HTTP Synchronization Pattern:**
```rust
#[component]
pub fn DocumentList() -> impl IntoView {
    let (documents, set_documents) = create_signal(Vec::<Document>::new());
    let (loading, set_loading) = create_signal(false);
    
    // Fetch documents from server
    let fetch_documents = move |_| {
        set_loading.set(true);
        
        spawn_local(async move {
            let api_client = use_context::<Rc<ApiClient>>();
            
            match api_client.get_documents().await {
                Ok(docs) => {
                    set_documents.set(docs);
                    set_loading.set(false);
                }
                Err(e) => {
                    log::error!("Failed to fetch documents: {:?}", e);
                    set_loading.set(false);
                }
            }
        });
    };
    
    // Fetch documents on mount
    create_effect(fetch_documents);
    
    // Refresh documents periodically
    create_effect(move |_| {
        set_interval_with_handle(
            fetch_documents,
            Duration::from_secs(30),
        )
    });
    
    view! {
        <div class="document-list">
            <Show
                when=loading
                fallback=|| view! {
                    <div class="document-list-items">
                        {move || {
                            documents.get()
                                .into_iter()
                                .map(|doc| view! { <DocumentCard document=doc /> })
                                .collect_view()
                        }}
                    </div>
                }
            >
                <div class="loading-indicator">"Loading..."</div>
            </Show>
        </div>
    }
}
```

### 8.5. State Management Best Practices

Follow these best practices for state management:

1. **Use Signals for Reactive State:** Use signals for all reactive state within components.

2. **Lift State Up When Appropriate:** Lift state up to the nearest common ancestor for shared state.

3. **Use Global Store for Application-Wide State:** Use global store for state that needs to be accessed by many components.

4. **Persist State When Appropriate:** Persist state to localStorage or sessionStorage when needed across sessions.

5. **Synchronize State with Server:** Use WebSocket or HTTP to synchronize state with server when needed.

6. **Use Derived Signals for Computed Values:** Use derived signals (create_memo) for computed values to avoid unnecessary recalculations.

7. **Handle Loading States:** Handle loading states explicitly when fetching or updating state.

8. **Handle Error States:** Handle error states explicitly when state operations can fail.

9. **Clean Up Resources:** Clean up resources (subscriptions, intervals) when components unmount.

### 8.6. TypeScript Interface Definitions

**Signal Type Definition:**
```typescript
/**
 * Signal type for reactive state.
 *
 * @template T - The type of value held by the signal
 */
export type Signal<T> = {
    (): T;
    (value: T): void;
    (updater: (value: T) => T): void;
    (updater: (updater: (value: T) => T) => void): void;
};
```

**Store Type Definition:**
```typescript
/**
 * Global store for application state.
 */
export interface GlobalStore {
    session: Signal<User | null>;
    theme: Signal<Theme>;
    notifications: Signal<Notification[]>;
    documents: Signal<Document[]>;
}
```

**Sync Status Type Definition:**
```typescript
/**
 * Synchronization status type.
 */
export type SyncStatus =
    | 'synced'
    | 'syncing'
    | 'error'
    | 'unknown';
```

---

---

## 9. COMPONENT SECURITY

Component security ensures that components protect against common web vulnerabilities and follow security best practices. This section specifies the security requirements for all web components.

### 9.1. Authentication

Authentication ensures that only authorized users can access protected resources.

#### 9.1.1. Session Management

Components must properly manage user sessions and authentication tokens.

**Session Management Pattern:**
```rust
#[component]
pub fn ProtectedComponent() -> impl IntoView {
    let session = use_context::<Signal<Option<Session>>>();
    
    // Redirect to login if not authenticated
    let is_authenticated = create_memo(move |_| {
        session.get().is_some()
    });
    
    view! {
        <Show
            when=is_authenticated
            fallback=|| view! { <Redirect to="/login" /> }
        >
            <div class="protected-content">
                <!-- Protected content -->
            </div>
        </Show>
    }
}
```

#### 9.1.2. Token Management

Components must securely store and use authentication tokens.

**Token Management Pattern:**
```rust
/// Token storage utility.
pub struct TokenStorage {
    access_token: Option<String>,
    refresh_token: Option<String>,
}

impl TokenStorage {
    /// Store tokens securely.
    pub fn store(&mut self, access_token: String, refresh_token: String) {
        // Store in memory for session
        self.access_token = Some(access_token);
        self.refresh_token = Some(refresh_token);
        
        // Store in httpOnly cookie for XSS protection
        window()
            .document()
            .cookie()
            .set_with_options(
                "access_token",
                &access_token,
                &CookieOptions {
                    http_only: true,
                    secure: true,
                    same_site: "strict",
                    expires: Some(Utc::now() + Duration::hours(1)),
                },
            );
    }
    
    /// Clear tokens on logout.
    pub fn clear(&mut self) {
        self.access_token = None;
        self.refresh_token = None;
        
        window()
            .document()
            .cookie()
            .delete("access_token");
    }
}
```

### 9.2. Authorization

Authorization ensures that users can only access resources they are permitted to access.

#### 9.2.1. Role-Based Access Control

Components must implement role-based access control for protected operations.

**RBAC Pattern:**
```rust
/// User role enumeration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Role {
    Admin,
    Editor,
    Viewer,
}

/// Check if user has required role.
pub fn has_role(user: &User, required_role: Role) -> bool {
    user.roles.contains(&required_role)
}

#[component]
pub fn AdminPanel(user: ReadSignal<User>) -> impl IntoView {
    let is_admin = create_memo(move |_| {
        has_role(&user.get(), Role::Admin)
    });
    
    view! {
        <Show
            when=is_admin
            fallback=|| view! {
                <div class="access-denied">
                    <p>"Access denied: Admin privileges required"</p>
                </div>
            }
        >
            <div class="admin-panel">
                <!-- Admin content -->
            </div>
        </Show>
    }
}
```

#### 9.2.2. Resource-Based Access Control

Components must implement resource-based access control for protected resources.

**RBAC Pattern:**
```rust
/// Check if user can access resource.
pub fn can_access_resource(user: &User, resource_id: &ResourceId) -> bool {
    // Check if user has permission for resource
    user.permissions
        .iter()
        .any(|perm| perm.resource_id == *resource_id)
}

#[component]
pub fn DocumentEditor(
    document: ReadSignal<Document>,
    user: ReadSignal<User>,
) -> impl IntoView {
    let can_edit = create_memo(move |_| {
        can_access_resource(&user.get(), &document.get().id)
    });
    
    view! {
        <Show
            when=can_edit
            fallback=|| view! {
                <div class="access-denied">
                    <p>"Access denied: You do not have permission to edit this document"</p>
                </div>
            }
        >
            <div class="document-editor">
                <!-- Editor content -->
            </div>
        </Show>
    }
}
```

### 9.3. Input Validation

Input validation ensures that user input is properly sanitized and validated.

#### 9.3.1. Client-Side Validation

Components must validate input on the client side before sending to server.

**Input Validation Pattern:**
```rust
/// Validate document title.
pub fn validate_document_title(title: &str) -> Result<(), ValidationError> {
    if title.trim().is_empty() {
        return Err(ValidationError::Required("Title is required"));
    }
    
    if title.len() > 200 {
        return Err(ValidationError::TooLong("Title must be less than 200 characters"));
    }
    
    if title.contains(|c| c.is_control()) {
        return Err(ValidationError::InvalidCharacters("Title contains invalid characters"));
    }
    
    Ok(())
}

#[component]
pub fn DocumentForm() -> impl IntoView {
    let (title, set_title) = create_signal(String::new());
    let (title_error, set_title_error) = create_signal(None::<ValidationError>);
    
    let handle_title_change = move |e: Event| {
        let new_title = event_target_value(&e);
        set_title.set(new_title.clone());
        
        match validate_document_title(&new_title) {
            Ok(()) => set_title_error.set(None),
            Err(e) => set_title_error.set(Some(e)),
        }
    };
    
    view! {
        <div class="document-form">
            <label for="title">"Title"</label>
            <input
                type="text"
                id="title"
                value=title
                on:input=handle_title_change
                aria_invalid=title_error.get().is_some()
            />
            <Show
                when=move || title_error.get().is_some()
                fallback=|| view! { <span></span> }
            >
                <span class="error-message" role="alert">
                    {move || title_error.get().map(|e| e.to_string()).unwrap_or_default()}
                </span>
            </Show>
        </div>
    }
}
```

#### 9.3.2. Output Encoding

Components must properly encode output to prevent XSS attacks.

**Output Encoding Pattern:**
```rust
/// Sanitize HTML output.
pub fn sanitize_html(input: &str) -> String {
    // Use a proper HTML sanitization library
    ammonia::clean(input).to_string()
}

#[component]
pub fn DocumentContent(content: ReadSignal<String>) -> impl IntoView {
    let sanitized_content = create_memo(move |_| {
        sanitize_html(&content.get())
    });
    
    view! {
        <div class="document-content" inner_html=sanitized_content />
    }
}
```

### 9.4. Content Security Policy

Components must implement Content Security Policy headers to prevent XSS attacks.

**CSP Implementation Pattern:**
```rust
/// Generate CSP headers.
pub fn generate_csp_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'wasm-unsafe-eval'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: https:; \
             connect-src 'self' wss://; \
             font-src 'self'; \
             object-src 'none'; \
             base-uri 'self'; \
             frame-ancestors 'none'; \
             form-action 'self';"
        ),
    );
    
    headers
}
```

### 9.5. CSRF Protection

Components must implement CSRF protection for state-changing operations.

**CSRF Protection Pattern:**
```rust
/// Generate CSRF token.
pub fn generate_csrf_token() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let token: String = (0..32)
        .map(|_| rng.gen::<char>())
        .collect();
    
    // Store token in session
    window()
        .session_storage()
        .set_item("csrf_token", &token);
    
    token
}

#[component]
pub fn DocumentForm() -> impl IntoView {
    let csrf_token = create_memo(move |_| {
        window()
            .session_storage()
            .get_item("csrf_token")
            .unwrap_or_default()
    });
    
    view! {
        <form method="post" action="/documents/save">
            <input
                type="hidden"
                name="csrf_token"
                value=csrf_token
            />
            <!-- Form fields -->
        </form>
    }
}
```

### 9.6. Security Best Practices

Follow these best practices for component security:

1. **Always Validate Input:** Validate all user input on the client side before sending to server.

2. **Sanitize Output:** Sanitize all output to prevent XSS attacks.

3. **Use HTTPS:** Always use HTTPS for all communications.

4. **Implement CSP:** Implement Content Security Policy headers to prevent XSS attacks.

5. **Use CSRF Protection:** Use CSRF tokens for state-changing operations.

6. **Secure Token Storage:** Store authentication tokens securely using httpOnly cookies.

7. **Implement RBAC:** Implement role-based access control for protected resources.

8. **Handle Errors Securely:** Handle errors securely without exposing sensitive information.

9. **Log Security Events:** Log security events for audit purposes.

10. **Keep Dependencies Updated:** Keep all dependencies updated to patch known vulnerabilities.

### 9.7. TypeScript Interface Definitions

**Role Type Definition:**
```typescript
/**
 * User role enumeration.
 */
export type Role =
    | 'admin'
    | 'editor'
    | 'viewer';
```

**ValidationError Type Definition:**
```typescript
/**
 * Validation error type.
 */
export type ValidationError =
    | { type: 'required', message: string }
    | { type: 'tooLong', message: string }
    | { type: 'invalidCharacters', message: string }
    | { type: 'invalidFormat', message: string };
```

**Permission Type Definition:**
```typescript
/**
 * Permission type.
 */
export interface Permission {
    resourceId: string;
    action: 'read' | 'write' | 'delete' | 'admin';
}
```

---

---

## 10. COMPONENT PERFORMANCE

Component performance ensures that components render efficiently and provide smooth user experience. This section specifies the performance requirements and optimization techniques for all web components.

### 10.1. Rendering Optimization

Rendering optimization ensures that components update the DOM efficiently.

#### 10.1.1. Fine-Grained Reactivity

Leverage Leptos's fine-grained reactivity to minimize DOM updates.

**Fine-Grained Reactivity Pattern:**
```rust
#[component]
pub fn Counter() -> impl IntoView {
    let (count, set_count) = create_signal(0);
    
    // ✅ Good: Fine-grained reactivity
    let double_count = create_memo(move |_| count.get() * 2);
    
    view! {
        <div>
            <p>"Count: " {count}</p>
            <p>"Double: " {double_count}</p>
            <button on:click=move |_| set_count.update(|c| c + 1)>
                "Increment"
            </button>
        </div>
    }
}
```

#### 10.1.2. Lazy Rendering

Use lazy rendering for expensive components to improve initial load time.

**Lazy Rendering Pattern:**
```rust
#[component]
pub fn DocumentList(documents: ReadSignal<Vec<Document>>) -> impl IntoView {
    view! {
        <div class="document-list">
            {move || {
                documents.get()
                    .into_iter()
                    .map(|doc| view! { <DocumentCard document=doc /> })
                    .collect_view()
            }}
        </div>
    }
}
```

#### 10.1.3. Virtual Scrolling

Use virtual scrolling for large lists to improve rendering performance.

**Virtual Scrolling Pattern:**
```rust
#[component]
pub fn VirtualList<T: Clone + 'static>(
    items: ReadSignal<Vec<T>>,
    item_height: u32,
) -> impl IntoView {
    let (visible_range, set_visible_range) = create_signal((0, 10));
    let container_ref = create_node_ref::<html::Div>();
    
    // Calculate visible range based on scroll position
    let handle_scroll = move |_| {
        if let Some(container) = container_ref.get() {
            let scroll_top = container.scroll_top();
            let container_height = container.client_height();
            
            let start_index = (scroll_top / item_height as f64) as usize;
            let visible_count = (container_height / item_height as f64) as usize + 1;
            let end_index = start_index + visible_count;
            
            set_visible_range.set((start_index, end_index));
        }
    };
    
    view! {
        <div
            class="virtual-list-container"
            node_ref=container_ref
            on:scroll=handle_scroll
            style=format!(
                "height: {}px; overflow-y: auto;",
                items.get().len() * item_height
            )
        >
            <div
                class="virtual-list-content"
                style=format!(
                    "transform: translateY({}px);",
                    visible_range.get().0 * item_height
                )
            >
                {move || {
                    items.get()
                        .into_iter()
                        .skip(visible_range.get().0)
                        .take(visible_range.get().1 - visible_range.get().0)
                        .map(|item| view! { <ListItem item=item /> })
                        .collect_view()
                }}
            </div>
        </div>
    }
}
```

### 10.2. Bundle Optimization

Bundle optimization ensures that the application loads quickly and efficiently.

#### 10.2.1. Code Splitting

Use code splitting to load routes on demand.

**Code Splitting Pattern:**
```rust
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path="/" view=Home />
                <Route path="/documents" view=DocumentList />
                <Route path="/documents/:id" view=DocumentDetail />
                <Route path="/settings" view=Settings />
            </Routes>
        </Router>
    }
}
```

#### 10.2.2. Tree Shaking

Use tree shaking to eliminate unused code from bundles.

**Tree Shaking Configuration:**
```javascript
// vite.config.ts
import { defineConfig } from 'vite';
import leptos from '@leptos/vite-plugin-leptos';

export default defineConfig({
  plugins: [
    leptos({
        // Enable tree shaking
        treeShake: true,
        
        // Configure output
        output: {
            // Minify output
            minify: true,
            
            // Generate source maps
            sourcemap: true,
        },
    }),
  ],
});
```

### 10.3. Memoization

Memoization ensures that expensive computations are cached and reused.

#### 10.3.1. Memoization Pattern

Use `create_memo` to cache expensive computations.

**Memoization Pattern:**
```rust
#[component]
pub fn DocumentStats(document: ReadSignal<Document>) -> impl IntoView {
    // Memoize expensive computations
    let word_count = create_memo(move |_| {
        document.get().content.split_whitespace().count()
    });
    
    let char_count = create_memo(move |_| {
        document.get().content.chars().count()
    });
    
    let paragraph_count = create_memo(move |_| {
        document.get().content.split("\n\n").count()
    });
    
    view! {
        <div class="document-stats">
            <p>"Words: " {word_count}</p>
            <p>"Characters: " {char_count}</p>
            <p>"Paragraphs: " {paragraph_count}</p>
        </div>
    }
}
```

### 10.4. Asset Optimization

Asset optimization ensures that images and other assets load efficiently.

#### 10.4.1. Image Optimization

Use optimized images and lazy loading for improved performance.

**Image Optimization Pattern:**
```rust
#[component]
pub fn DocumentImage(src: String) -> impl IntoView {
    let (is_loaded, set_is_loaded) = create_signal(false);
    
    // Handle image load
    let handle_load = move |_| {
        set_is_loaded.set(true);
    };
    
    view! {
        <img
            class="document-image"
            src=src
            loading="lazy"
            on:load=handle_load
            style=format!(
                "opacity: {};",
                if is_loaded.get() { "1" } else { "0" }
            )
        />
    }
}
```

#### 10.4.2. Font Optimization

Use optimized fonts and font loading strategies.

**Font Optimization Pattern:**
```css
/* styles.css */
@font-face {
    font-family: 'Inter';
    src: url('/fonts/inter-variable.woff2') format('woff2-variations');
    font-weight: 100 900;
    font-display: swap;
    unicode-range: U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+2000-206F, U+2074, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD;
}

body {
    font-family: 'Inter', system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif;
}
```

### 10.5. Performance Monitoring

Performance monitoring ensures that components meet performance targets.

#### 10.5.1. Performance Metrics

Track key performance metrics for components.

**Performance Metrics Pattern:**
```rust
/// Performance metrics tracker.
pub struct PerformanceMetrics {
    pub render_time: Duration,
    pub update_count: u32,
    pub dom_updates: u32,
}

impl PerformanceMetrics {
    /// Create new performance metrics tracker.
    pub fn new() -> Self {
        Self {
            render_time: Duration::ZERO,
            update_count: 0,
            dom_updates: 0,
        }
    }
    
    /// Record render time.
    pub fn record_render(&mut self, duration: Duration) {
        self.render_time = duration;
    }
    
    /// Record update.
    pub fn record_update(&mut self) {
        self.update_count += 1;
    }
    
    /// Record DOM update.
    pub fn record_dom_update(&mut self) {
        self.dom_updates += 1;
    }
}

#[component]
pub fn PerformanceMonitoredComponent() -> impl IntoView {
    let metrics = create_signal(PerformanceMetrics::new());
    
    // Track render time
    let start_time = create_signal(Instant::now());
    
    create_effect(move |_| {
        let start = start_time.get();
        let duration = start.elapsed();
        metrics.update(|m| {
            m.record_render(duration);
        });
    });
    
    view! {
        <div class="component">
            <!-- Component content -->
        </div>
    }
}
```

### 10.6. Performance Best Practices

Follow these best practices for component performance:

1. **Use Fine-Grained Reactivity:** Leverage Leptos's fine-grained reactivity to minimize DOM updates.

2. **Lazy Load Components:** Lazy load components that are not immediately visible.

3. **Use Virtual Scrolling:** Use virtual scrolling for large lists.

4. **Memoize Expensive Computations:** Memoize expensive computations to avoid recalculations.

5. **Optimize Images:** Optimize images and use lazy loading.

6. **Optimize Fonts:** Use optimized fonts and font loading strategies.

7. **Monitor Performance:** Monitor component performance to identify bottlenecks.

8. **Use Code Splitting:** Use code splitting to load routes on demand.

9. **Minimize Bundle Size:** Minimize bundle size through tree shaking and minification.

10. **Use Web Workers:** Use Web Workers for CPU-intensive operations.

### 10.7. Performance Targets

Components must meet the following performance targets:

| Metric | Target | Measurement Method |
|---------|--------|-------------------|
| **First Contentful Paint** | < 1 second | Lighthouse |
| **Time to Interactive** | < 2 seconds | Lighthouse |
| **Largest Contentful Paint** | < 2.5 seconds | Lighthouse |
| **Cumulative Layout Shift** | < 0.1 | Lighthouse |
| **Total Blocking Time** | < 200 ms | Lighthouse |
| **Script Evaluation Time** | < 50 ms | Custom |
| **DOM Update Time** | < 16 ms | Custom |

### 10.8. TypeScript Interface Definitions

**PerformanceMetrics Type Definition:**
```typescript
/**
 * Performance metrics type.
 */
export interface PerformanceMetrics {
    renderTime: number;
    updateCount: number;
    domUpdates: number;
}
```

**VirtualListProps Type Definition:**
```typescript
/**
 * Props for VirtualList component.
 *
 * @template T - The type of items in the list
 */
export interface VirtualListProps<T> {
    items: ReadSignal<T[]>;
    itemHeight: number;
}
```

---

---

## 11. REFERENCES

This section provides references to related documents, standards, and external resources.

### 11.1. Related Documents

| Document ID | Title | Location |
|-------------|-------|----------|
| [TACHYON-STD-V1.0](../.specs/01_standards/coding_standards.md) | TACHYON: CODING AND DOCUMENTATION STANDARDS | `.specs/01_standards/coding_standards.md` |
| [TACHYON-REQ-WEB-V1.0](../.specs/04_future_state/reqs/web_requirements.md) | TACHYON: WEB FRONTEND REQUIREMENTS | `.specs/04_future_state/reqs/web_requirements.md` |
| [TACHYON-DSN-WEB-V1.0](../.specs/04_future_state/design/web_design.md) | TACHYON: WEB FRONTEND DESIGN | `.specs/04_future_state/design/web_design.md` |
| [TACHYON-TMA-V1.0](../.specs/03_threat_model/analysis.md) | TACHYON: THREAT MODEL ANALYSIS | `.specs/03_threat_model/analysis.md` |
| [TACHYON-TSK-V1.0](../.specs/tasks.md) | TACHYON: EXECUTION TASKS AND WORK BREAKDOWN STRUCTURE | `.specs/tasks.md` |

### 11.2. Architectural Decision Records

| ADR ID | Title | Location |
|---------|-------|----------|
| [TACHYON-ADR-004-V1.0](../.specs/02_adrs/004_leptos_for_web_frontend.md) | ADR-004: Leptos for Web Frontend | `.specs/02_adrs/004_leptos_for_web_frontend.md` |
| [TACHYON-ADR-005-V1.0](../.specs/02_adrs/005_bun_for_javascript_runtime.md) | ADR-005: Bun for JavaScript Runtime | `.specs/02_adrs/005_bun_for_javascript_runtime.md` |
| [TACHYON-ADR-009-V1.0](../.specs/02_adrs/009_ipc_communication_architecture.md) | ADR-009: IPC Communication Architecture | `.specs/02_adrs/009_ipc_communication_architecture.md` |
| [TACHYON-ADR-010-V1.0](../.specs/02_adrs/010_security_architecture.md) | ADR-010: Security Architecture | `.specs/02_adrs/010_security_architecture.md` |

### 11.3. Technology References

| Technology | Version | Documentation URL |
|------------|---------|------------------|
| **Leptos** | v0.8.15 | https://book.leptos.dev/ |
| **leptos_axum** | v0.8.7 | https://github.com/leptos-rs/leptos_axum |
| **leptos_router** | v0.8.11 | https://github.com/leptos-rs/leptos_router |
| **leptos_meta** | v0.8.5 | https://github.com/leptos-rs/leptos_meta |
| **Bun** | Latest stable | https://bun.sh/ |
| **Vite** | v7.3.1 | https://vitejs.dev/ |
| **TailwindCSS** | v4.1.18 | https://tailwindcss.com/ |
| **TypeScript** | Latest | https://www.typescriptlang.org/ |
| **WebAssembly** | Latest | https://webassembly.org/ |

### 11.4. Standards References

| Standard | Version | Description |
|----------|---------|-------------|
| **ISO/IEC 26514:2021** | Systems and software engineering — Requirements for designers and developers of user documentation | https://www.iso.org/standard/iso-iec-26514-2021 |
| **IEEE 829-2008** | IEEE Standard for Software Test Documentation | https://standards.ieee.org/standard/829-2008 |
| **WCAG 2.1** | Web Content Accessibility Guidelines (WCAG) 2.1 | https://www.w3.org/WAI/WCAG21/quickref/ |
| **RFC 7540** | Hypertext Transfer Protocol Version 2 (HTTP/2) | https://datatracker.ietf.org/doc/html/rfc7540 |
| **RFC 6265** | The WebSocket Protocol | https://datatracker.ietf.org/doc/html/rfc6265 |

### 11.5. Security References

| Reference | Description | URL |
|-----------|-------------|-----|
| **OWASP Top 10** | OWASP Top Ten Web Application Security Risks | https://owasp.org/www-project-top-ten |
| **OWASP XSS Prevention Cheat Sheet** | Cross-Site Scripting (XSS) Prevention Cheat Sheet | https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet |
| **OWASP CSRF Prevention Cheat Sheet** | Cross-Site Request Forgery (CSRF) Prevention Cheat Sheet | https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Request_Forgery_Prevention_Cheat_Sheet |
| **CSP Evaluator** | Content Security Policy Evaluator | https://csp-evaluator.withgoogle.com/ |

### 11.6. Performance References

| Reference | Description | URL |
|-----------|-------------|-----|
| **Web Vitals** | Essential metrics for a healthy site | https://web.dev/vitals |
| **Lighthouse** | Automated auditing, performance metrics, and best practices for the web | https://developers.google.com/web/tools/lighthouse |
| **WebPageTest** | Website performance and accessibility analysis tool | https://pagespeed.web.dev/ |

### 11.7. Requirement Traceability

This specification implements the following requirements from the Tachyon requirements specification:

| Requirement ID | Title | Implementation Section |
|---------------|-------|------------------------|
| **REQ-WEB-001** | Leptos Framework | Section 1.3 |
| **REQ-WEB-002** | SSR Support | Section 1.3 |
| **REQ-WEB-003** | Hydration | Section 1.3 |
| **REQ-WEB-004** | WASM Compilation | Section 1.3 |
| **REQ-WEB-005** | Bundle Optimization | Section 1.3 |
| **REQ-WEB-011** | Sidebar Navigation | Section 3.1 |
| **REQ-WEB-012** | Breadcrumb Navigation | Section 3.1 |
| **REQ-WEB-013** | Quick Search | Section 3.1 |
| **REQ-WEB-014** | User Menu | Section 3.1 |
| **REQ-WEB-015** | Mobile Navigation | Section 3.1 |
| **REQ-WEB-016** | Document Content | Section 3.1 |
| **REQ-WEB-017** | Table of Contents | Section 3.1 |
| **REQ-WEB-018** | Document Metadata | Section 3.1 |
| **REQ-WEB-019** | Edit Button | Section 3.1 |
| **REQ-WEB-020** | Share Button | Section 3.1 |
| **REQ-WEB-021** | Search Results | Section 5.1 |
| **REQ-WEB-022** | Search Filters | Section 5.1 |
| **REQ-WEB-023** | Search History | Section 5.1 |
| **REQ-WEB-024** | Advanced Search | Section 5.1 |
| **REQ-WEB-025** | Search Empty State | Section 5.1 |
| **REQ-WEB-026** | Reactive State | Section 2.3 |
| **REQ-WEB-027** | Global Store | Section 2.3 |
| **REQ-WEB-028** | Local Storage | Section 2.3 |
| **REQ-WEB-029** | Session Storage | Section 2.3 |
| **REQ-WEB-030** | State Synchronization | Section 2.3 |
| **REQ-WEB-031** | Settings Persistence | Section 2.3 |
| **REQ-WEB-032** | Editor State | Section 2.3 |
| **REQ-WEB-033** | Form State | Section 2.3 |
| **REQ-WEB-034** | Offline Cache | Section 2.3 |
| **REQ-WEB-035** | Cache Invalidation | Section 2.3 |
| **REQ-WEB-036** | HTTP/2 Client | Section 5.1 |
| **REQ-WEB-037** | API Client | Section 5.1 |
| **REQ-WEB-038** | Error Handling | Section 5.1 |
| **REQ-WEB-039** | Request Cancellation | Section 5.1 |
| **REQ-WEB-040** | Request Deduplication | Section 5.1 |
| **REQ-WEB-041** | WebSocket Client | Section 5.1 |
| **REQ-WEB-042** | Auto-Reconnection | Section 5.1 |
| **REQ-WEB-043** | Message Handling | Section 5.1 |
| **REQ-WEB-044** | Connection Status | Section 5.1 |
| **REQ-WEB-045** | Message Queue | Section 5.1 |
| **REQ-WEB-046** | Live Updates | Section 5.1 |
| **REQ-WEB-047** | User Presence | Section 5.1 |
| **REQ-WEB-048** | Conflict Notifications | Section 5.1 |
| **REQ-WEB-049** | Typing Indicators | Section 5.1 |
| **REQ-WEB-050** | Cursor Position | Section 5.1 |
| **REQ-WEB-051** | Content Editable | Section 3.2 |
| **REQ-WEB-052** | Syntax Highlighting | Section 3.2 |
| **REQ-WEB-053** | Live Preview | Section 3.2 |
| **REQ-WEB-054** | Auto-Save | Section 3.2 |
| **REQ-WEB-055** | Formatting Toolbar | Section 3.2 |
| **REQ-WEB-056** | Code Block Support | Section 3.2 |
| **REQ-WEB-057** | Image Embedding | Section 3.2 |
| **REQ-WEB-058** | Link Creation | Section 3.2 |
| **REQ-WEB-059** | Table Support | Section 3.2 |
| **REQ-WEB-060** | Math Support | Section 3.2 |
| **REQ-WEB-061** | Mobile Toolbar | Section 3.2 |
| **REQ-WEB-062** | Debounced Highlighting | Section 3.2 |
| **REQ-WEB-063** | Touch Optimization | Section 3.2 |
| **REQ-WEB-064** | Virtual Keyboard Handling | Section 3.2 |
| **REQ-WEB-065** | Responsive Layout | Section 3.2 |
| **REQ-WEB-066** | First Contentful Paint | Section 10.1 |
| **REQ-WEB-067** | Time to Interactive | Section 10.1 |
| **REQ-WEB-068** | Smooth Scrolling | Section 10.1 |
| **REQ-WEB-069** | Animation Performance | Section 10.1 |
| **REQ-WEB-070** | Lazy Loading | Section 10.1 |
| **REQ-WEB-071** | Code Splitting | Section 10.2 |
| **REQ-WEB-072** | Tree Shaking | Section 10.2 |
| **REQ-WEB-073** | Minification | Section 10.2 |
| **REQ-WEB-074** | Compression | Section 10.2 |
| **REQ-WEB-075** | Asset Caching | Section 10.2 |
| **REQ-WEB-076** | Full Keyboard Support | Section 2.5 |
| **REQ-WEB-077** | Focus Management | Section 2.5 |
| **REQ-WEB-078** | Skip Links | Section 2.5 |
| **REQ-WEB-079** | Focus Indicators | Section 2.5 |
| **REQ-WEB-080** | Keyboard Shortcuts | Section 2.5 |
| **REQ-WEB-081** | ARIA Labels | Section 2.5 |
| **REQ-WEB-082** | Semantic HTML | Section 2.5 |
| **REQ-WEB-083** | Live Regions | Section 2.5 |
| **REQ-WEB-084** | Alt Text | Section 2.5 |
| **REQ-WEB-085** | Heading Structure | Section 2.5 |
| **REQ-WEB-086** | High Contrast Mode | Section 2.5 |
| **REQ-WEB-087** | Font Scaling | Section 2.5 |
| **REQ-WEB-088** | Color Independence | Section 2.5 |
| **REQ-WEB-089** | Reduced Motion | Section 2.5 |
| **REQ-WEB-090** | Text Resizing | Section 2.5 |

### 11.8. Design Element Traceability

This specification implements the following design elements from the Tachyon web design specification:

| Design Element ID | Title | Implementation Section |
|------------------|-------|------------------------|
| **DES-WD-001** | ApplicationState | Section 2.3 |
| **DES-WD-002** | DocumentState | Section 2.3 |
| **DES-WD-003** | RepositoryState | Section 2.3 |
| **DES-WD-004** | UIState | Section 2.3 |
| **DES-WD-005** | WASMExports | Section 2.3 |
| **DES-WD-006** | ApiClient | Section 5.1 |
| **DES-WD-007** | WebSocketClient | Section 5.1 |

### 11.9. ADR Traceability

This specification implements the following architectural decisions:

| ADR ID | Decision | Implementation Section |
|---------|---------|------------------------|
| **ADR-004** | Leptos for Web Frontend | Section 1.3 |
| **ADR-005** | Bun for JavaScript Runtime | Section 1.3 |
| **ADR-009** | IPC Communication Architecture | Section 5.1 |
| **ADR-010** | Security Architecture | Section 9.1 |

### 11.10. Threat Model Traceability

This specification implements the following threat mitigations:

| Threat Category | Mitigation | Implementation Section |
|---------------|-----------|------------------------|
| **Spoofing** | Multi-Factor Authentication, Secure Session Management | Section 9.1 |
| **Tampering** | Input Validation, Output Encoding, Content Security Policy | Section 9.3 |
| **Information Disclosure** | Error Handling, Secure Token Storage | Section 9.1 |
| **Denial of Service** | Rate Limiting, Request Deduplication | Section 5.1 |
| **Elevation of Privilege** | Role-Based Access Control, Resource-Based Access Control | Section 9.2 |

### 11.11. Version History

| Version | Date | Changes |
|---------|------|---------|
| **V1.0** | February 2026 | Initial version |

---

**END OF DOCUMENT**

**Document ID:** TACHYON-API-012-V1.0
**Status:** Proposed
**Classification:** API Specification Document
**Compliance Level:** ISO/IEC 26514:2021, IEEE 829-2008
