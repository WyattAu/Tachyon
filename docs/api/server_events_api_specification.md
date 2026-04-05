# TACHYON: SERVER EVENTS API SPECIFICATION

**Document ID:** TACHYON-API-010-V1.0
**Date:** February 2026
**Status:** Proposed
**Classification:** API Specification
**Dependencies:** [TACHYON-STD-V1.0](../../.specs/01_standards/coding_standards.md), [TACHYON-REQ-SRV-V1.0](../../.specs/04_future_state/reqs/server_requirements.md), [TACHYON-ADR-003-V1.0](../../.specs/02_adrs/003_axum_for_http2_server.md), [TACHYON-ADR-007-V1.0](../../.specs/02_adrs/007_tokio_for_async_runtime.md), [TACHYON-TMA-V1.0](../../.specs/03_threat_model/analysis.md)

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [WebSocket Event Design Principles](#2-websocket-event-design-principles)
3. [Connection Events](#3-connection-events)
4. [Document Events](#4-document-events)
5. [User Events](#5-user-events)
6. [Repository Events](#6-repository-events)
7. [Event Security](#7-event-security)
8. [Event Performance](#8-event-performance)
9. [References](#9-references)

---

## 1. INTRODUCTION

### 1.1. Document Purpose

This document defines the comprehensive specification for the Tachyon Server Events API, which provides real-time event delivery through WebSocket connections. The Events API enables clients to receive instantaneous updates for documents, user presence, repository state changes, and collaborative editing events without requiring polling or periodic refresh operations.

### 1.2. Scope

This specification covers:
- WebSocket connection lifecycle and management
- Event message formats and schemas
- Subscription mechanisms for event filtering
- Event security and authorization
- Performance requirements and optimization strategies
- Error handling and recovery procedures

Out of scope:
- REST API endpoints (covered in separate API specifications)
- Desktop-specific IPC events (covered in IPC protocol specification)
- Internal server event bus architecture (covered in server design documentation)

### 1.3. Design Philosophy

The Tachyon Server Events API adheres to the following design principles:

**Event-Driven Architecture:**
Events represent discrete state changes within the system, enabling clients to maintain synchronized state without explicit polling. Each event carries sufficient information for clients to update their local state representation.

**Backwards Compatibility:**
Event schemas are designed with versioning support to allow for evolution without breaking existing clients. New fields may be added to events without requiring client updates.

**Idempotency:**
Events are designed to be idempotent, meaning that receiving the same event multiple times does not result in inconsistent state. Clients may safely re-process events without side effects.

**Deterministic Ordering:**
Events are delivered in a deterministic order based on their occurrence timestamp, ensuring that all clients observe the same sequence of state changes.

### 1.4. WebSocket Protocol Overview

The Events API uses the WebSocket protocol (RFC 6455) over TLS 1.3 for secure, bidirectional communication. WebSocket connections are established at the `/ws` endpoint and require authentication via session tokens or OAuth 2.0 bearer tokens.

**Connection URL Format:**
```
wss://tachyon.example.com/ws?token=<session_token>
```

**Protocol Version:**
The Events API uses protocol version 1.0, specified in the initial handshake via the `X-Tachyon-Protocol-Version` header.

---

## 2. WEBSOCKET EVENT DESIGN PRINCIPLES

### 2.1. Event Message Structure

All events transmitted over the WebSocket connection follow a standardized JSON message structure:

```json
{
  "event_type": "string",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": { }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `event_type` | string | Yes | The type identifier for the event (e.g., "document.updated", "user.presence") |
| `event_id` | string (UUID) | Yes | Unique identifier for the event, used for deduplication and ordering |
| `timestamp` | string (ISO8601) | Yes | UTC timestamp of when the event occurred |
| `data` | object | Yes | Event-specific data payload |

### 2.2. Event Type Taxonomy

Events are organized into the following categories:

**Connection Events:**
Events related to WebSocket connection lifecycle, including connection establishment, authentication, heartbeat, and disconnection.

**Document Events:**
Events representing changes to document content, including creation, updates, deletion, and metadata changes.

**User Events:**
Events related to user presence and activity, including online status, typing indicators, and cursor positions.

**Repository Events:**
Events representing Git repository state changes, including branch switches, commits, and sync operations.

**System Events:**
Events related to system-wide notifications, maintenance windows, and configuration changes.

### 2.3. Event Subscription Model

The Events API implements a subscription-based filtering mechanism to reduce bandwidth and processing overhead on clients. Clients may subscribe to specific event types and scopes during connection establishment.

**Subscription Request Format:**
```json
{
  "action": "subscribe",
  "subscriptions": [
    {
      "event_type": "document.updated",
      "filters": {
        "document_id": ["doc-123", "doc-456"]
      }
    },
    {
      "event_type": "user.presence",
      "filters": {
        "document_id": "doc-123"
      }
    }
  ]
}
```

**Subscription Response Format:**
```json
{
  "action": "subscribe_response",
  "status": "success",
  "subscription_id": "sub-uuid",
  "acknowledged": [
    {
      "event_type": "document.updated",
      "filters": { }
    }
  ]
}
```

### 2.4. Event Ordering and Delivery Guarantees

The Events API provides the following delivery guarantees:

**At-Least-Once Delivery:**
Events are delivered at least once to all subscribed clients. Clients must handle potential duplicate events using the `event_id` field for deduplication.

**In-Order Delivery:**
Events are delivered to each client in the order they occurred, as determined by their timestamp. This ensures consistent state across all clients.

**Bounded Latency:**
Events are delivered within 50 milliseconds of occurrence under normal load conditions, ensuring real-time responsiveness.

**Event Persistence:**
Events are persisted for 24 hours to support client reconnection and event replay. Clients may request missed events upon reconnection.

### 2.5. Error Handling

Error conditions are communicated through standardized error event messages:

**Error Event Format:**
```json
{
  "event_type": "error",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "error_code": "string",
    "error_message": "string",
    "error_details": { }
  }
}
```

**Error Codes:**

| Error Code | Description | HTTP Status Equivalent |
|------------|-------------|----------------------|
| `AUTH_FAILED` | Authentication or authorization failure | 401 |
| `SUBSCRIPTION_INVALID` | Invalid subscription request | 400 |
| `RATE_LIMITED` | Client has exceeded rate limits | 429 |
| `SERVER_ERROR` | Internal server error | 500 |
| `CONNECTION_LOST` | WebSocket connection lost | N/A |
| `EVENT_MALFORMED` | Malformed event received from client | 400 |

---

## 3. CONNECTION EVENTS

### 3.1. Connection Lifecycle Events

Connection events manage the WebSocket connection lifecycle, including establishment, authentication, heartbeat, and graceful disconnection.

#### 3.1.1. Connection Established Event

**Event Type:** `connection.established`

**Description:** Emitted when a WebSocket connection is successfully established and authenticated. This event confirms that the client is connected and ready to receive events.

**Event Payload:**
```json
{
  "event_type": "connection.established",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "connection_id": "string",
    "user_id": "string",
    "session_id": "string",
    "server_version": "string",
    "protocol_version": "string",
    "capabilities": {
      "supports_event_replay": boolean,
      "max_event_batch_size": integer,
      "heartbeat_interval_seconds": integer
    }
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `connection_id` | string | Yes | Unique identifier for the WebSocket connection |
| `user_id` | string | Yes | Identifier of the authenticated user |
| `session_id` | string | Yes | Identifier of the user session |
| `server_version` | string | Yes | Version of the Tachyon server |
| `protocol_version` | string | Yes | Version of the Events API protocol |
| `capabilities.supports_event_replay` | boolean | Yes | Whether the server supports event replay on reconnection |
| `capabilities.max_event_batch_size` | integer | Yes | Maximum number of events in a single batch |
| `capabilities.heartbeat_interval_seconds` | integer | Yes | Interval in seconds between heartbeat messages |

**Related Requirements:**
- REQ-SRV-091: WebSocket Endpoint
- REQ-SRV-092: Connection Authentication

#### 3.1.2. Heartbeat Event

**Event Type:** `connection.heartbeat`

**Description:** Periodically emitted by the server to maintain the WebSocket connection and detect dead connections. Clients must respond with a heartbeat acknowledgment.

**Event Payload:**
```json
{
  "event_type": "connection.heartbeat",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "sequence": integer,
    "server_time": "ISO8601"
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `sequence` | integer | Yes | Monotonically increasing sequence number for heartbeat messages |
| `server_time` | string (ISO8601) | Yes | Current server time for clock synchronization |

**Client Response:**
Clients must respond with a heartbeat acknowledgment within 10 seconds:

```json
{
  "action": "heartbeat_ack",
  "sequence": integer,
  "client_time": "ISO8601"
}
```

**Related Requirements:**
- REQ-SRV-094: Heartbeat Mechanism

#### 3.1.3. Connection Closing Event

**Event Type:** `connection.closing`

**Description:** Emitted when the server is initiating a graceful connection closure. This event provides advance notice and reason for disconnection.

**Event Payload:**
```json
{
  "event_type": "connection.closing",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "reason": "string",
    "code": integer,
    "reconnect_delay_seconds": integer,
    "reconnect_url": "string"
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `reason` | string | Yes | Human-readable description of the closure reason |
| `code` | integer | Yes | WebSocket close code (RFC 6455) |
| `reconnect_delay_seconds` | integer | No | Suggested delay before attempting reconnection |
| `reconnect_url` | string | No | Alternative URL for reconnection (for load balancing) |

**Close Codes:**

| Code | Description | Reconnectable |
|------|-------------|---------------|
| 1000 | Normal closure | Yes |
| 1001 | Endpoint going away | Yes |
| 1008 | Policy violation | No |
| 1011 | Internal server error | Yes |
| 4000 | Authentication expired | Yes |
| 4001 | Session terminated | Yes |
| 4002 | Server maintenance | Yes |

**Related Requirements:**
- REQ-SRV-095: Graceful Disconnection

#### 3.1.4. Connection Lost Event

**Event Type:** `connection.lost`

**Description:** Emitted when the WebSocket connection is unexpectedly lost. This event is sent to other connected clients to notify them of a user's disconnection.

**Event Payload:**
```json
{
  "event_type": "connection.lost",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "user_id": "string",
    "session_id": "string",
    "last_seen": "ISO8601",
    "reason": "string"
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `user_id` | string | Yes | Identifier of the disconnected user |
| `session_id` | string | Yes | Identifier of the lost session |
| `last_seen` | string (ISO8601) | Yes | Timestamp of the last activity from the user |
| `reason` | string | No | Reason for connection loss if known |

**Related Requirements:**
- REQ-SRV-095: Graceful Disconnection

### 3.2. Subscription Events

Subscription events manage client subscriptions to specific event types and filters.

#### 3.2.1. Subscription Confirmed Event

**Event Type:** `subscription.confirmed`

**Description:** Emitted when a client's subscription request is successfully processed and confirmed.

**Event Payload:**
```json
{
  "event_type": "subscription.confirmed",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "subscription_id": "string",
    "subscriptions": [
      {
        "event_type": "string",
        "filters": { }
      }
    ],
    "effective_from": "ISO8601"
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `subscription_id` | string | Yes | Unique identifier for the subscription |
| `subscriptions` | array | Yes | List of confirmed subscriptions |
| `effective_from` | string (ISO8601) | Yes | Timestamp from which the subscription is active |

#### 3.2.2. Subscription Failed Event

**Event Type:** `subscription.failed`

**Description:** Emitted when a subscription request fails due to invalid parameters or authorization issues.

**Event Payload:**
```json
{
  "event_type": "subscription.failed",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "error_code": "string",
    "error_message": "string",
    "failed_subscription": {
      "event_type": "string",
      "filters": { }
    }
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `error_code` | string | Yes | Machine-readable error code |
| `error_message` | string | Yes | Human-readable error description |
| `failed_subscription` | object | Yes | The subscription that failed |

**Error Codes:**

| Error Code | Description |
|------------|-------------|
| `INVALID_EVENT_TYPE` | The specified event type does not exist |
| `UNAUTHORIZED_SCOPE` | Client is not authorized to subscribe to the specified scope |
| `INVALID_FILTER` | The filter specification is invalid |
| `SUBSCRIPTION_LIMIT_EXCEEDED` | Client has exceeded the maximum number of subscriptions |

#### 3.2.3. Subscription Cancelled Event

**Event Type:** `subscription.cancelled`

**Description:** Emitted when a subscription is cancelled, either by the client or by the server.

**Event Payload:**
```json
{
  "event_type": "subscription.cancelled",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "subscription_id": "string",
    "reason": "string",
    "cancelled_by": "client|server"
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `subscription_id` | string | Yes | Identifier of the cancelled subscription |
| `reason` | string | Yes | Reason for cancellation |
| `cancelled_by` | string | Yes | Entity that initiated the cancellation |

---

## 4. DOCUMENT EVENTS

### 4.1. Document Lifecycle Events

Document lifecycle events represent changes to document state, including creation, updates, deletion, and metadata changes.

#### 4.1.1. Document Created Event

**Event Type:** `document.created`

**Description:** Emitted when a new document is created in the system. This event includes the initial document content and metadata.

**Event Payload:**
```json
{
  "event_type": "document.created",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "document_id": "string",
    "repository_id": "string",
    "branch": "string",
    "path": "string",
    "title": "string",
    "created_by": "string",
    "created_at": "ISO8601",
    "frontmatter": { },
    "content_preview": "string",
    "content_length": integer
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `document_id` | string | Yes | Unique identifier for the document |
| `repository_id` | string | Yes | Identifier of the repository containing the document |
| `branch` | string | Yes | Git branch where the document was created |
| `path` | string | Yes | Relative path to the document file |
| `title` | string | Yes | Document title extracted from frontmatter or filename |
| `created_by` | string | Yes | User ID of the document creator |
| `created_at` | string (ISO8601) | Yes | Timestamp when the document was created |
| `frontmatter` | object | Yes | YAML frontmatter metadata |
| `content_preview` | string | Yes | Preview of the document content (first 200 characters) |
| `content_length` | integer | Yes | Length of the document content in bytes |

**Related Requirements:**
- REQ-SRV-023: Document Creation
- REQ-SRV-047: Commit Management

#### 4.1.2. Document Updated Event

**Event Type:** `document.updated`

**Description:** Emitted when a document's content or metadata is modified. This event includes the changes made to the document.

**Event Payload:**
```json
{
  "event_type": "document.updated",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "document_id": "string",
    "repository_id": "string",
    "branch": "string",
    "path": "string",
    "updated_by": "string",
    "updated_at": "ISO8601",
    "changes": {
      "content_changed": boolean,
      "frontmatter_changed": boolean,
      "content_diff": "string",
      "frontmatter_diff": { }
    },
    "commit_id": "string",
    "content_preview": "string"
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `document_id` | string | Yes | Unique identifier for the document |
| `repository_id` | string | Yes | Identifier of the repository containing the document |
| `branch` | string | Yes | Git branch where the document was updated |
| `path` | string | Yes | Relative path to the document file |
| `updated_by` | string | Yes | User ID of the user who updated the document |
| `updated_at` | string (ISO8601) | Yes | Timestamp when the document was updated |
| `changes.content_changed` | boolean | Yes | Whether the document content changed |
| `changes.frontmatter_changed` | boolean | Yes | Whether the frontmatter changed |
| `changes.content_diff` | string | No | Unified diff of content changes |
| `changes.frontmatter_diff` | object | No | Diff of frontmatter changes |
| `commit_id` | string | Yes | Git commit ID for the update |
| `content_preview` | string | Yes | Preview of the updated document content |

**Related Requirements:**
- REQ-SRV-024: Document Update
- REQ-SRV-047: Commit Management
- REQ-SRV-096: Content Updates

#### 4.1.3. Document Deleted Event

**Event Type:** `document.deleted`

**Description:** Emitted when a document is deleted from the system.

**Event Payload:**
```json
{
  "event_type": "document.deleted",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "document_id": "string",
    "repository_id": "string",
    "branch": "string",
    "path": "string",
    "deleted_by": "string",
    "deleted_at": "ISO8601",
    "commit_id": "string",
    "reason": "string"
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `document_id` | string | Yes | Unique identifier for the deleted document |
| `repository_id` | string | Yes | Identifier of the repository containing the document |
| `branch` | string | Yes | Git branch where the document was deleted |
| `path` | string | Yes | Relative path to the deleted document file |
| `deleted_by` | string | Yes | User ID of the user who deleted the document |
| `deleted_at` | string (ISO8601) | Yes | Timestamp when the document was deleted |
| `commit_id` | string | Yes | Git commit ID for the deletion |
| `reason` | string | No | Reason for deletion if specified |

**Related Requirements:**
- REQ-SRV-025: Document Deletion
- REQ-SRV-047: Commit Management

### 4.2. Document Collaboration Events

Document collaboration events support real-time collaborative editing features, including conflict notifications and edit acknowledgments.

#### 4.2.1. Document Conflict Event

**Event Type:** `document.conflict`

**Description:** Emitted when concurrent edits to a document are detected that may result in conflicts. This event provides information for conflict resolution.

**Event Payload:**
```json
{
  "event_type": "document.conflict",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "document_id": "string",
    "repository_id": "string",
    "branch": "string",
    "conflicting_edits": [
      {
        "edit_id": "string",
        "user_id": "string",
        "timestamp": "ISO8601",
        "edit_type": "insert|delete|replace",
        "position": {
          "line": integer,
          "column": integer
        },
        "content": "string"
      }
    ],
    "resolution_strategy": "last-write-wins|manual|merge",
    "requires_resolution": boolean
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `document_id` | string | Yes | Unique identifier for the document |
| `repository_id` | string | Yes | Identifier of the repository |
| `branch` | string | Yes | Git branch where the conflict occurred |
| `conflicting_edits` | array | Yes | List of conflicting edits |
| `conflicting_edits[].edit_id` | string | Yes | Unique identifier for the edit |
| `conflicting_edits[].user_id` | string | Yes | User ID of the edit author |
| `conflicting_edits[].timestamp` | string (ISO8601) | Yes | Timestamp of the edit |
| `conflicting_edits[].edit_type` | string | Yes | Type of edit (insert, delete, replace) |
| `conflicting_edits[].position` | object | Yes | Position of the edit in the document |
| `conflicting_edits[].content` | string | Yes | Content of the edit |
| `resolution_strategy` | string | Yes | Suggested resolution strategy |
| `requires_resolution` | boolean | Yes | Whether manual resolution is required |

**Related Requirements:**
- REQ-SRV-098: Conflict Notifications
- REQ-SRV-101: Last-Write-Wins
- REQ-SRV-103: Conflict Resolution UI

#### 4.2.2. Document Edit Acknowledgment Event

**Event Type:** `document.edit_acknowledged`

**Description:** Emitted to acknowledge that a client's edit has been successfully applied to the document.

**Event Payload:**
```json
{
  "event_type": "document.edit_acknowledged",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "document_id": "string",
    "edit_id": "string",
    "user_id": "string",
    "acknowledged_at": "ISO8601",
    "applied_position": {
      "line": integer,
      "column": integer
    },
    "status": "applied|merged|rejected"
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `document_id` | string | Yes | Unique identifier for the document |
| `edit_id` | string | Yes | Unique identifier for the acknowledged edit |
| `user_id` | string | Yes | User ID of the edit author |
| `acknowledged_at` | string (ISO8601) | Yes | Timestamp of acknowledgment |
| `applied_position` | object | Yes | Position where the edit was applied |
| `status` | string | Yes | Status of the edit (applied, merged, rejected) |

**Related Requirements:**
- REQ-SRV-102: Edit Queue
- REQ-SRV-104: Edit History

### 4.3. Document Subscription Mechanisms

Clients may subscribe to document-specific events to receive updates for particular documents.

#### 4.3.1. Document Subscription Request

**Subscription Request Format:**
```json
{
  "action": "subscribe",
  "subscriptions": [
    {
      "event_type": "document.updated",
      "filters": {
        "document_id": ["doc-123", "doc-456"]
      }
    },
    {
      "event_type": "document.conflict",
      "filters": {
        "document_id": "doc-123"
      }
    }
  ]
}
```

#### 4.3.2. Document Subscription Filters

**Supported Filter Fields:**

| Filter Field | Type | Description |
|-------------|------|-------------|
| `document_id` | string or array | Document ID(s) to filter |
| `repository_id` | string | Repository ID to filter |
| `branch` | string | Git branch to filter |
| `user_id` | string | User ID to filter (for edits by specific users) |

**Filter Examples:**

**Subscribe to specific documents:**
```json
{
  "event_type": "document.updated",
  "filters": {
    "document_id": ["doc-123", "doc-456"]
  }
}
```

**Subscribe to all documents in a repository:**
```json
{
  "event_type": "document.created",
  "filters": {
    "repository_id": "repo-789"
  }
}
```

**Subscribe to edits by a specific user:**
```json
{
  "event_type": "document.updated",
  "filters": {
    "user_id": "user-abc"
  }
}
```

---

## 5. USER EVENTS

### 5.1. User Presence Events

User presence events communicate the online status and activity state of users within the system.

#### 5.1.1. User Presence Event

**Event Type:** `user.presence`

**Description:** Emitted when a user's presence state changes, including online/offline status and current activity.

**Event Payload:**
```json
{
  "event_type": "user.presence",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "user_id": "string",
    "username": "string",
    "display_name": "string",
    "status": "online|away|busy|offline",
    "last_seen": "ISO8601",
    "current_document": {
      "document_id": "string",
      "document_title": "string"
    },
    "session_id": "string"
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `user_id` | string | Yes | Unique identifier for the user |
| `username` | string | Yes | Username of the user |
| `display_name` | string | Yes | Display name of the user |
| `status` | string | Yes | Presence status (online, away, busy, offline) |
| `last_seen` | string (ISO8601) | Yes | Timestamp of the last activity |
| `current_document.document_id` | string | No | ID of the document the user is viewing |
| `current_document.document_title` | string | No | Title of the document the user is viewing |
| `session_id` | string | Yes | Identifier of the user's current session |

**Status Values:**

| Status | Description |
|--------|-------------|
| `online` | User is actively using the system |
| `away` | User is connected but inactive for > 5 minutes |
| `busy` | User has manually set busy status |
| `offline` | User has disconnected from the system |

**Related Requirements:**
- REQ-SRV-097: User Presence

### 5.2. Typing Indicator Events

Typing indicator events communicate when users are actively editing documents, enabling collaborative editing awareness.

#### 5.2.1. Typing Started Event

**Event Type:** `typing.started`

**Description:** Emitted when a user begins typing in a document.

**Event Payload:**
```json
{
  "event_type": "typing.started",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "user_id": "string",
    "username": "string",
    "display_name": "string",
    "document_id": "string",
    "document_title": "string",
    "position": {
      "line": integer,
      "column": integer
    },
    "started_at": "ISO8601"
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `user_id` | string | Yes | Unique identifier for the user |
| `username` | string | Yes | Username of the user |
| `display_name` | string | Yes | Display name of the user |
| `document_id` | string | Yes | ID of the document being edited |
| `document_title` | string | Yes | Title of the document being edited |
| `position.line` | integer | Yes | Line number where typing started |
| `position.column` | integer | Yes | Column number where typing started |
| `started_at` | string (ISO8601) | Yes | Timestamp when typing started |

#### 5.2.2. Typing Stopped Event

**Event Type:** `typing.stopped`

**Description:** Emitted when a user stops typing in a document.

**Event Payload:**
```json
{
  "event_type": "typing.stopped",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "user_id": "string",
    "document_id": "string",
    "stopped_at": "ISO8601",
    "duration_seconds": integer
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `user_id` | string | Yes | Unique identifier for the user |
| `document_id` | string | Yes | ID of the document where typing stopped |
| `stopped_at` | string (ISO8601) | Yes | Timestamp when typing stopped |
| `duration_seconds` | integer | Yes | Duration of typing in seconds |

**Related Requirements:**
- REQ-SRV-099: Typing Indicators

### 5.3. Cursor Position Events

Cursor position events communicate the current cursor location of users within documents, enabling real-time collaborative editing visualization.

#### 5.3.1. Cursor Moved Event

**Event Type:** `cursor.moved`

**Description:** Emitted when a user's cursor position changes within a document.

**Event Payload:**
```json
{
  "event_type": "cursor.moved",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "user_id": "string",
    "username": "string",
    "display_name": "string",
    "document_id": "string",
    "document_title": "string",
    "position": {
      "line": integer,
      "column": integer
    },
    "selection": {
      "start": {
        "line": integer,
        "column": integer
      },
      "end": {
        "line": integer,
        "column": integer
      }
    },
    "moved_at": "ISO8601"
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `user_id` | string | Yes | Unique identifier for the user |
| `username` | string | Yes | Username of the user |
| `display_name` | string | Yes | Display name of the user |
| `document_id` | string | Yes | ID of the document |
| `document_title` | string | Yes | Title of the document |
| `position.line` | integer | Yes | Line number of cursor position |
| `position.column` | integer | Yes | Column number of cursor position |
| `selection.start.line` | integer | No | Line number of selection start |
| `selection.start.column` | integer | No | Column number of selection start |
| `selection.end.line` | integer | No | Line number of selection end |
| `selection.end.column` | integer | No | Column number of selection end |
| `moved_at` | string (ISO8601) | Yes | Timestamp when cursor moved |

**Related Requirements:**
- REQ-SRV-100: Cursor Position

### 5.4. User Event Subscription Mechanisms

Clients may subscribe to user-specific events to receive updates about user presence, typing indicators, and cursor positions.

#### 5.4.1. User Event Subscription Request

**Subscription Request Format:**
```json
{
  "action": "subscribe",
  "subscriptions": [
    {
      "event_type": "user.presence",
      "filters": {
        "document_id": "doc-123"
      }
    },
    {
      "event_type": "typing.started",
      "filters": {
        "document_id": "doc-123"
      }
    },
    {
      "event_type": "cursor.moved",
      "filters": {
        "document_id": "doc-123"
      }
    }
  ]
}
```

#### 5.4.2. User Event Subscription Filters

**Supported Filter Fields:**

| Filter Field | Type | Description |
|-------------|------|-------------|
| `document_id` | string | Document ID to filter events for |
| `user_id` | string | User ID to filter events from |

**Filter Examples:**

**Subscribe to presence for a specific document:**
```json
{
  "event_type": "user.presence",
  "filters": {
    "document_id": "doc-123"
  }
}
```

**Subscribe to typing indicators from a specific user:**
```json
{
  "event_type": "typing.started",
  "filters": {
    "user_id": "user-abc"
  }
}
```

---

## 6. REPOSITORY EVENTS

### 6.1. Repository Sync Events

Repository sync events communicate Git repository synchronization operations with remote repositories.

#### 6.1.1. Repository Sync Started Event

**Event Type:** `repository.sync_started`

**Description:** Emitted when a repository synchronization operation with a remote repository begins.

**Event Payload:**
```json
{
  "event_type": "repository.sync_started",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "repository_id": "string",
    "repository_name": "string",
    "remote_url": "string",
    "sync_type": "fetch|push|pull",
    "initiated_by": "string",
    "started_at": "ISO8601"
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `repository_id` | string | Yes | Unique identifier for repository |
| `repository_name` | string | Yes | Name of the repository |
| `remote_url` | string | Yes | URL of the remote repository |
| `sync_type` | string | Yes | Type of sync operation (fetch, push, pull) |
| `initiated_by` | string | Yes | User ID of the user who initiated the sync |
| `started_at` | string (ISO8601) | Yes | Timestamp when the sync operation started |

#### 6.1.2. Repository Sync Completed Event

**Event Type:** `repository.sync_completed`

**Description:** Emitted when a repository synchronization operation completes successfully.

**Event Payload:**
```json
{
  "event_type": "repository.sync_completed",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "repository_id": "string",
    "repository_name": "string",
    "sync_type": "fetch|push|pull",
    "initiated_by": "string",
    "completed_at": "ISO8601",
    "duration_seconds": integer,
    "changes": {
      "objects_fetched": integer,
      "objects_pushed": integer,
      "commits_received": integer,
      "commits_sent": integer
    }
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `repository_id` | string | Yes | Unique identifier for repository |
| `repository_name` | string | Yes | Name of the repository |
| `sync_type` | string | Yes | Type of sync operation (fetch, push, pull) |
| `initiated_by` | string | Yes | User ID of the user who initiated the sync |
| `completed_at` | string (ISO8601) | Yes | Timestamp when the sync operation completed |
| `duration_seconds` | integer | Yes | Duration of the sync operation in seconds |
| `changes.objects_fetched` | integer | No | Number of Git objects fetched |
| `changes.objects_pushed` | integer | No | Number of Git objects pushed |
| `changes.commits_received` | integer | No | Number of commits received |
| `changes.commits_sent` | integer | No | Number of commits sent |

#### 6.1.3. Repository Sync Failed Event

**Event Type:** `repository.sync_failed`

**Description:** Emitted when a repository synchronization operation fails.

**Event Payload:**
```json
{
  "event_type": "repository.sync_failed",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "repository_id": "string",
    "repository_name": "string",
    "sync_type": "fetch|push|pull",
    "initiated_by": "string",
    "failed_at": "ISO8601",
    "error_code": "string",
    "error_message": "string",
    "error_details": { }
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `repository_id` | string | Yes | Unique identifier for repository |
| `repository_name` | string | Yes | Name of the repository |
| `sync_type` | string | Yes | Type of sync operation (fetch, push, pull) |
| `initiated_by` | string | Yes | User ID of the user who initiated the sync |
| `failed_at` | string (ISO8601) | Yes | Timestamp when the sync operation failed |
| `error_code` | string | Yes | Machine-readable error code |
| `error_message` | string | Yes | Human-readable error message |
| `error_details` | object | No | Additional error details |

**Related Requirements:**
- REQ-SRV-050: Repository Sync

### 6.2. Branch Change Events

Branch change events communicate Git branch operations including creation, switching, and deletion.

#### 6.2.1. Branch Created Event

**Event Type:** `branch.created`

**Description:** Emitted when a new Git branch is created in a repository.

**Event Payload:**
```json
{
  "event_type": "branch.created",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "repository_id": "string",
    "repository_name": "string",
    "branch_name": "string",
    "created_from": "string",
    "created_by": "string",
    "created_at": "ISO8601"
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `repository_id` | string | Yes | Unique identifier for repository |
| `repository_name` | string | Yes | Name of the repository |
| `branch_name` | string | Yes | Name of the newly created branch |
| `created_from` | string | Yes | Name of the branch or commit that the new branch was created from |
| `created_by` | string | Yes | User ID of the user who created the branch |
| `created_at` | string (ISO8601) | Yes | Timestamp when the branch was created |

#### 6.2.2. Branch Switched Event

**Event Type:** `branch.switched`

**Description:** Emitted when the active Git branch is changed.

**Event Payload:**
```json
{
  "event_type": "branch.switched",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "repository_id": "string",
    "repository_name": "string",
    "previous_branch": "string",
    "new_branch": "string",
    "switched_by": "string",
    "switched_at": "ISO8601"
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `repository_id` | string | Yes | Unique identifier for repository |
| `repository_name` | string | Yes | Name of the repository |
| `previous_branch` | string | Yes | Name of the previous active branch |
| `new_branch` | string | Yes | Name of the new active branch |
| `switched_by` | string | Yes | User ID of the user who switched the branch |
| `switched_at` | string (ISO8601) | Yes | Timestamp when the branch was switched |

#### 6.2.3. Branch Deleted Event

**Event Type:** `branch.deleted`

**Description:** Emitted when a Git branch is deleted from a repository.

**Event Payload:**
```json
{
  "event_type": "branch.deleted",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "repository_id": "string",
    "repository_name": "string",
    "branch_name": "string",
    "deleted_by": "string",
    "deleted_at": "ISO8601"
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `repository_id` | string | Yes | Unique identifier for repository |
| `repository_name` | string | Yes | Name of the repository |
| `branch_name` | string | Yes | Name of the deleted branch |
| `deleted_by` | string | Yes | User ID of the user who deleted the branch |
| `deleted_at` | string (ISO8601) | Yes | Timestamp when the branch was deleted |

**Related Requirements:**
- REQ-SRV-033: Branch List
- REQ-SRV-034: Branch Switch
- REQ-SRV-048: Branch Operations

### 6.3. Conflict Events

Conflict events communicate Git merge conflicts and resolution status.

#### 6.3.1. Merge Conflict Detected Event

**Event Type:** `merge.conflict_detected`

**Description:** Emitted when a Git merge operation encounters conflicts that require manual resolution.

**Event Payload:**
```json
{
  "event_type": "merge.conflict_detected",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "repository_id": "string",
    "repository_name": "string",
    "source_branch": "string",
    "target_branch": "string",
    "conflicting_files": [
      {
        "path": "string",
        "conflict_markers": integer
      }
    ],
    "detected_at": "ISO8601"
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `repository_id` | string | Yes | Unique identifier for repository |
| `repository_name` | string | Yes | Name of the repository |
| `source_branch` | string | Yes | Name of the source branch being merged |
| `target_branch` | string | Yes | Name of the target branch |
| `conflicting_files` | array | Yes | List of files with conflicts |
| `conflicting_files[].path` | string | Yes | Path to the conflicting file |
| `conflicting_files[].conflict_markers` | integer | Yes | Number of conflict markers in the file |
| `detected_at` | string (ISO8601) | Yes | Timestamp when the conflict was detected |

#### 6.3.2. Merge Conflict Resolved Event

**Event Type:** `merge.conflict_resolved`

**Description:** Emitted when a merge conflict has been resolved.

**Event Payload:**
```json
{
  "event_type": "merge.conflict_resolved",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "repository_id": "string",
    "repository_name": "string",
    "source_branch": "string",
    "target_branch": "string",
    "resolved_files": [
      "string"
    ],
    "resolved_by": "string",
    "resolved_at": "ISO8601",
    "resolution_strategy": "accept_theirs|accept_ours|manual"
  }
}
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `repository_id` | string | Yes | Unique identifier for repository |
| `repository_name` | string | Yes | Name of the repository |
| `source_branch` | string | Yes | Name of the source branch being merged |
| `target_branch` | string | Yes | Name of the target branch |
| `resolved_files` | array | Yes | List of files that were resolved |
| `resolved_by` | string | Yes | User ID of the user who resolved the conflict |
| `resolved_at` | string (ISO8601) | Yes | Timestamp when the conflict was resolved |
| `resolution_strategy` | string | Yes | Strategy used for resolution (accept_theirs, accept_ours, manual) |

**Related Requirements:**
- REQ-SRV-049: Merge Operations

### 6.4. Repository Event Subscription Mechanisms

Clients may subscribe to repository-specific events to receive updates for particular repositories.

#### 6.4.1. Repository Event Subscription Request

**Subscription Request Format:**
```json
{
  "action": "subscribe",
  "subscriptions": [
    {
      "event_type": "repository.sync_started",
      "filters": {
        "repository_id": "repo-789"
      }
    },
    {
      "event_type": "branch.switched",
      "filters": {
        "repository_id": "repo-789"
      }
    },
    {
      "event_type": "merge.conflict_detected",
      "filters": {
        "repository_id": "repo-789"
      }
    }
  ]
}
```

#### 6.4.2. Repository Event Subscription Filters

**Supported Filter Fields:**

| Filter Field | Type | Description |
|-------------|------|-------------|
| `repository_id` | string | Repository ID to filter events for |
| `branch_name` | string | Branch name to filter events for |

**Filter Examples:**

**Subscribe to sync events for a specific repository:**
```json
{
  "event_type": "repository.sync_started",
  "filters": {
    "repository_id": "repo-789"
  }
}
```

**Subscribe to branch changes for a specific branch:**
```json
{
  "event_type": "branch.switched",
  "filters": {
    "repository_id": "repo-789",
    "branch_name": "main"
  }
}
```

---

## 7. EVENT SECURITY

### 7.1. Authentication Requirements

WebSocket connections require authentication before receiving or sending events. Authentication ensures that only authorized users may establish connections and receive events for resources they have permission to access.

#### 7.1.1. Connection Authentication

WebSocket connections must be authenticated during the initial handshake using one of the following methods:

**Method 1: Query Parameter Authentication**
```
wss://tachyon.example.com/ws?token=<session_token>
```

**Method 2: HTTP Header Authentication**
```
GET /ws HTTP/1.1
Host: tachyon.example.com
Authorization: Bearer <session_token>
Upgrade: websocket
Connection: Upgrade
```

**Authentication Flow:**

1. Client initiates WebSocket connection with authentication token
2. Server validates token and extracts user identity
3. Server emits `connection.established` event upon successful authentication
4. Server emits `error` event with `AUTH_FAILED` code upon authentication failure

**Token Types:**

| Token Type | Description | Validity Period |
|------------|-------------|-----------------|
| Session Token | JWT token issued after login | 24 hours |
| OAuth 2.0 Bearer Token | OAuth 2.0 access token | Provider-specific |
| API Key | Long-lived API key for service accounts | 90 days |

**Related Requirements:**
- REQ-SRV-076: Session Management
- REQ-SRV-092: Connection Authentication

#### 7.1.2. Token Refresh

WebSocket connections support token refresh without requiring reconnection. Clients may send a token refresh message when their current token is approaching expiration.

**Token Refresh Request:**
```json
{
  "action": "refresh_token",
  "data": {
    "refresh_token": "string"
  }
}
```

**Token Refresh Response:**
```json
{
  "action": "refresh_token_response",
  "status": "success|failed",
  "data": {
    "new_token": "string",
    "expires_at": "ISO8601"
  }
}
```

**Related Requirements:**
- REQ-SRV-087: Session Refresh

### 7.2. Authorization Requirements

Event delivery is subject to Role-Based Access Control (RBAC) to ensure that clients receive only events for resources they have permission to access.

#### 7.2.1. Event Authorization Model

The server enforces authorization for each event before delivery to a client. The authorization model follows these principles:

**Principle of Least Privilege:**
Clients receive only the minimum set of events necessary for their current context. Additional events require explicit subscription and authorization.

**Document-Level Authorization:**
Document events are filtered based on the client's access to the specific document. Access control is determined by:

1. Frontmatter access control directives
2. Repository-level permissions
3. User role and permissions

**Repository-Level Authorization:**
Repository events are filtered based on the client's access to the specific repository. Access control is determined by:

1. Repository visibility settings
2. User role and permissions
3. Team membership

**User Event Authorization:**
User presence and activity events are filtered based on the client's relationship to the user. Presence events are delivered only for:

1. Users in the same repository
2. Users with shared document access
3. Users in the same team

#### 7.2.2. Authorization Enforcement

Authorization is enforced at multiple layers:

**Layer 1: Subscription Validation**
When a client subscribes to an event type, the server validates that the client has permission to receive events of that type.

**Layer 2: Event Filtering**
Before delivering an event to a client, the server validates that the client has permission to receive the specific event instance.

**Layer 3: Content Redaction**
For events containing sensitive content, the server redacts information based on the client's access level. Internal blocks marked with `::: internal` are removed from event payloads for unauthorized users.

**Related Requirements:**
- REQ-SRV-081: RBAC Enforcement
- REQ-SRV-082: Frontmatter Access Control
- REQ-SRV-083: Block Redaction

### 7.3. Rate Limiting

Rate limiting prevents abuse of the Events API through excessive connection attempts, subscription requests, or event flooding.

#### 7.3.1. Connection Rate Limiting

The server limits the number of WebSocket connections per user and per IP address to prevent connection flooding.

**Rate Limits:**

| Limit Type | Threshold | Time Window | Exceed Action |
|------------|-----------|-------------|---------------|
| Connections per User | 5 | 1 minute | Reject new connections |
| Connections per IP | 20 | 1 minute | Reject new connections |
| Connection Attempts | 100 | 1 hour | Temporary IP ban |

**Rate Limit Exceeded Event:**
```json
{
  "event_type": "error",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "error_code": "RATE_LIMITED",
    "error_message": "Connection rate limit exceeded",
    "error_details": {
      "limit_type": "connections_per_user|connections_per_ip",
      "threshold": integer,
      "retry_after_seconds": integer
    }
  }
}
```

#### 7.3.2. Subscription Rate Limiting

The server limits the number of subscription requests per client to prevent subscription flooding.

**Rate Limits:**

| Limit Type | Threshold | Time Window | Exceed Action |
|------------|-----------|-------------|---------------|
| Subscription Requests | 50 | 1 minute | Reject new requests |
| Active Subscriptions | 100 | Per connection | Reject new subscriptions |

**Related Requirements:**
- REQ-SRV-118: Rate Limiting

### 7.4. DDoS Protection

The server implements multiple layers of DDoS protection to ensure availability of the Events API during attack scenarios.

#### 7.4.1. Connection-Level Protection

**Connection Throttling:**
The server throttles new connection acceptance when detecting connection flood patterns. Connection acceptance rate is dynamically adjusted based on:

1. Current connection count
2. Historical connection patterns
3. IP reputation score

**Connection Validation:**
Before accepting a connection, the server validates:

1. Client TLS certificate (if using mTLS)
2. IP address reputation
3. Geolocation consistency

#### 7.4.2. Event-Level Protection

**Event Batching:**
The server batches events to reduce per-message processing overhead. Events are delivered in batches of up to 100 events with a maximum delay of 50 milliseconds.

**Event Throttling:**
When detecting event flooding from a specific client, the server throttles event delivery to that client. Throttling is applied when:

1. Client sends more than 1000 events per minute
2. Client sends malformed events repeatedly

**Throttling Notification:**
```json
{
  "event_type": "error",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "error_code": "THROTTLED",
    "error_message": "Event delivery throttled",
    "error_details": {
      "throttle_duration_seconds": integer,
      "reason": "event_flood|malformed_events"
    }
  }
}
```

#### 7.4.3. Infrastructure-Level Protection

The server integrates with external DDoS protection services for additional protection against volumetric attacks.

**Protection Services:**

| Service | Protection Type | Integration Method |
|---------|---------------|-------------------|
| Cloudflare | Volumetric DDoS | DNS proxy |
| AWS Shield | Volumetric and protocol DDoS | Network layer |
| Custom WAF | Application layer DDoS | Reverse proxy |

**Related Requirements:**
- REQ-SRV-117: Request Size Limits
- REQ-SRV-119: Connection Timeouts

### 7.5. Event Payload Security

Event payloads are secured to prevent information leakage and injection attacks.

#### 7.5.1. Content Sanitization

All user-generated content within event payloads is sanitized to prevent XSS and injection attacks. Sanitization is applied to:

1. Document content previews
2. User display names
3. Error messages
4. Custom metadata fields

**Sanitization Rules:**

| Content Type | Sanitization Method |
|-------------|-------------------|
| HTML content | Strip tags, escape entities |
| JavaScript | Remove or escape |
| SQL keywords | Escape or parameterize |
| File paths | Validate and canonicalize |

#### 7.5.2. Sensitive Data Redaction

Sensitive data is redacted from event payloads based on the client's access level. Redaction is applied to:

1. Internal blocks marked with `::: internal`
2. Documents with restricted access
3. User personal information (unless authorized)

**Redaction Format:**
```json
{
  "event_type": "document.updated",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "document_id": "string",
    "content_preview": "[REDACTED: Internal content]",
    "frontmatter": {
      "title": "string",
      "internal_notes": "[REDACTED]"
    }
  }
}
```

**Related Threat Model References:**
- Threat: Information Disclosure (Section 2.4)
- Threat: Data Exfiltration (Section 2.4.1)
- Threat: XSS via WebView (Section 3.1.1)

---

## 8. EVENT PERFORMANCE

### 8.1. Latency Requirements

The Events API maintains strict latency requirements to ensure real-time responsiveness for collaborative features.

#### 8.1.1. Event Delivery Latency

Events are delivered to clients within bounded latency windows to ensure real-time user experience.

**Latency Requirements:**

| Metric | Requirement | Target | Maximum |
|--------|-------------|--------|----------|
| Event Delivery Latency (P50) | 50th percentile | 25 ms | 50 ms |
| Event Delivery Latency (P95) | 95th percentile | 40 ms | 100 ms |
| Event Delivery Latency (P99) | 99th percentile | 50 ms | 200 ms |
| Connection Establishment | Time to `connection.established` | 100 ms | 500 ms |
| Subscription Confirmation | Time to `subscription.confirmed` | 50 ms | 200 ms |

**Latency Measurement:**
Latency is measured as the time difference between:
1. Event occurrence timestamp (event generation)
2. Event delivery timestamp (when client receives event)

**Related Requirements:**
- REQ-SRV-109: WebSocket Latency

#### 8.1.2. Heartbeat Latency

Heartbeat messages must be processed and acknowledged within strict latency bounds to maintain connection health detection.

**Heartbeat Latency Requirements:**

| Metric | Requirement | Target | Maximum |
|--------|-------------|--------|----------|
| Heartbeat Processing (P50) | 50th percentile | 10 ms | 25 ms |
| Heartbeat Processing (P95) | 95th percentile | 20 ms | 50 ms |
| Heartbeat Acknowledgment | Client response time | 50 ms | 100 ms |

### 8.2. Throughput Requirements

The Events API supports high throughput to accommodate real-time updates from multiple concurrent users.

#### 8.2.1. Event Throughput

The server processes and delivers events at high throughput to support collaborative editing scenarios.

**Throughput Requirements:**

| Metric | Requirement | Target |
|--------|-------------|--------|
| Events per Second (Server) | Total events processed | 100,000 |
| Events per Second per Connection | Events delivered to single client | 1,000 |
| Concurrent Connections | Active WebSocket connections | 500 |
| Concurrent Subscriptions | Active subscriptions across all clients | 10,000 |

**Throughput Optimization:**
The server implements multiple optimization strategies to achieve throughput targets:

1. **Event Batching:** Multiple events are batched for efficient delivery
2. **Binary Protocol:** Optional binary message format for reduced payload size
3. **Compression:** Message compression for large event payloads
4. **Connection Pooling:** Efficient connection management using Tokio runtime

**Related Requirements:**
- REQ-SRV-111: Concurrent Users
- REQ-SRV-113: WebSocket Connections

#### 8.2.2. Message Size Limits

Message size limits prevent resource exhaustion and ensure fair resource allocation.

**Message Size Requirements:**

| Message Type | Maximum Size | Rationale |
|-------------|---------------|-----------|
| Event Payload | 1 MB | Prevents memory exhaustion |
| Subscription Request | 100 KB | Limits subscription complexity |
| Client Message | 1 MB | Prevents client flooding |

**Oversized Message Handling:**
Messages exceeding size limits are rejected with an appropriate error event:

```json
{
  "event_type": "error",
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "data": {
    "error_code": "MESSAGE_TOO_LARGE",
    "error_message": "Message exceeds maximum size limit",
    "error_details": {
      "actual_size_bytes": integer,
      "maximum_size_bytes": integer
    }
  }
}
```

### 8.3. Optimization Strategies

The Events API implements multiple optimization strategies to achieve performance requirements.

#### 8.3.1. Event Batching

Events are batched for efficient delivery, reducing per-message overhead and improving throughput.

**Batching Configuration:**

| Parameter | Default Value | Description |
|-----------|---------------|-------------|
| Batch Size | 50 events | Maximum events per batch |
| Batch Timeout | 50 ms | Maximum time to wait before sending batch |
| Priority Batching | Enabled | High-priority events bypass batching |

**Batch Format:**
```json
{
  "batch": true,
  "batch_id": "uuid",
  "event_count": integer,
  "events": [
    {
      "event_type": "string",
      "event_id": "uuid",
      "timestamp": "ISO8601",
      "data": { }
    }
  ]
}
```

#### 8.3.2. Event Compression

Large event payloads are compressed to reduce bandwidth usage and improve delivery latency.

**Compression Configuration:**

| Parameter | Default Value | Description |
|-----------|---------------|-------------|
| Compression Threshold | 1 KB | Minimum payload size for compression |
| Compression Algorithm | gzip | Compression algorithm used |
| Compression Level | 6 | Compression level (1-9, higher = more compression) |

**Compressed Message Format:**
```json
{
  "compressed": true,
  "compression_algorithm": "gzip",
  "original_size_bytes": integer,
  "compressed_size_bytes": integer,
  "payload": "base64_encoded_compressed_data"
}
```

#### 8.3.3. Event Caching

Frequently accessed events are cached to reduce processing overhead and improve delivery latency.

**Cache Configuration:**

| Parameter | Default Value | Description |
|-----------|---------------|-------------|
| Cache Size | 10,000 events | Maximum events in cache |
| Cache TTL | 300 seconds | Time-to-live for cached events |
| Cache Hit Rate Target | 80% | Target cache hit rate |

**Cache Invalidation:**
Cached events are invalidated when:

1. Underlying data changes
2. User permissions change
3. Subscription filters change
4. TTL expires

#### 8.3.4. Connection Pooling

WebSocket connections are managed efficiently using Tokio's connection pooling and work-stealing scheduler.

**Pooling Configuration:**

| Parameter | Default Value | Description |
|-----------|---------------|-------------|
| Worker Threads | CPU core count | Number of worker threads |
| Connection Per Worker | 100 | Maximum connections per worker |
| Connection Queue Size | 1,000 | Pending connection queue size |

**Related ADR References:**
- ADR-007: Tokio for Async Runtime (Section 4.3: Work-Stealing Scheduler)

### 8.4. Monitoring and Metrics

The Events API provides comprehensive monitoring and metrics to track performance characteristics.

#### 8.4.1. Performance Metrics

The following metrics are collected and exposed for monitoring:

**Connection Metrics:**

| Metric | Type | Description |
|--------|------|-------------|
| `websocket_connections_active` | Gauge | Current active WebSocket connections |
| `websocket_connections_total` | Counter | Total WebSocket connections established |
| `websocket_connections_failed` | Counter | Total WebSocket connection failures |
| `websocket_connection_duration_seconds` | Histogram | WebSocket connection duration |

**Event Metrics:**

| Metric | Type | Description |
|--------|------|-------------|
| `events_generated_total` | Counter | Total events generated |
| `events_delivered_total` | Counter | Total events delivered |
| `events_dropped_total` | Counter | Total events dropped (rate limiting, etc.) |
| `event_delivery_latency_seconds` | Histogram | Event delivery latency |
| `event_processing_duration_seconds` | Histogram | Event processing duration |

**Subscription Metrics:**

| Metric | Type | Description |
|--------|------|-------------|
| `subscriptions_active` | Gauge | Current active subscriptions |
| `subscriptions_total` | Counter | Total subscriptions created |
| `subscriptions_failed` | Counter | Total subscription failures |

#### 8.4.2. Performance Alerts

Alerts are generated when performance metrics exceed defined thresholds.

**Alert Thresholds:**

| Metric | Warning Threshold | Critical Threshold | Alert Action |
|--------|-------------------|-------------------|-------------|
| Event Delivery Latency (P95) | 75 ms | 150 ms | Notify operations team |
| Event Delivery Latency (P99) | 150 ms | 300 ms | Escalate to engineering |
| Active Connections | 400 | 450 | Scale infrastructure |
| Event Drop Rate | 0.1% | 1% | Investigate bottlenecks |
| Cache Hit Rate | 70% | 50% | Review cache configuration |

**Related Requirements:**
- REQ-SRV-110: Cache Hit Rate
- REQ-SRV-115: Async Processing

---

## 9. REFERENCES

### 9.1. Internal Project References

This document references the following Tachyon project documents:

**Standards Documents:**
- [TACHYON-STD-V1.0](../../.specs/01_standards/coding_standards.md) - Coding and Documentation Standards

**Requirements Documents:**
- [TACHYON-REQ-SRV-V1.0](../../.specs/04_future_state/reqs/server_requirements.md) - Server Application Requirements

**Architectural Decision Records:**
- [TACHYON-ADR-003-V1.0](../../.specs/02_adrs/003_axum_for_http2_server.md) - Axum for HTTP/2 Server
- [TACHYON-ADR-007-V1.0](../../.specs/02_adrs/007_tokio_for_async_runtime.md) - Tokio for Async Runtime

**Threat Model Documents:**
- [TACHYON-TMA-V1.0](../../.specs/03_threat_model/analysis.md) - Threat Model Analysis

**Task Documentation:**
- [TACHYON-TSK-V1.0](../../.specs/tasks.md) - Execution Tasks and Work Breakdown Structure

### 9.2. External Standards and Specifications

This document references the following external standards and specifications:

**WebSocket Protocol:**
- RFC 6455 - The WebSocket Protocol, IETF, 2011

**HTTP/2 Protocol:**
- RFC 7540 - Hypertext Transfer Protocol Version 2 (HTTP/2), IETF, 2015

**TLS Protocol:**
- RFC 8446 - The Transport Layer Security (TLS) Protocol Version 1.3, IETF, 2018

**ISO/IEC Standards:**
- ISO/IEC 26514:2021 - Systems and Software Engineering — Requirements for Designers and Developers of User Documentation
- ISO/IEC 12207:2017 - Systems and Software Engineering — Software Life Cycle Processes
- ISO/IEC 25010:2011 - Systems and Software Engineering — Systems and Software Quality Requirements

**IEEE Standards:**
- IEEE 829-2008 - Software Test Documentation
- IEEE 1063-2001 - Standard for Software User Documentation
- IEEE 1016-2009 - Standard for Information Technology — Software Design Descriptions

### 9.3. Requirements Traceability

The following requirements from TACHYON-REQ-SRV-V1.0 are addressed in this specification:

| Requirement ID | Description | Section |
|----------------|-------------|---------|
| REQ-SRV-091 | WebSocket Endpoint | 3.1.1 |
| REQ-SRV-092 | Connection Authentication | 7.1.1 |
| REQ-SRV-093 | Connection Limiting | 7.3.1 |
| REQ-SRV-094 | Heartbeat Mechanism | 3.1.2 |
| REQ-SRV-095 | Graceful Disconnection | 3.1.3 |
| REQ-SRV-096 | Content Updates | 4.1.2 |
| REQ-SRV-097 | User Presence | 5.1.1 |
| REQ-SRV-098 | Conflict Notifications | 4.2.1 |
| REQ-SRV-099 | Typing Indicators | 5.2 |
| REQ-SRV-100 | Cursor Position | 5.3 |
| REQ-SRV-101 | Last-Write-Wins | 4.2.1 |
| REQ-SRV-102 | Edit Queue | 4.2.2 |
| REQ-SRV-104 | Edit History | 4.2.2 |
| REQ-SRV-109 | WebSocket Latency | 8.1.1 |
| REQ-SRV-110 | Cache Hit Rate | 8.3.3 |
| REQ-SRV-111 | Concurrent Users | 8.2.1 |
| REQ-SRV-113 | WebSocket Connections | 8.2.1 |
| REQ-SRV-115 | Async Processing | 8.3.4 |
| REQ-SRV-116 | Memory Limits | 8.2.2 |
| REQ-SRV-117 | Request Size Limits | 8.2.2 |
| REQ-SRV-118 | Rate Limiting | 7.3 |
| REQ-SRV-119 | Connection Timeouts | 7.4.2 |
| REQ-SRV-076 | Session Management | 7.1.1 |
| REQ-SRV-081 | RBAC Enforcement | 7.2.1 |
| REQ-SRV-082 | Frontmatter Access Control | 7.2.1 |
| REQ-SRV-083 | Block Redaction | 7.5.2 |

### 9.4. ADR Traceability

The following Architectural Decision Records inform the design decisions in this specification:

| ADR ID | Title | Relevance |
|---------|-------|-----------|
| ADR-003 | Axum for HTTP/2 Server | WebSocket framework selection and async architecture |
| ADR-007 | Tokio for Async Runtime | Async runtime for WebSocket connection management |
| ADR-010 | Security Architecture | Authentication, authorization, and security controls |

### 9.5. Threat Model Traceability

The following threat model considerations are addressed in this specification:

| Threat Category | Threat | Mitigation Section |
|----------------|--------|-------------------|
| Spoofing | User Identity Spoofing | 7.1.1 |
| Spoofing | System Component Spoofing | 7.1.1 |
| Information Disclosure | Data Exfiltration | 7.5.2 |
| Information Disclosure | Unauthorized Information Access | 7.2.1 |
| Denial of Service | Resource Exhaustion | 7.4 |
| Denial of Service | Logic-Based DoS | 7.4.2 |
| Tampering | Data Tampering | 7.5.1 |

### 9.6. Bibliography

[1] IETF, "RFC 6455: The WebSocket Protocol," December 2011. [Online]. Available: https://tools.ietf.org/html/rfc6455

[2] IETF, "RFC 7540: Hypertext Transfer Protocol Version 2 (HTTP/2)," May 2015. [Online]. Available: https://tools.ietf.org/html/rfc7540

[3] IETF, "RFC 8446: The Transport Layer Security (TLS) Protocol Version 1.3," August 2018. [Online]. Available: https://tools.ietf.org/html/rfc8446

[4] ISO/IEC, "ISO/IEC 26514:2021 Systems and Software Engineering — Requirements for Designers and Developers of User Documentation," 2021.

[5] ISO/IEC, "ISO/IEC 12207:2017 Systems and Software Engineering — Software Life Cycle Processes," 2017.

[6] ISO/IEC, "ISO/IEC 25010:2011 Systems and Software Engineering — Systems and Software Quality Requirements," 2011.

[7] IEEE, "IEEE 829-2008 Standard for Software Test Documentation," 2008.

[8] IEEE, "IEEE 1063-2001 Standard for Software User Documentation," 2001.

[9] IEEE, "IEEE 1016-2009 Standard for Information Technology — Software Design Descriptions," 2009.

---

**Document Change History:**

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-02-05 | Technical Writer | Initial version - Complete Server Events API specification |
