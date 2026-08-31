# WebSocket Protocol

Documentation of Tachyon's real-time WebSocket protocol.

## Overview

WebSocket enables real-time features:
- Live document updates
- User presence
- Cursor positions
- Collaborative editing

## Connection

### Endpoint

```
wss://tachyon.example.com/ws
```

### Authentication

Query parameter:
```
wss://tachyon.example.com/ws?token=<jwt>
```

Or first message after connection:
```json
{
  "type": "auth",
  "token": "<jwt>"
}
```

## Message Format

All messages are JSON:

```typescript
interface Message {
  type: string;
  payload: any;
  timestamp?: string;
  id?: string;
}
```

## Client → Server Messages

### Authentication

```json
{
  "type": "auth",
  "token": "<jwt>"
}
```

Response:
```json
{
  "type": "auth:success",
  "user": {
    "id": "user_123",
    "username": "johndoe"
  }
}
```

### Join Document

```json
{
  "type": "document:join",
  "documentId": "doc_abc123"
}
```

Response:
```json
{
  "type": "document:joined",
  "documentId": "doc_abc123",
  "users": [
    {"id": "user_123", "username": "johndoe", "color": "#FF5733"}
  ]
}
```

### Leave Document

```json
{
  "type": "document:leave",
  "documentId": "doc_abc123"
}
```

### Cursor Update

```json
{
  "type": "cursor:move",
  "documentId": "doc_abc123",
  "position": {
    "line": 10,
    "column": 5
  },
  "selection": {
    "start": {"line": 10, "column": 5},
    "end": {"line": 10, "column": 15}
  }
}
```

### Document Edit

```json
{
  "type": "document:edit",
  "documentId": "doc_abc123",
  "version": 5,
  "operations": [
    {
      "type": "insert",
      "position": {"line": 10, "column": 5},
      "text": "Hello"
    }
  ]
}
```

### Comment

```json
{
  "type": "comment:create",
  "documentId": "doc_abc123",
  "comment": {
    "text": "This needs review",
    "position": {"line": 15, "column": 0},
    "threadId": null
  }
}
```

### Ping

```json
{
  "type": "ping"
}
```

Response:
```json
{
  "type": "pong",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

## Server → Client Messages

### User Joined

```json
{
  "type": "presence:join",
  "documentId": "doc_abc123",
  "user": {
    "id": "user_456",
    "username": "janedoe",
    "color": "#33FF57"
  }
}
```

### User Left

```json
{
  "type": "presence:leave",
  "documentId": "doc_abc123",
  "userId": "user_456"
}
```

### Cursor Update

```json
{
  "type": "cursor:update",
  "documentId": "doc_abc123",
  "userId": "user_456",
  "position": {
    "line": 12,
    "column": 8
  },
  "selection": null
}
```

### Document Update

```json
{
  "type": "document:update",
  "documentId": "doc_abc123",
  "version": 6,
  "operations": [
    {
      "type": "insert",
      "userId": "user_456",
      "position": {"line": 10, "column": 5},
      "text": "Hello"
    }
  ]
}
```

### Comment Event

```json
{
  "type": "comment:create",
  "documentId": "doc_abc123",
  "comment": {
    "id": "comment_xyz",
    "userId": "user_456",
    "username": "janedoe",
    "text": "This needs review",
    "position": {"line": 15, "column": 0},
    "createdAt": "2024-01-15T10:30:00Z"
  }
}
```

### Error

```json
{
  "type": "error",
  "code": "DOCUMENT_NOT_FOUND",
  "message": "Document doc_abc123 not found"
}
```

## Operational Transform

For collaborative editing, Tachyon uses operational transform:

### Operation Types

**Insert:**
```json
{
  "type": "insert",
  "position": {"line": 10, "column": 5},
  "text": "Hello World"
}
```

**Delete:**
```json
{
  "type": "delete",
  "position": {"line": 10, "column": 5},
  "length": 11
}
```

**Replace:**
```json
{
  "type": "replace",
  "position": {"line": 10, "column": 5},
  "oldText": "Hello",
  "newText": "Hi"
}
```

### Version Control

Each edit increments version:

```json
{
  "type": "document:edit",
  "version": 5,
  "operations": [...]
}
```

Server rejects outdated edits:
```json
{
  "type": "error",
  "code": "VERSION_CONFLICT",
  "message": "Expected version 6, got 5",
  "currentVersion": 6
}
```

## Presence

### User Colors

Server assigns colors for cursor visibility:

```javascript
const colors = [
  '#FF5733', '#33FF57', '#3357FF', '#F333FF',
  '#FF33A1', '#33FFF5', '#F5FF33', '#FF8C33'
];
```

### Presence Timeout

Users are marked offline after 30 seconds of inactivity:
- No ping received
- No cursor updates

## Reconnection

### Session Recovery

Include last version on reconnect:

```json
{
  "type": "document:join",
  "documentId": "doc_abc123",
  "lastVersion": 10
}
```

Server sends missed operations:
```json
{
  "type": "document:catchup",
  "documentId": "doc_abc123",
  "fromVersion": 10,
  "toVersion": 15,
  "operations": [...]
}
```

## Message Acknowledgment

Request acknowledgment:
```json
{
  "type": "document:edit",
  "id": "msg_123",
  "operations": [...]
}
```

Acknowledgment:
```json
{
  "type": "ack",
  "id": "msg_123",
  "success": true
}
```

## Error Codes

| Code | Description |
|------|-------------|
| `AUTH_FAILED` | Invalid or expired token |
| `DOCUMENT_NOT_FOUND` | Document doesn't exist |
| `PERMISSION_DENIED` | No access to document |
| `VERSION_CONFLICT` | Edit conflicts with current version |
| `INVALID_MESSAGE` | Malformed message |
| `RATE_LIMITED` | Too many messages |

## Connection Lifecycle

```
Client                    Server
   |                         |
   |-------- CONNECT ------->|
   |                         |
   |<----- CONNECTED --------|
   |                         |
   |--------- AUTH --------->|
   |                         |
   |<---- AUTH:SUCCESS ------|
   |                         |
   |---- DOCUMENT:JOIN ----->|
   |                         |
   |<-- DOCUMENT:JOINED -----|
   |                         |
   |------ CURSOR:MOVE ----->|
   |                         |
   |<----- CURSOR:UPDATE ----|
   |                         |
   |------ (activity) ------>|
   |                         |
   |--------- PING --------->|
   |                         |
   |<--------- PONG ---------|
   |                         |
   |------- DISCONNECT ----->|
   |                         |
```

## Implementation Example

```javascript
const ws = new WebSocket('wss://tachyon.example.com/ws?token=xxx');

ws.onopen = () => {
  ws.send(JSON.stringify({
    type: 'document:join',
    documentId: 'doc_abc123'
  }));
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  
  switch (message.type) {
    case 'document:joined':
      console.log('Users:', message.users);
      break;
    case 'cursor:update':
      renderCursor(message.userId, message.position);
      break;
    case 'document:update':
      applyOperations(message.operations);
      break;
  }
};

// Send cursor update
function sendCursorUpdate(position) {
  ws.send(JSON.stringify({
    type: 'cursor:move',
    documentId: 'doc_abc123',
    position
  }));
}
```
