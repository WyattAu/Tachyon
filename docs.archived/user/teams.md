# Team Management

Guide to managing teams and users in Tachyon.

## Overview

Tachyon server mode provides comprehensive team management:

- User management
- Team organization
- Role-based access control
- Group permissions

## User Management

### Adding Users

#### Via Admin UI

1. Navigate to **Admin > Users**
2. Click **Add User**
3. Enter details:
   - Username
   - Email
   - Initial role
4. Send invitation

#### Via API

```bash
curl -X POST https://tachyon.example.com/api/v1/users \
  -H "Authorization: Bearer {admin-token}" \
  -d '{
    "username": "jane-doe",
    "email": "jane@example.com",
    "role": "editor"
  }'
```

### User Properties

| Property | Description |
|----------|-------------|
| `username` | Unique identifier |
| `email` | Email address |
| `display_name` | Full name |
| `role` | System role |
| `groups` | Group memberships |
| `status` | active | inactive | suspended |

### Editing Users

1. Navigate to **Admin > Users**
2. Click user to edit
3. Modify properties
4. Save changes

### Deactivating Users

1. Navigate to **Admin > Users**
2. Select user
3. Click **Deactivate**
4. Confirm action

Deactivated users:
- Cannot log in
- Retain document history
- Can be reactivated

### Deleting Users

1. Deactivate user first
2. Navigate to user details
3. Click **Delete**
4. Confirm with username

Deleted users:
- Removed from all groups
- Documents reassigned or deleted
- Action is irreversible

## Teams and Groups

### Creating Groups

1. Navigate to **Admin > Groups**
2. Click **Create Group**
3. Enter:
   - Group name
   - Description
   - Initial members

### Group Properties

| Property | Description |
|----------|-------------|
| `name` | Unique group name |
| `display_name` | Human-readable name |
| `description` | Group purpose |
| `members` | User list |
| `permissions` | Access levels |

### Managing Members

Add members to a group:
1. Open group details
2. Click **Add Members**
3. Search and select users
4. Confirm

Remove members:
1. Open group details
2. Find member in list
3. Click **Remove**
4. Confirm

### Nested Groups

Create hierarchical group structures:

```yaml
groups:
  - name: engineering
    members: []
    children:
      - name: backend
        members: [alice, bob]
      - name: frontend
        members: [carol, dave]
```

Permissions inherit from parent groups.

## Roles and Permissions

### System Roles

| Role | Description | Permissions |
|------|-------------|-------------|
| **admin** | Full system access | Everything |
| **reviewer** | Content reviewer | Edit + approve |
| **editor** | Content creator | Create + edit |
| **commenter** | Can comment | View + comment |
| **viewer** | Read-only | View public docs |

### Custom Roles

Define custom roles in configuration:

```toml
[[roles]]
name = "tech-writer"
permissions = [
  "documents:read",
  "documents:create",
  "documents:edit",
  "documents:delete:own",
  "comments:create",
  "comments:resolve"
]
```

### Permission Types

| Category | Permissions |
|----------|-------------|
| Documents | read, create, edit, delete, publish |
| Comments | create, edit, delete, resolve |
| Users | read, create, edit, delete |
| Groups | read, create, edit, delete |
| Admin | settings, audit, backup |

### Permission Scopes

```
documents:edit          # All documents
documents:edit:own      # Own documents only
documents:edit:group    # Group documents
```

## Access Control

### Document-Level Access

Control access per document:

```yaml
---
title: Internal API Keys
access: restricted
groups: [devops, security]
roles: [admin]
---
```

### Folder-Level Access

Set permissions on folders:

1. Right-click folder
2. Select **Permissions**
3. Configure:
   - Read access
   - Write access
   - Admin access

### Block-Level Redaction

Hide sensitive content:

```markdown
::: internal
API Key: sk-1234567890
:::
```

Users without `internal` group see nothing.

### Access Modes

| Mode | Behavior |
|------|----------|
| `public` | Visible to all authenticated users |
| `private` | Owner and admins only |
| `restricted` | Specific groups/roles only |

## Authentication

### Auth Providers

Configure in `tachyon.toml`:

```toml
[auth]
provider = "kanidm"  # kanidm | oauth | ldap | local
```

### Kanidm (Recommended)

```toml
[auth.kanidm]
url = "https://kanidm.example.com"
client_id = "tachyon"
client_secret = "${KANIDM_SECRET}"
```

### OAuth (Generic)

```toml
[auth.oauth]
provider = "github"  # github | gitlab | google | custom
client_id = "your-client-id"
client_secret = "${OAUTH_SECRET}"
redirect_uri = "https://tachyon.example.com/auth/callback"
```

### LDAP

```toml
[auth.ldap]
url = "ldap://ldap.example.com"
base_dn = "ou=users,dc=example,dc=com"
bind_dn = "cn=admin,dc=example,dc=com"
bind_password = "${LDAP_PASSWORD}"
```

### Local (Development)

```toml
[auth.local]
enabled = true
allow_registration = false
```

### Single Sign-On (SSO)

Enable SSO for seamless authentication:

```toml
[auth]
enable_sso = true
sso_domain = ".example.com"
```

## Team Workflows

### Onboarding

1. Create user account
2. Add to appropriate groups
3. Assign role
4. Send welcome email with docs

### Offboarding

1. Deactivate account
2. Reassign documents
3. Remove from groups
4. Archive or transfer work

### Audit Trail

All team actions are logged:
- User logins
- Permission changes
- Document access
- Admin actions

View in **Admin > Audit Log**

## API Management

### API Tokens

Create tokens for automation:

1. Navigate to **Settings > API Tokens**
2. Click **Create Token**
3. Set name and expiration
4. Select scopes
5. Copy token (shown once)

### Token Scopes

| Scope | Access |
|-------|--------|
| `documents:read` | Read documents |
| `documents:write` | Create/update documents |
| `search` | Search functionality |
| `admin` | Admin operations |

### Managing Tokens

```bash
# List tokens
curl https://tachyon.example.com/api/v1/tokens \
  -H "Authorization: Bearer {token}"

# Revoke token
curl -X DELETE https://tachyon.example.com/api/v1/tokens/{id} \
  -H "Authorization: Bearer {token}"
```

## Best Practices

### Principle of Least Privilege

- Grant minimum required permissions
- Use groups over individual assignments
- Review permissions regularly

### Group Strategy

- Create functional groups (engineering, docs, legal)
- Create project groups (project-alpha)
- Combine for access control

### Regular Reviews

- Audit group memberships monthly
- Review inactive users quarterly
- Validate permission assignments

### Security

- Use SSO when available
- Enforce strong passwords
- Enable MFA for admin roles
- Rotate API tokens regularly
