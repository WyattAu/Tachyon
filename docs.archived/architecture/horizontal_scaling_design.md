# Horizontal Scaling Architecture Design (G.6)

## 1. Overview

Scale the Tachyon platform from a single-server deployment to a horizontally distributed architecture capable of handling 10M+ documents and concurrent users across multiple server nodes.

Core requirements:

- **Stateless servers**: Any node can handle any request
- **Externalized state**: Sessions and cache in Redis cluster
- **Database scaling**: Read replicas for query distribution
- **Content delivery**: CDN for static assets and cacheable responses
- **WebSocket coordination**: Cross-node message routing

## 2. Stateless Servers

### Requirements for Statelessness

All server-side state must be externalized to shared infrastructure:

| Current State | Target Storage | Migration |
|---------------|---------------|-----------|
| In-memory session data | Redis | Session middleware pointing to Redis store |
| In-memory ApiCache | Redis cluster | Replace in-memory LRU with Redis GET/SET |
| WebSocket connection map | Redis Pub/Sub + local map | Broadcast via Pub/Sub, local map for connected clients |
| File uploads (local disk) | S3-compatible storage | Upload service writes to object store |

### Session Storage in Redis

- Sessions stored as Redis hashes with configurable TTL
- Session middleware reads from Redis on each request
- No sticky sessions required; any node can serve any request
- Session key format: `session:{session_id}`

### Cache Strategy

- Replace in-memory `ApiCache` with Redis `GET`/`SET`/`DEL` operations
- Cache invalidation via Redis Pub/Sub channel: `cache:invalidate:{key_pattern}`
- All nodes subscribe to invalidation channel and evict local cache entries
- TTL-based expiry as fallback for stale entries

### Request Routing

```
[Client] -> [Load Balancer (round-robin or least-conn)]
    -> [Server Node A] --or-- [Server Node B] --or-- [Server Node N]
    -> [Redis] (session lookup, cache)
    -> [PostgreSQL] (primary or replica)
    -> [Response]
```

Load balancer performs health checks on each node. Failed nodes are removed from the pool automatically.

## 3. Redis Cluster

### Topology

Minimum production deployment: 3 primary nodes + 3 replica nodes (6 nodes total). Redis Cluster uses hash slots (16384) distributed across primaries. Each primary has one replica for failover.

```
Primary 1 (slots 0-5460)    Primary 2 (slots 5461-10922)   Primary 3 (slots 10923-16383)
    |                              |                              |
Replica 1'                      Replica 2'                     Replica 3'
```

### Responsibilities

| Function | Redis Data Type | Key Pattern | TTL |
|----------|----------------|-------------|-----|
| Session storage | Hash | `session:{id}` | 24h |
| API cache | String (JSON) | `cache:{endpoint}:{hash}` | 5-60s |
| Rate limiting | String (counter) | `ratelimit:{tenant}:{window}` | Window duration |
| Pub/Sub (WebSocket) | Pub/Sub channel | `ws:document:{id}` | N/A (ephemeral) |
| Cache invalidation | Pub/Sub channel | `cache:invalidate:*` | N/A (ephemeral) |

### High Availability

- **Redis Sentinel** monitors primaries and triggers automatic failover
- Replica promotion on primary failure: < 10 seconds
- Application uses Sentinel-aware client to discover current primary
- Quorum set to 2 of 3 sentinels for failover decision

### WebSocket Pub/Sub

Cross-node WebSocket message routing:

```
[Client A on Node 1] -> [WebSocket Handler]
    -> [PUBLISH ws:document:{id} {message}]
    -> [All nodes SUBSCRIBED to ws:document:{id}]
    -> [Node 2 finds Client B connected] -> [Send to Client B]
```

Each node maintains a local map of connected WebSocket clients. On message publish, all nodes check their local map and deliver to matching clients.

## 4. Database Scaling

### Read Replicas

- 1 primary (write), N replicas (read)
- Application routes read queries to replicas, write queries to primary
- Replica lag target: < 1 second under normal load

```
[Application] -> [PgBouncer]
    -> [Primary] (writes, critical reads)
    -> [Replica 1] (reads)
    -> [Replica 2] (reads)
    -> [Replica N] (reads)
```

