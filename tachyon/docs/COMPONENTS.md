# Tachyon Component Reference

This document provides a detailed reference for all UI components in the Tachyon frontend.

---

## Table of Contents

- [AppShell](#1-appshell)
- [DocumentEditor](#2-documenteditor)
- [VersionHistory](#3-versionhistory)
- [AttachmentManager](#4-attachmentmanager)
- [TemplateSelector](#5-templateselector)
- [ActivityFeed](#6-activityfeed)
- [RoleBadge](#7-rolebadge)
- [Common](#8-common)
- [Layout](#9-layout)
- [Catalog](#10-catalog)

---

## 1. AppShell

**File:** `crates/frontend/src/components/app_shell.rs`

The main application shell providing sidebar navigation, top header bar, and a content area. Wraps all page routes.

### Props

| Prop | Type | Description |
|------|------|-------------|
| `theme` | `ReadSignal<String>` | Current theme value (`"light"` or `"dark"`) |
| `toggle_theme` | `F: Fn() + 'static` | Callback invoked when the theme toggle button is clicked |
| `children` | `Children` | Slot for page content rendered inside the main area |

### Description

Provides the top-level layout structure:
- **Sidebar** (left): Logo, navigation links (Home, Documents, Search, Catalog, Settings), collapse/expand toggle. Collapses to a 64px icon-only rail.
- **Top header** (sticky): Title "Tachyon", theme toggle button, sign-in link.
- **Main content area**: Rendered inside `<main class="p-6">`.

Includes a private `NavLink` component for sidebar navigation items.

### API Endpoints Used

None directly. Contains a static sign-in link (`<a href="/login">`).

### Usage Example

```rust
use crate::components::AppShell;
use leptos::prelude::*;

let (theme, set_theme) = signal("light".to_string());
let toggle_theme = move || {
    let new_theme = if theme.get() == "light" { "dark" } else { "light" };
    set_theme.set(new_theme);
};

view! {
    <AppShell theme=theme toggle_theme=toggle_theme>
        <div>"Page content here"</div>
    </AppShell>
}
```

### Dependencies

- `leptos::prelude::*` (signals, `ReadSignal`, `Children`, `Show`)

---

## 2. DocumentEditor

**File:** `crates/frontend/src/components/document_editor.rs`

A real-time collaborative document editor with WebSocket-based operational transforms, presence indicators, and a status bar.

### Public Components

#### `DocumentEditor`

| Prop | Type | Description |
|------|------|-------------|
| `document_id` | `String` | ID of the document to edit |
| `user_id` | `String` | Current user's ID |
| `user_name` | `String` | Current user's display name |

#### `PresenceIndicators`

| Prop | Type | Description |
|------|------|-------------|
| `users` | `Vec<PresenceUser>` | List of users currently present |

### Description

- Connects via WebSocket for real-time collaboration.
- Sends character-level diff operations (insert, delete, replace) with a 300ms debounce.
- Receives edits from other users and applies them to the local content.
- Tracks presence (join/leave/presence messages) and displays colored avatar initials.
- Shows connection state indicator (Connected/Connecting/Reconnecting/Disconnected).
- Bottom status bar shows word count, character count, and save status.

### Public Types

```rust
pub struct PresenceUser {
    pub user_id: String,
    pub user_name: String,
    pub color: String,       // Deterministic color from user_id hash
}

pub struct EditorState {
    pub content: String,
    pub version: u64,
    pub is_saving: bool,
    pub last_saved: Option<String>,
    pub presence_users: Vec<PresenceUser>,
}
```

### API Endpoints Used

- **WebSocket:** `ws://<host>/ws` -- connect, join/leave document, send/receive edit operations

### Usage Example

```rust
use crate::components::{DocumentEditor, PresenceIndicators, PresenceUser};

view! {
    <DocumentEditor
        document_id="doc-123".into()
        user_id="user-456".into()
        user_name="Alice".into()
    />
}

// Standalone presence indicator
view! {
    <PresenceIndicators users=vec![
        PresenceUser { user_id: "u1".into(), user_name: "Alice".into(), color: "#3B82F6".into() },
        PresenceUser { user_id: "u2".into(), user_name: "Bob".into(), color: "#10B981".into() },
    ] />
}
```

### Dependencies

- `crate::api::ApiClient`
- `crate::websocket::{WebSocketClient, ConnectionState, DocumentEditMessage, EditOperation, WsMessage}`
- `leptos::prelude::*`, `leptos::task::spawn_local`
- `uuid`, `serde`, `wasm_bindgen`, `web_sys::HtmlTextAreaElement`

---

## 3. VersionHistory

**File:** `crates/frontend/src/components/version_history.rs`

Displays document version history with a diff comparison view and rollback capability.

### Public Components

#### `VersionHistory`

| Prop | Type | Description |
|------|------|-------------|
| `document_id` | `String` | ID of the document |
| `on_rollback` | `Option<Callback<String>>` | Optional callback fired with version ID when rollback is clicked |

#### `VersionDiffView`

| Prop | Type | Description |
|------|------|-------------|
| `document_id` | `String` | ID of the document |
| `version1` | `i32` | First version number to compare |
| `version2` | `i32` | Second version number to compare |

### Description

- Loads version list from the API via `LocalResource`.
- Each version entry shows: version badge (v1, v2, ...), timestamp, commit message, author, and a "Rollback" button.
- "Compare Versions" toggle switches the list to radio-button mode for selecting two versions.
- Side-by-side diff view uses a line-based LCS (Longest Common Subsequence) algorithm to compute added/removed/unchanged lines.
- Diff renders in two columns (Old Version / New Version) with color-coded backgrounds (green for additions, red for removals).

### API Endpoints Used

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/documents/{id}/versions` | List all versions |
| GET | `/api/v1/documents/{id}/versions/{number}` | Get specific version content (called twice for diff) |

### Usage Example

```rust
use crate::components::{VersionHistory, VersionDiffView};
use leptos::prelude::*;

let on_rollback = Callback::new(|version_id: String| {
    log::info!("Rollback to: {}", version_id);
});

view! {
    <VersionHistory
        document_id="doc-123".into()
        on_rollback=on_rollback
    />
}

// Standalone diff view
view! {
    <VersionDiffView
        document_id="doc-123".into()
        version1=1
        version2=3
    />
}
```

### Dependencies

- `crate::api::ApiClient`
- `crate::types::DocumentVersion`
- `leptos::prelude::*`
- `chrono` (timestamp formatting)
- `std::sync::{Arc, Mutex}`

---

## 4. AttachmentManager

**File:** `crates/frontend/src/components/attachments.rs`

File upload and management component for document attachments.

### Public Component

#### `AttachmentManager`

| Prop | Type | Description |
|------|------|-------------|
| `document_id` | `String` | ID of the document to manage attachments for |

### Description

- Provides a file input and "Upload" button for uploading files.
- Displays upload progress state (button text changes to "Uploading..." and is disabled).
- Shows error messages on upload failure.
- Lists all attachments with: filename, formatted file size, download link, and delete button.
- Automatically refreshes the attachment list after successful upload or delete.

### API Endpoints Used

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/documents/{id}/attachments` | List attachments |
| POST | `/api/v1/documents/{id}/attachments` | Upload file (multipart/form-data) |
| DELETE | `/api/v1/documents/{id}/attachments/{attachment_id}` | Delete attachment |

### Usage Example

```rust
use crate::components::AttachmentManager;

view! {
    <AttachmentManager document_id="doc-123".into() />
}
```

### Dependencies

- `crate::api::ApiClient`
- `crate::types::Attachment`
- `leptos::prelude::*`
- `wasm_bindgen_futures::spawn_local`
- `std::sync::{Arc, Mutex}`

---

## 5. TemplateSelector

**File:** `crates/frontend/src/components/template_selector.rs`

Document template browser with category filtering, card grid, and a preview modal.

### Public Components

#### `TemplateSelector`

| Prop | Type | Description |
|------|------|-------------|
| `on_select` | `Callback<DocumentTemplate>` | Fired when a template is selected (via card click or "Use Template" button) |
| `category` | `Option<String>` | Initial category filter (loads all if `None`) |

#### `TemplateCard`

| Prop | Type | Description |
|------|------|-------------|
| `template` | `DocumentTemplate` | The template to display |
| `on_select` | `Callback<DocumentTemplate>` | Fired when the card is clicked |

### Description

- Left sidebar lists template categories loaded from the API; "All Templates" shown by default.
- Main area shows a 2-column grid of template cards with name, description, tags, and "Use Template" button.
- Clicking a card opens a modal preview showing the full template content in a `<pre>` block.
- Modal provides "Use Template" and "Cancel" buttons.
- Category selection filters the template list reactively via `LocalResource`.

### API Endpoints Used

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/templates?category={cat}` | List templates (optionally filtered) |
| GET | `/api/v1/templates/categories` | List all template categories |

### Usage Example

```rust
use crate::components::{TemplateSelector, TemplateCard};
use crate::types::DocumentTemplate;
use leptos::prelude::*;

let on_select = Callback::new(|template: DocumentTemplate| {
    log::info!("Selected template: {}", template.name);
});

// Full selector with category sidebar
view! {
    <TemplateSelector
        on_select=on_select
        category=None
    />
}

// Standalone card
view! {
    <TemplateCard
        template=DocumentTemplate { /* ... */ }
        on_select=on_select
    />
}
```

### Dependencies

- `crate::api::ApiClient`
- `crate::types::DocumentTemplate`
- `leptos::prelude::*`

---

## 6. ActivityFeed

**File:** `crates/frontend/src/components/activity_feed.rs`

Activity log display with filtering, available in full and compact variants.

### Public Components

#### `ActivityFeed`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `activities` | `Vec<Activity>` | -- | List of activity items to display |
| `filter` | `Option<String>` | `None` | Initial filter type (e.g., `"all"`, `"edit"`, `"comment"`) |
| `max_items` | `Option<usize>` | `None` | Maximum number of items to display (unlimited if `None`) |

#### `ActivityFeedCompact`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `activities` | `Vec<Activity>` | -- | List of activity items to display |
| `max_items` | `usize` | `5` | Maximum items to show |

### Description

- **ActivityFeed**: Full sidebar-style feed with filter buttons (All, Edits, Comments, Presence). Each item shows an icon, username, relative timestamp, description, and optional document link.
- **ActivityFeedCompact**: Compact card-style feed showing icon, description, and relative timestamp in a single row.

### Public Types

```rust
pub enum ActivityType {
    Edit, Comment, Join, Leave, Create, Delete, Publish,
}

pub struct Activity {
    pub id: String,
    pub activity_type: ActivityType,
    pub user_id: String,
    pub user_name: String,
    pub document_id: Option<String>,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}
```

`Activity` provides a builder pattern:

```rust
let activity = Activity::new(ActivityType::Edit, "user-1".into(), "Alice".into(), "Edited doc".into())
    .with_document("doc-123".into())
    .with_metadata(serde_json::json!({"words": 150}));
```

### API Endpoints Used

None directly -- receives activities as props. Activities are typically fetched by parent components via WebSocket or REST.

### Usage Example

```rust
use crate::components::{ActivityFeed, ActivityFeedCompact};
use crate::components::{Activity, ActivityType};
use chrono::Utc;

let activities = vec![
    Activity::new(ActivityType::Edit, "u1".into(), "Alice".into(), "Edited Introduction".into()),
    Activity::new(ActivityType::Comment, "u2".into(), "Bob".into(), "Left a comment".into()),
];

// Full feed with filtering
view! {
    <ActivityFeed
        activities=activities.clone()
        filter=Some("all".into())
        max_items=Some(50)
    />
}

// Compact card
view! {
    <ActivityFeedCompact activities=activities max_items=5 />
}
```

### Dependencies

- `chrono::{DateTime, Utc}`
- `leptos::prelude::*`
- `serde::{Deserialize, Serialize}`

---

## 7. RoleBadge

**File:** `crates/frontend/src/components/role_badge.rs`

User role display badges, permission indicators, and role-based visibility wrappers for RBAC.

### Public Components

#### `RoleBadge`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `role` | `UserRole` | -- | The user role to display |
| `size` | `Option<String>` | `None` | Badge size: `"sm"`, `"lg"`, or default medium |

#### `PermissionBadge`

| Prop | Type | Description |
|------|------|-------------|
| `permission` | `Permission` | The permission level to display |

#### `RoleBasedVisibility`

| Prop | Type | Description |
|------|------|-------------|
| `current_role` | `UserRole` | The current user's role |
| `required_permission` | `Permission` | Minimum permission required to see children |
| `children` | `Children` | Content only visible if role has sufficient permission |

#### `AdminOnly`

| Prop | Type | Description |
|------|------|-------------|
| `current_role` | `UserRole` | Current user's role |
| `children` | `Children` | Content visible only to admin+ users |

#### `OwnerOnly`

| Prop | Type | Description |
|------|------|-------------|
| `current_role` | `UserRole` | Current user's role |
| `children` | `Children` | Content visible only to owner users |

### Description

- **RoleBadge**: Renders a colored pill with the role name. Colors are automatically assigned by role name (owner=purple, admin=red, editor=orange, writer=blue, reader=gray).
- **PermissionBadge**: Displays a permission level with an emoji icon and label.
- **RoleBasedVisibility**: Conditionally renders children based on permission hierarchy.
- **AdminOnly / OwnerOnly**: Convenience wrappers around `RoleBasedVisibility`.

### Public Types

```rust
pub enum Permission {
    Read,    // level 1
    Write,   // level 2
    Delete,  // level 3
    Admin,   // level 4
    Owner,   // level 5
}

pub struct UserRole {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,   // e.g., ["read", "write", "admin"]
    pub is_system: bool,
}
```

Permission hierarchy: `Owner > Admin > Delete > Write > Read`. `has_permission` checks if any of the role's permissions have a level >= the required permission.

### API Endpoints Used

None directly -- roles are typically loaded via auth endpoints and passed as props.

### Usage Example

```rust
use crate::components::role_badge::*;
use leptos::prelude::*;

let role = UserRole {
    id: 1,
    name: "admin".into(),
    description: Some("Administrator".into()),
    permissions: vec!["read".into(), "write".into(), "admin".into()],
    is_system: true,
};

view! {
    <RoleBadge role=role.clone() size=Some("sm".into()) />
    <PermissionBadge permission=Permission::Write />
    <RoleBasedVisibility current_role=role.clone() required_permission=Permission::Write>
        <p>"Only visible to users with Write permission or higher"</p>
    </RoleBasedVisibility>
    <AdminOnly current_role=role.clone()>
        <p>"Admin-only content"</p>
    </AdminOnly>
}
```

### Dependencies

- `leptos::prelude::*`
- `serde::{Deserialize, Serialize}`

---

## 8. Common

**File:** `crates/frontend/src/components/common.rs`

A collection of shared, reusable UI primitives.

### Components

#### `Button`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `text` | `String` | -- | Button label text |
| `variant` | `String` | `"primary"` | Visual style: `"primary"`, `"secondary"`, or `"danger"` |

#### `Card`

| Prop | Type | Description |
|------|------|-------------|
| `title` | `String` | Card header title |
| `children` | `Children` | Card body content |

#### `LoadingSpinner`

No props. Renders a spinning blue circle.

#### `StatusBadge`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `status` | `String` | -- | Badge text |
| `color` | `String` | `"gray"` | Color theme: `"green"`, `"yellow"`, `"red"`, `"blue"`, `"purple"`, `"gray"` |

#### `EmptyState`

| Prop | Type | Description |
|------|------|-------------|
| `title` | `String` | Message to display in the empty state |

#### `Grid`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `cols` | `u32` | `3` | Number of columns (1-6) |
| `gap` | `u32` | `4` | Tailwind gap unit |
| `children` | `Children` | Grid items |

#### `PageHeader`

| Prop | Type | Description |
|------|------|-------------|
| `title` | `String` | Page title (rendered as `<h1>`) |

### API Endpoints Used

None.

### Usage Example

```rust
use crate::components::common::*;

view! {
    <PageHeader title="Dashboard".into() />
    <Grid cols=2 gap=6>
        <Card title="Stats".into()>
            <p>"Content here"</p>
        </Card>
        <Card title="Activity".into()>
            <StatusBadge status="Active".into() color="green".into() />
        </Card>
    </Grid>
    <Button text="Save".into() variant="primary".into() />
    <Button text="Delete".into() variant="danger".into() />
    <LoadingSpinner />
    <EmptyState title="No data available".into() />
}
```

### Dependencies

- `leptos::prelude::*`

---

## 9. Layout

**File:** `crates/frontend/src/components/layout.rs`

Placeholder module for page layout components. Currently empty.

### Status

Reserved for future layout primitives (e.g., two-column layouts, page sections). Layout is currently handled directly in `AppShell`.

---

## 10. Catalog

**File:** `crates/frontend/src/components/catalog.rs`

Placeholder module for catalog browsing components.

### Status

Reserved for future catalog UI (project cards, component lists, filter panels). Catalog pages currently use `crates/frontend/src/pages/catalog.rs` directly.

### Expected Future Components

- Project card grid
- Component list with type filters
- Lifecycle stage indicators
- Project detail view
