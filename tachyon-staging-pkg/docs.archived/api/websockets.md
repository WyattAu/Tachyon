# WebSocket Events Reference

Complete reference for WebSocket events and messages.

## Connection

### Endpoint

```
ws://localhost:8080/ws
wss://api.example.com/ws
```

### Authentication

Include JWT token in query parameter:

```javascript
const ws = new WebSocket('wss://api.example.com/ws?token=YOUR_JWT_TOKEN');
```

---

## Message Format

All messages use JSON:

```typescript
interface Message {
  type: string;
  payload: any;
  timestamp: string;
  id?: string;
}
```

---

## Client Events

### document.join

Join a document for real-time collaboration.

**Payload**

```json
{
  "document_id": "00000000-0000-0000-0000-000000000001"
}
```

**Example**

```json
{
  "type": "document.join",
  "payload": {
    "document_id": "00000000-0000-0000-0000-000000000001"
  }
}
```

---

### document.leave

Leave a document.

**Payload**

```json
{
  "document_id": "00000000-0000-0000-0000-000000000001"
}
```

---

### cursor.update

Update cursor position.

**Payload**

```json
{
  "document_id": "00000000-0000-0000-0000-000000000001",
  "position": {
    "line": 42,
    "column": 15
  }
}
```

---

### document.edit

Send document edits.

**Payload**

```json
{
  "document_id": "00000000-0000-0000-0000-000000000001",
  "version": 5,
  "operations": [
    {
      "type": "insert",
      "position": 245,
      "text": "Hello, "
    },
    {
      "type": "delete",
      "position": 252,
      "length": 5
    }
  ]
}
```

---

### ping

Keep connection alive.

**Payload**

```json
{}
```

---

## Server Events

### pong

Response to ping.

**Payload**

```json
{}
```

---

### presence.update

User presence update.

**Payload**

```json
{
  "document_id": "00000000-0000-0000-0000-000000000001",
  "users": [
    {
      "user_id": "00000000-0000-0000-0000-000000000100",
      "name": "John Doe",
      "color": "#FF5733",
      "cursor": {
        "line": 42,
        "column": 15
      }
    }
  ]
}
```

---

### cursor.broadcast

Broadcast cursor position.

**Payload**

```json
{
  "document_id": "00000000-0000-0000-0000-000000000001",
  "user_id": "00000000-0000-0000-0000-000000000100",
  "name": "Jane Doe",
  "color": "#33FF57",
  "position": {
    "line": 20,
    "column": 8
  }
}
```

---

### document.sync

Document synchronization.

**Payload**

```json
{
  "document_id": "00000000-0000-0000-0000-000000000001",
  "version": 6,
  "content": "Updated content...",
  "operations": [
    {
      "type": "insert",
      "position": 245,
      "text": "Hello, "
    }
  ]
}
```

---

### error

Error message.

**Payload**

```json
{
  "code": "DOCUMENT_NOT_FOUND",
  "message": "Document not found"
}
```

---

### notification

User notification.

**Payload**

```json
{
  "type": "comment",
  "title": "New Comment",
  "message": "John commented on your document",
  "data": {
    "document_id": "00000000-0000-0000-0000-000000000001",
    "comment_id": "00000000-0000-0000-0000-000000000200"
  }
}
```

---

## Operation Types

### Insert

Insert text at position.

```json
{
  "type": "insert",
  "position": 100,
  "text": "inserted text"
}
```

### Delete

Delete text from position.

```json
{
  "type": "delete",
  "position": 100,
  "length": 10
}
```

### Retain

Keep text as-is.

```json
{
  "type": "retain",
  "position": 100,
  "length": 50
}
```

---

## Error Codes

| Code | Description |
|------|-------------|
| `UNAUTHORIZED` | Authentication required |
| `DOCUMENT_NOT_FOUND` | Document not found |
| `PERMISSION_DENIED` | Insufficient permissions |
| `INVALID_OPERATION` | Invalid edit operation |
| `VERSION_CONFLICT` | Version conflict detected |

---

## Connection Lifecycle

```
1. Connect with token
2. Receive "connected" event
3. Join documents
4. Send/receive events
5. Leave documents
6. Disconnect
```

---

## Best Practices

1. **Reconnect on disconnect** with exponential backoff
2. **Send pings** every 30 seconds
3. **Handle errors** gracefully
4. **Buffer operations** during reconnection
5. **Use version numbers** to detect conflicts

---

## Next Steps

- [Developer: WebSockets](../developer/websockets.md)
- [Documents API](documents.md)
- [Authentication API](authentication.md)
