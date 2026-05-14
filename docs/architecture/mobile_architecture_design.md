# Mobile Application Architecture Design (G.4)

## 1. Overview

Mobile client providing native access to the Tachyon document platform. Key requirements:

- **Framework**: React Native or Flutter
- **Data strategy**: Offline-first with CRDT-based synchronization (see G.2)
- **Authentication**: Biometric (Face ID, fingerprint) with secure token storage
- **Notifications**: Push notifications for document updates, mentions, and collaboration events
- **Target platforms**: iOS 15+, Android 12+

The mobile client consumes the same REST/WebSocket API as the web client and shares the TypeScript API layer where feasible.

## 2. Framework Selection

### React Native

- Leverages existing TypeScript API client and shared types from the web codebase
- Shared business logic (CRDT engine, document models, validation) between web and mobile
- Mature ecosystem with Expo for managed workflow
- Large community and hiring pool
- Native module bridge adds complexity for performance-critical paths

### Flutter

- Superior rendering performance (Skia-based engine)
- Single codebase for iOS, Android, and potentially desktop
- Dart is not shared with the existing TypeScript codebase
- Requires separate API client implementation or code generation
- Strong widget system for complex UIs

### Recommendation

**React Native**. The primary driver is code reuse: the CRDT engine, API client types, validation logic, and document models are already implemented in TypeScript. Maintaining a single codebase for shared logic reduces duplication and divergence risk. Flutter's performance advantage does not justify the cost of a separate Dart codebase at this stage.

## 3. Offline-First Sync

### Local Storage

- **SQLite** via WatermelonDB (reactive, lazy-loading ORM for RN) or Drizzle ORM
- IndexedDB as fallback on web, SQLite on mobile
- Schema mirrors the server document structure

### CRDT Integration

- Same CRDT engine as G.2 runs locally on the device
- Edits are applied to the local CRDT state immediately
- Conflict resolution is deterministic and does not require server coordination
- Sync protocol: client pushes operations to server, server merges and broadcasts

### Sync Strategy

- Background sync triggered by connectivity changes (NetInfo)
- Incremental sync: only changed operations are transmitted
- Sync queue persists to local storage to survive app restarts
- Conflict resolution handled by CRDT merge (last-writer-wins for metadata, operational transform for content)
- Manual pull-to-refresh for immediate sync

### Sync Lifecycle

```
[Local Edit] -> [CRDT Apply] -> [Queue Operation] -> [Network Available]
    -> [Push to Server] -> [Server Merge] -> [Broadcast] -> [Other Clients]
```

## 4. Push Notifications

### Infrastructure

- **Android**: Firebase Cloud Messaging (FCM)
- **iOS**: Apple Push Notification Service (APNs)
- Server-side notification service aggregates events and dispatches to FCM/APNs

### Notification Types

| Type | Trigger | Payload |
|------|---------|---------|
| Document mention | User mentioned in document | Document ID, author, snippet |
| Comment reply | Reply to user's comment | Comment ID, document ID, author |
| Collaboration join | User added to document space | Space ID, inviter |
| Sync conflict | Manual resolution required | Document ID, conflict summary |
| System | Maintenance, updates | Message, severity |

### Per-User Preferences

- Toggle per notification type
- Quiet hours configuration
- Aggregate digest mode (batch notifications into periodic summaries)

### Implementation

- FCM/APNs credentials stored server-side per environment
- Device tokens registered on app install, updated on token refresh
- Notification service uses existing event bus to subscribe to relevant events
- Tap action routes to the relevant document or screen via deep linking

## 5. Biometric Authentication

### Token Storage

- JWT access token stored in memory only (short-lived, 15-minute TTL)
- JWT refresh token stored in platform secure storage:
  - **iOS**: Keychain Services
  - **Android**: Android Keystore System (via `react-native-keychain`)
- Refresh token is encrypted at rest and never exposed to the application layer in plaintext

### Authentication Flow

```
[App Launch] -> [Biometric Prompt] -> [Success]
    -> [Read Refresh Token from Keychain] -> [POST /auth/refresh]
    -> [Receive Access Token] -> [App Unlocked]

[Biometric Prompt] -> [Failure/Cancel]
    -> [PIN Fallback] -> [Verify PIN] -> [Same refresh flow]

[PIN Failure (3 attempts)] -> [Full Re-authentication Required]
```

### Biometric Enrollment

- Biometric unlock is optional; users can opt for PIN-only
- Enrollment stores a boolean preference server-side (`biometric_enabled`)
- Keychain/Keystore operations are gated behind biometric verification
- Fallback PIN is set during initial biometric enrollment

## 6. Navigation

### Tab Structure

| Tab | Primary Content |
|-----|----------------|
| Documents | Recent documents, pinned, favorites |
| Search | Full-text search with filters |
| Spaces | Shared spaces and collaboration |
| Settings | Account, preferences, sync status |

### Document Editor

- Split view: editor pane and markdown preview pane (iPad/tablet)
- Single pane with toggle on phone
- Toolbar: formatting, mention insertion, image attachment
- Real-time collaboration indicators (cursor positions, active editors) via WebSocket
- Autosave with local CRDT state; sync on connectivity

### Deep Linking

- `tachyon://document/{id}` opens specific document
- `tachyon://space/{id}` opens space
- Push notification taps route through deep links

## 7. Implementation Priority

| Phase | Scope | Duration |
|-------|-------|----------|
| 1 | Core app: auth, document list, editor, basic navigation | 3 weeks |
| 2 | Offline-first sync: SQLite, CRDT local engine, background sync | 3 weeks |
| 3 | Push notifications: FCM/APNs integration, notification service | 2 weeks |
| 4 | Biometric auth: Keychain/Keystore, enrollment, PIN fallback | 1 week |
| 5 | Polish: deep linking, collaboration indicators, tablet layout | 1 week |

**Total estimated effort: 10 weeks** (1 senior mobile engineer)