### Connection Pooling with PgBouncer

- PgBouncer in transaction-mode pooling
- Pool size per server: `max_connections * 0.8` (leaving headroom)
- PgBouncer handles schema switching for multi-tenant (G.5)
- Separate PgBouncer instance per deployment zone to minimize latency

### Query Optimization

#### Composite Indexes

```sql
-- Document search by tenant + updated timestamp
CREATE INDEX idx_documents_tenant_updated
  ON documents (tenant_id, updated_at DESC);

-- Full-text search with ranking
CREATE INDEX idx_documents_fts
  ON documents USING GIN (to_tsvector('english', title || ' ' || content));

-- Permission lookups by user + document
CREATE INDEX idx_permissions_user_document
  ON permissions (user_id, document_id);
```

#### Query Plan Analysis

- Run `EXPLAIN ANALYZE` on critical paths monthly
- Target: all queries under 100ms at p99
- Slow query log enabled with threshold of 200ms
- Automated alerting for queries exceeding 500ms

#### Partitioning (Optional for 10M+ Documents)

Range partition documents table by creation quarter if row count exceeds performance thresholds:

```sql
CREATE TABLE documents_2026_q1 PARTITION OF documents
  FOR VALUES FROM ('2026-01-01') TO ('2026-04-01');
```

## 5. CDN

### Static Asset Delivery

Assets served via CDN with long cache durations:

| Asset | Cache-Control | Invalidation |
|-------|---------------|--------------|
| Frontend WASM bundle | `public, max-age=31536000, immutable` | Versioned URL (content hash) |
| Images and attachments | `public, max-age=86400` | Purge on update |
| SSG output (public pages) | `public, max-age=3600, s-maxage=86400` | Rebuild trigger |
| Fonts | `public, max-age=31536000, immutable` | Versioned URL |

### Edge Caching for API Responses

Read-only API endpoints can be cached at the CDN edge:

- `GET /documents/{id}` with `Cache-Control: public, s-maxage=60`
- `GET /spaces/{id}` with `Cache-Control: public, s-maxage=120`
- Cache invalidation via CDN purge API on document/space update
- Stale-while-revalidate for eventual consistency

### CDN Configuration

- Origin: application server(s)
- Custom headers for CORS preflight caching
- Brotli compression enabled
- Geographic distribution matching user base

## 6. WebSocket Scaling

### Connection Model

Each WebSocket connection is pinned to a single server node for the duration of the connection. The local node handles message delivery for its connected clients.

### Cross-Node Routing

1. Client sends message to connected node
2. Node publishes to Redis Pub/Sub channel for the document
3. All nodes receive the message
4. Each node delivers to locally connected clients subscribed to that document

### Connection Lifecycle

```
[Connect] -> [Node assignment (load balancer)]
    -> [Subscribe to Redis channels for open documents]
    -> [Register in local connection map]

[Disconnect] -> [Remove from local connection map]
    -> [Unsubscribe from Redis channels]
    -> [Notify other clients via Pub/Sub]
```

### Reconnection Handling

- Client implements exponential backoff reconnection (1s, 2s, 4s, 8s, max 30s)
- On reconnect, client sends last received sequence number
- Server replays missed operations from the CRDT operation log
- Connection state is not stored server-side; recovery is client-driven

### Connection Limits

- Default: 10,000 concurrent WebSocket connections per node
- Tuned based on memory profiling (each connection ~10-50KB)
- Load balancer health check includes WebSocket connection count

## 7. Implementation Priority

| Phase | Scope | Duration |
|-------|-------|----------|
| 1 | Stateless servers: externalize sessions, cache, file storage to Redis/S3 | 1.5 weeks |
| 2 | Redis cluster: deployment, Sentinel HA, Pub/Sub for WebSocket routing | 1.5 weeks |
| 3 | Read replicas: PgBouncer setup, read/write routing, query optimization | 1.5 weeks |
| 4 | CDN: static asset delivery, edge caching, invalidation pipeline | 1.5 weeks |

**Total estimated effort: 6 weeks** (1 senior backend engineer + 1 DevOps engineer for infrastructure)
