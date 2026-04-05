# Permissions and Roles

Comprehensive guide to access control in Tachyon.

## Overview

Tachyon implements Role-Based Access Control (RBAC) with:

- Hierarchical roles
- Group-based permissions
- Document-level access control
- Block-level content redaction

## Role Hierarchy

### Built-in Roles

Roles are hierarchical - higher roles inherit lower permissions:

```
Admin
  └── Reviewer
        └── Editor
              └── Commenter
                    └── Viewer
```

### Role Definitions

| Role | Description |
|------|-------------|
| **Admin** | Full system access |
| **Reviewer** | Review and approve content |
| **Editor** | Create and edit documents |
| **Commenter** | Add comments to documents |
| **Viewer** | Read-only access |

## Permissions by Role

### Viewer

| Permission | Description |
|------------|-------------|
| `documents:read` | View public documents |
| `documents:search` | Search public documents |
| `documents:history` | View document history |
| `profile:read` | View own profile |

### Commenter

All Viewer permissions, plus:

| Permission | Description |
|------------|-------------|
| `comments:create` | Add comments |
| `comments:edit:own` | Edit own comments |
| `comments:delete:own` | Delete own comments |
| `documents:watch` | Watch documents |

### Editor

All Commenter permissions, plus:

| Permission | Description |
|------------|-------------|
| `documents:create` | Create new documents |
| `documents:edit` | Edit documents |
| `documents:delete:own` | Delete own documents |
| `documents:upload` | Upload attachments |
| `documents:export` | Export documents |

### Reviewer

All Editor permissions, plus:

| Permission | Description |
|------------|-------------|
| `documents:publish` | Publish documents |
| `documents:delete:any` | Delete any document |
| `documents:approve` | Approve changes |
| `comments:resolve` | Resolve comment threads |
| `documents:archive` | Archive documents |

### Admin

All Reviewer permissions, plus:

| Permission | Description |
|------------|-------------|
| `users:*` | Full user management |
| `groups:*` | Full group management |
| `roles:*` | Manage roles and permissions |
| `settings:*` | System configuration |
| `audit:read` | View audit logs |
| `backup:*` | Backup and restore |

## Custom Roles

### Defining Custom Roles

Create custom roles in `tachyon.toml`:

```toml
[[roles]]
name = "tech-writer"
display_name = "Technical Writer"
inherits = "editor"
permissions = [
  "documents:publish",
  "documents:template"
]

[[roles]]
name = "project-lead"
display_name = "Project Lead"
inherits = "reviewer"
permissions = [
  "documents:delete:any",
  "groups:read"
]
```

### Permission Format

```
resource:action[:scope]
```

Examples:
- `documents:read` - Read all documents
- `documents:edit:own` - Edit own documents only
- `documents:delete:group` - Delete group documents

### Available Permissions

| Resource | Actions |
|----------|---------|
| `documents` | read, create, edit, delete, publish, archive, export, template |
| `comments` | create, edit, delete, resolve |
| `users` | read, create, edit, delete |
| `groups` | read, create, edit, delete |
| `roles` | read, assign |
| `settings` | read, write |
| `audit` | read |
| `backup` | create, restore |

## Document Access Control

### Access Levels

| Level | Description |
|-------|-------------|
| `public` | All authenticated users |
| `internal` | All internal users (excludes external) |
| `restricted` | Specific groups only |
| `private` | Owner and admins only |

### Setting Document Access

Via frontmatter:

```yaml
---
title: API Documentation
access: restricted
groups: [engineering, product]
---
```

### Access Matrix

| Access Level | Viewer | Commenter | Editor | Reviewer | Admin |
|--------------|--------|-----------|--------|----------|-------|
| public | ✓ | ✓ | ✓ | ✓ | ✓ |
| internal | ✗ | ✓ | ✓ | ✓ | ✓ |
| restricted | ✗ | ✗ | ✓* | ✓* | ✓ |
| private | ✗ | ✗ | ✗ | ✗ | ✓ |

*Only if in specified groups

### Folder Permissions

Set default permissions for folders:

1. Navigate to folder
2. Click **...** > **Permissions**
3. Set:
   - Default access level
   - Allowed groups
   - Inherit to subfolders

## Group-Based Access

### Creating Access Groups

```toml
[[groups]]
name = "engineering"
display_name = "Engineering Team"
permissions = []

[[groups]]
name = "security-review"
display_name = "Security Reviewers"
permissions = ["documents:approve"]
```

### Assigning Documents to Groups

```yaml
---
title: Security Procedures
access: restricted
groups: [security-review, engineering]
---
```

### Group Inheritance

Groups can inherit from parent groups:

```yaml
groups:
  - name: engineering
    children:
      - name: backend
      - name: frontend
```

Members of `backend` inherit `engineering` access.

## Block-Level Redaction

### Using Redaction Blocks

Hide sensitive content from unauthorized users:

```markdown
# API Reference

This endpoint is available to all users.

::: internal
**Internal Note:** The production API key is stored in Vault.
API Key: sk-prod-xxxxx
:::

::: security
**Security Classification: CONFIDENTIAL**
This section contains sensitive security information.
:::

Public documentation continues here...
```

### Redaction Block Types

| Block Type | Who Sees |
|------------|----------|
| `internal` | Members of `internal` group |
| `security` | Members of `security` group |
| `admin` | Admin role only |
| `custom:{group}` | Members of specified group |

### Configuring Redaction

```toml
[security.redaction]
default_behavior = "hide"  # hide | placeholder
placeholder_text = "[REDACTED]"
audit_redactions = true
```

### Audit Trail

Redaction access is logged:
- Who accessed redacted content
- When
- Which blocks

## Permission Checks

### How Permissions Work

1. User attempts action
2. System checks user's role
3. System checks group memberships
4. System checks document access
5. If any check fails, access denied

### Permission Resolution

```
User has:
- Role: Editor
- Groups: engineering, project-alpha

Document has:
- Access: restricted
- Groups: engineering, product

Result: ALLOW (engineering group match)
```

### Explicit Deny

Deny always takes precedence:

```yaml
---
access: restricted
groups: [engineering]
deny: [contractors]  # Explicitly deny even if in engineering
---
```

## API Permission Model

### Token Scopes

API tokens have scopes, not roles:

| Scope | Equivalent Role |
|-------|-----------------|
| `read` | Viewer |
| `write` | Editor |
| `review` | Reviewer |
| `admin` | Admin |

### Scope Combinations

```bash
# Read-only token
tachyon token create --scopes read

# Full access token
tachyon token create --scopes read,write,review,admin
```

### Permission Errors

API returns structured errors:

```json
{
  "error": {
    "code": "FORBIDDEN",
    "message": "Insufficient permissions",
    "required": ["documents:publish"],
    "actual": ["documents:edit"]
  }
}
```

## Best Practices

### Least Privilege

- Assign minimum required role
- Use groups for access control
- Avoid individual permissions

### Regular Audits

- Review user roles quarterly
- Audit group memberships
- Check for orphaned accounts

### Separation of Duties

- Don't combine admin with content roles
- Separate reviewers from publishers
- Use different groups for sensitive access

### Documentation

- Document role definitions
- Document group purposes
- Document access policies

## Troubleshooting

### Access Denied Unexpectedly

1. Check user's role
2. Check group memberships
3. Check document access level
4. Check for explicit deny
5. Review audit log

### Can't See Document

1. Verify document status (published?)
2. Check document access level
3. Verify group membership
4. Check folder permissions

### Can't Edit Document

1. Verify editor role or higher
2. Check document isn't locked
3. Verify not in archive
4. Check for edit:own scope
