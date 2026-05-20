# Database Schema

Documentation of Tachyon's SQLite database schema.

## Overview

Tachyon uses SQLite for metadata storage:
- Document metadata
- User sessions
- Audit logs
- Configuration

## Entity-Relationship Diagram

```
┌─────────────────┐       ┌─────────────────┐
│    documents    │       │      users      │
├─────────────────┤       ├─────────────────┤
│ id (PK)         │       │ id (PK)         │
│ title           │       │ username        │
│ path            │       │ email           │
│ status          │       │ role            │
│ visibility      │       │ created_at      │
│ created_at      │       │ last_login      │
│ updated_at      │       └─────────────────┘
│ author_id (FK)  │───────│
└─────────────────┘       │
                          │
┌─────────────────┐       │
│    sessions     │       │
├─────────────────┤       │
│ id (PK)         │       │
│ user_id (FK)    │───────┘
│ token           │
│ expires_at      │
│ created_at      │
└─────────────────┘

┌─────────────────┐       ┌─────────────────┐
│     groups      │       │  group_members  │
├─────────────────┤       ├─────────────────┤
│ id (PK)         │       │ group_id (FK)   │
│ name            │───────│ user_id (FK)    │
│ description     │       │ joined_at       │
│ created_at      │       └─────────────────┘
└─────────────────┘

┌─────────────────┐
│   audit_log     │
├─────────────────┤
│ id (PK)         │
│ user_id (FK)    │
│ action          │
│ resource        │
│ details         │
│ created_at      │
└─────────────────┘
```

## Tables

### documents

```sql
CREATE TABLE documents (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'draft',
    visibility TEXT NOT NULL DEFAULT 'public',
    access_level TEXT,
    author_id TEXT REFERENCES users(id),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    metadata JSON
);

CREATE INDEX idx_documents_status ON documents(status);
CREATE INDEX idx_documents_visibility ON documents(visibility);
CREATE INDEX idx_documents_author ON documents(author_id);
CREATE INDEX idx_documents_updated ON documents(updated_at);
```

### users

```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    display_name TEXT,
    role TEXT NOT NULL DEFAULT 'viewer',
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_login TIMESTAMP,
    preferences JSON
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role ON users(role);
```

### sessions

```sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token TEXT NOT NULL UNIQUE,
    ip_address TEXT,
    user_agent TEXT,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_sessions_user ON sessions(user_id);
CREATE INDEX idx_sessions_token ON sessions(token);
CREATE INDEX idx_sessions_expires ON sessions(expires_at);
```

### groups

```sql
CREATE TABLE groups (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    display_name TEXT,
    description TEXT,
    parent_id TEXT REFERENCES groups(id),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_groups_name ON groups(name);
CREATE INDEX idx_groups_parent ON groups(parent_id);
```

### group_members

```sql
CREATE TABLE group_members (
    group_id TEXT NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'member',
    joined_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (group_id, user_id)
);

CREATE INDEX idx_group_members_user ON group_members(user_id);
```

### document_groups

```sql
CREATE TABLE document_groups (
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    group_id TEXT NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    permission TEXT NOT NULL DEFAULT 'read',
    PRIMARY KEY (document_id, group_id)
);

CREATE INDEX idx_document_groups_group ON document_groups(group_id);
```

### audit_log

```sql
CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT REFERENCES users(id),
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT,
    details JSON,
    ip_address TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_audit_user ON audit_log(user_id);
CREATE INDEX idx_audit_action ON audit_log(action);
CREATE INDEX idx_audit_resource ON audit_log(resource_type, resource_id);
CREATE INDEX idx_audit_created ON audit_log(created_at);
```

### tags

```sql
CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_tags_name ON tags(name);
```

### document_tags

```sql
CREATE TABLE document_tags (
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (document_id, tag_id)
);

CREATE INDEX idx_document_tags_tag ON document_tags(tag_id);
```

## Migrations

Migrations are versioned and applied automatically:

```
migrations/
├── 001_initial.sql
├── 002_add_audit_log.sql
├── 003_add_tags.sql
└── 004_add_document_groups.sql
```

### Example Migration

```sql
-- 002_add_audit_log.sql

-- Up
CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT REFERENCES users(id),
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT,
    details JSON,
    ip_address TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_audit_user ON audit_log(user_id);
CREATE INDEX idx_audit_action ON audit_log(action);
CREATE INDEX idx_audit_created ON audit_log(created_at);

-- Down
DROP TABLE audit_log;
```

## Query Patterns

### Document with Author

```sql
SELECT d.*, u.username as author_name
FROM documents d
LEFT JOIN users u ON d.author_id = u.id
WHERE d.id = ?;
```

### User Groups

```sql
SELECT g.*
FROM groups g
JOIN group_members gm ON g.id = gm.group_id
WHERE gm.user_id = ?;
```

### Documents by Group

```sql
SELECT d.*
FROM documents d
JOIN document_groups dg ON d.id = dg.document_id
WHERE dg.group_id = ? AND dg.permission IN ('read', 'write');
```

### Audit Trail

```sql
SELECT al.*, u.username
FROM audit_log al
LEFT JOIN users u ON al.user_id = u.id
WHERE al.resource_type = ? AND al.resource_id = ?
ORDER BY al.created_at DESC
LIMIT 100;
```

## Performance Considerations

### Indexing Strategy

- All foreign keys indexed
- Status/visibility for filtering
- Timestamps for sorting
- Full-text search via Tantivy (not SQLite FTS)

### Connection Pooling

```rust
let manager = SqliteConnectionManager::file("tachyon.db");
let pool = Pool::builder()
    .max_size(10)
    .build(manager)?;
```

### Write Optimization

- WAL mode for concurrent reads
- Batch inserts for bulk operations
- Periodic VACUUM for space reclamation

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -64000; -- 64MB
```
