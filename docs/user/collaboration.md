# Real-Time Collaboration

Guide to Tachyon's collaboration features for team documentation.

## Overview

In server mode, Tachyon provides real-time collaboration features:

- Multi-user editing
- Live cursors and presence
- Comments and discussions
- Version history
- Conflict resolution

## Enabling Collaboration

### Server Mode

Collaboration requires server mode:

```bash
tachyon serve --port 8080 --config tachyon.toml
```

### Configuration

```toml
[server]
enable_collaboration = true
presence_timeout_seconds = 30
conflict_resolution = "operational-transform"

[auth]
provider = "kanidm"  # or "oauth", "ldap"
enable_sso = true
```

## Real-Time Editing

### Live Cursors

When multiple users edit a document:

1. Each user's cursor appears with their name
2. Selections are highlighted with user colors
3. Typing updates appear instantly

### User Presence

The sidebar shows:
- Who is currently viewing
- Who is editing
- Last active time

### Synchronization

Changes sync via WebSocket:
- < 10ms update latency
- Automatic conflict resolution
- Offline queue support

## Comments

### Adding Comments

1. Select text
2. Click **Comment** in toolbar or press `Ctrl+Alt+C`
3. Type your comment
4. Submit

### Comment Threads

Comments support threading:
- Reply to existing comments
- @mention other users
- Resolve threads

### Managing Comments

| Action | How |
|--------|-----|
| Reply | Click reply field |
| Resolve | Click checkmark |
| Delete | Click trash icon |
| Edit | Click edit icon |

### Notifications

Users are notified when:
- Mentioned with `@username`
- Comment added to their document
- Thread resolved

## Document Sharing

### Sharing Links

Share documents with specific permissions:

1. Open document
2. Click **Share** button
3. Set permissions:
   - View only
   - Comment
   - Edit
4. Copy link

### Access Control

```yaml
---
access: restricted
groups: [team-alpha, stakeholders]
---
```

### Public Documents

Make documents publicly accessible:

```yaml
---
visibility: public
---
```

## Collaborative Workflows

### Review Process

1. Author creates draft
2. Assigns reviewers
3. Reviewers add comments
4. Author addresses feedback
5. Reviewer approves
6. Document published

### Approval Workflow

Configure in `tachyon.toml`:

```toml
[workflow]
require_approval = true
approvers = ["tech-lead", "docs-lead"]
min_approvers = 1
```

### Status Transitions

| From | To | Requires |
|------|-----|----------|
| draft | in-review | Submit for review |
| in-review | published | Approval |
| published | archived | Admin action |

## Conflict Resolution

### Operational Transform

Tachyon uses operational transform (OT) for conflict-free editing:

- Changes merge automatically
- Intent is preserved
- No data loss

### Manual Resolution

When conflicts can't auto-resolve:

1. Warning appears
2. Compare conflicting changes
3. Choose resolution:
   - Keep mine
   - Keep theirs
   - Merge manually

### Best Practices

- Communicate with collaborators
- Work on different sections
- Save frequently
- Review before major changes

## Team Features

### Activity Feed

View recent team activity:
- Document updates
- Comments
- Status changes
- New documents

### @Mentions

Mention team members:
```markdown
@jane Can you review this section?
```

Mentioned users receive notifications.

### Assignments

Assign documents to team members:

```yaml
---
assignee: jane-doe
reviewer: john-smith
---
```

### Watch Documents

Get notifications for specific documents:

1. Open document
2. Click **Watch** button
3. Choose notification level:
   - All activity
   - Comments only
   - Major changes only

## Permissions

### Role-Based Access

| Role | Permissions |
|------|-------------|
| **Viewer** | Read public documents |
| **Commenter** | View + comment |
| **Editor** | View + edit + create |
| **Reviewer** | Editor + approve |
| **Admin** | Full access |

### Document Permissions

Set per-document access:

```yaml
---
permissions:
  read: [team-alpha, stakeholders]
  write: [team-alpha]
  admin: [tech-lead]
---
```

### Inheritance

Permissions cascade:
- Folder permissions apply to contents
- Document permissions override folder
- Explicit deny takes precedence

## Notifications

### Notification Types

| Type | When |
|------|------|
| @mention | You're mentioned |
| Comment | Comment on your document |
| Review | Review requested |
| Status | Document status changed |
| Share | Document shared with you |

### Notification Settings

Configure in user preferences:

```toml
[notifications]
email = true
browser = true
slack_webhook = "https://hooks.slack.com/..."
```

### Digest Mode

Receive daily or weekly summaries instead of instant notifications:

```toml
[notifications]
digest = "daily"  # or "weekly", "instant"
```

## Mobile Access

### Web Interface

Access from any device:
- Responsive design
- Touch-optimized
- Offline support (limited)

### Mobile Editing

- Full editing capabilities
- Simplified UI
- Sync when online

## API Access

### REST API

Collaborate programmatically:

```bash
# Add comment
curl -X POST https://tachyon.example.com/api/v1/documents/{id}/comments \
  -H "Authorization: Bearer {token}" \
  -d '{"content": "Great work!"}'

# Get comments
curl https://tachyon.example.com/api/v1/documents/{id}/comments
```

### WebSocket API

Real-time updates:

```javascript
const ws = new WebSocket('wss://tachyon.example.com/ws');

ws.onmessage = (event) => {
  const update = JSON.parse(event.data);
  if (update.type === 'cursor') {
    // Show collaborator cursor
  }
};
```

## Best Practices

### Communication

- Use comments for discussion
- Resolve threads when done
- @mention relevant team members

### Organization

- Use consistent folder structure
- Tag documents appropriately
- Assign clear ownership

### Workflow

- Draft → Review → Publish
- Use status field consistently
- Get approvals before publishing

### Conflict Avoidance

- Coordinate major edits
- Work on different sections
- Check presence before editing
