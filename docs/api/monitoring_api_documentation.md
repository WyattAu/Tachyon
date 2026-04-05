# TACHYON: MONITORING API DOCUMENTATION

**Document ID:** TACHYON-API-014-V1.0
**Date:** February 2026
**Status:** Proposed
**Classification:** API Specification
**Compliance Level:** ISO/IEC 26514:2021, IEEE 1063-2001

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [Monitoring API Framework](#2-monitoring-api-framework)
3. [Health Check API](#3-health-check-api)
4. [Diagnostics API](#4-diagnostics-api)
5. [Log Aggregation API](#5-log-aggregation-api)
6. [Alerting API](#6-alerting-api)
7. [Metrics Collection API](#7-metrics-collection-api)
8. [Trace Collection API](#8-trace-collection-api)
9. [Incident Management API](#9-incident-management-api)
10. [Error Handling](#10-error-handling)
11. [References](#11-references)

---

## 1. INTRODUCTION

### 1.1. Document Purpose

This document provides comprehensive API specifications for the Tachyon Monitoring subsystem. The Monitoring API enables observability, health monitoring, diagnostics, log aggregation, alerting, metrics collection, distributed tracing, and incident management across all Tachyon components (Desktop, Server, and Web).

### 1.2. Scope

This document covers the following monitoring APIs:

- **Health Check API:** System health and readiness endpoints
- **Diagnostics API:** System diagnostics and performance profiling
- **Log Aggregation API:** Centralized log collection and querying
- **Alerting API:** Alert rule management and notification delivery
- **Metrics Collection API:** Application metrics and telemetry collection
- **Trace Collection API:** Distributed tracing and span collection
- **Incident Management API:** Incident lifecycle management and response

The Monitoring API is implemented in Rust using the Axum framework for HTTP/2 endpoints and Tokio for asynchronous operations, following the architectural decisions established in [ADR-001](../../.specs/02_adrs/001_rust_as_primary_language.md) and [ADR-010](../../.specs/02_adrs/010_security_architecture.md).

### 1.3. Document Dependencies

This document depends on the following documents:

- [TACHYON-STD-V1.0](../../.specs/01_standards/coding_standards.md) - Coding and Documentation Standards
- [TACHYON-REQ-SRV-V1.0](../../.specs/04_future_state/reqs/server_requirements.md) - Server Application Requirements
- [TACHYON-ADR-001-V1.0](../../.specs/02_adrs/001_rust_as_primary_language.md) - Rust as Primary Language
- [TACHYON-ADR-010-V1.0](../../.specs/02_adrs/010_security_architecture.md) - Security Architecture
- [TACHYON-TST-V1.0](../../.specs/04_future_state/test_plan.md) - Test Plan

### 1.4. Compliance Statement

This document complies with the following standards:

- **ISO/IEC 26514:2021:** Systems and Software Engineering—Requirements for Designers and Developers of User Documentation
- **IEEE 1063-2001:** Standard for Software User Documentation
- **ISO/IEC 25010:2011:** System and Software Quality Requirements and Evaluations

### 1.5. Terminology

The following terminology is used throughout this document:

| Term | Definition |
|-------|------------|
| **Observability** | The ability to infer the internal state of a system from its external outputs |
| **Telemetry** | Automated collection and transmission of data from remote or inaccessible points to a receiving system for monitoring |
| **Metric** | A quantitative measure of a system's behavior or performance |
| **Log** | A record of events that occurred within a system, typically with timestamps |
| **Trace** | A representation of a request's journey through a distributed system |
| **Span** | A named, timed operation representing a unit of work in a distributed trace |
| **Alert** | A notification triggered when a monitored condition meets specified criteria |
| **Incident** | An unplanned interruption to an IT service or reduction in the quality of an IT service |
| **SLA** | Service Level Agreement—a commitment between a service provider and a customer |
| **SLO** | Service Level Objective—a target value for a service level metric |

---

## 2. MONITORING API FRAMEWORK

### 2.1. Architecture Overview

The Tachyon Monitoring API framework provides a comprehensive observability solution integrated across all system components. The architecture follows a modular design with distinct subsystems for health monitoring, diagnostics, logging, alerting, metrics, tracing, and incident management.

**Architectural Principles:**

1. **Unified API Surface:** All monitoring endpoints follow consistent RESTful conventions
2. **Secure by Default:** All monitoring endpoints require authentication and authorization per [ADR-010](../../.specs/02_adrs/010_security_architecture.md)
3. **High Performance:** Async implementation using Tokio ensures minimal overhead on monitored systems
4. **Extensible:** Plugin-based architecture allows for custom collectors and exporters
5. **Observability as Code:** Alert rules and monitoring configurations are versioned in Git

### 2.2. Base URL Structure

The Monitoring API is exposed under the `/api/monitoring` base path:

```
https://tachyon.example.com/api/monitoring
```

All monitoring endpoints are prefixed with this base URL. For example:

- Health Check: `GET /api/monitoring/health`
- Metrics: `GET /api/monitoring/metrics`
- Logs: `POST /api/monitoring/logs/ingest`

### 2.3. Authentication and Authorization

All Monitoring API endpoints require authentication using JWT tokens obtained via the authentication endpoints specified in [REQ-SRV-036](../../.specs/04_future_state/reqs/server_requirements.md) through [REQ-SRV-040](../../.specs/04_future_state/reqs/server_requirements.md).

**Authorization Levels:**

| Role | Permissions |
|------|-------------|
| **Admin** | Full access to all monitoring endpoints, including incident management |
| **Operator** | Read access to health, metrics, logs, and alerts; write access to incident responses |
| **Viewer** | Read-only access to health, metrics, and logs |
| **System** | Service account with write access to metrics, logs, and traces |

Authentication is performed via the `Authorization` header:

```
Authorization: Bearer <jwt_token>
```

### 2.4. Request/Response Formats

All Monitoring API endpoints use JSON for request and response bodies. The following sections define common data structures used across multiple endpoints.

#### 2.4.1. Common Response Structure

All API responses follow a consistent structure:

```json
{
  "success": true,
  "data": { ... },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `success` | `boolean` | Indicates whether the request was successful |
| `data` | `object` | The response payload (present on success) |
| `error` | `object` | Error details (present on failure) |
| `timestamp` | `string` | ISO 8601 timestamp of the response |
| `request_id` | `string` | Unique identifier for request tracing |

#### 2.4.2. Error Response Structure

Error responses follow this structure:

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "MONITORING_001",
    "message": "Health check failed",
    "details": { ... }
  },
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Error Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `code` | `string` | Unique error code for programmatic handling |
| `message` | `string` | Human-readable error message |
| `details` | `object` | Additional error context (optional) |

### 2.5. Rate Limiting

Monitoring API endpoints implement rate limiting to prevent abuse and ensure system stability:

| Endpoint | Rate Limit | Burst |
|----------|-------------|-------|
| Health Check | 100 requests/minute | 10 |
| Diagnostics | 10 requests/minute | 5 |
| Log Ingestion | 1000 requests/minute | 100 |
| Alert Management | 50 requests/minute | 10 |
| Metrics Collection | 1000 requests/minute | 100 |
| Trace Collection | 1000 requests/minute | 100 |
| Incident Management | 20 requests/minute | 5 |

Rate limit headers are included in responses:

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1644264000
```

### 2.6. Versioning

The Monitoring API follows the versioning strategy defined in [TACHYON-API-V1.0](api_versioning.md). The current version is `v1`, specified in the URL path:

```
/api/monitoring/v1/health
```

Version negotiation is performed via the `Accept` header:

```
Accept: application/vnd.tachyon.monitoring.v1+json
```

### 2.7. Monitoring Data Retention

The following retention policies apply to monitoring data:

| Data Type | Retention Period | Rationale |
|------------|------------------|-----------|
| Health Check Results | 7 days | Short-term health monitoring |
| Diagnostic Snapshots | 30 days | Medium-term debugging support |
| Application Logs | 90 days | Compliance and operational needs |
| Alert Events | 365 days | Long-term trend analysis |
| Metrics | 90 days (raw), 365 days (aggregated) | Performance trend analysis |
| Traces | 7 days (detailed), 30 days (sampled) | Debugging and performance analysis |
| Incident Records | 7 years | Compliance and audit requirements |

Data older than the retention period is automatically purged according to the data lifecycle management policy.

---

## 3. HEALTH CHECK API

### 3.1. Overview

The Health Check API provides endpoints for monitoring system health, readiness, and liveness across all Tachyon components. These endpoints are designed for integration with load balancers, orchestrators (Kubernetes, Docker Swarm), and monitoring systems.

**Related Requirements:**
- [REQ-SRV-005](../../.specs/04_future_state/reqs/server_requirements.md): Health Check endpoint requirement

### 3.2. Endpoints

#### 3.2.1. Get System Health

Retrieves the current health status of the system.

**Endpoint:** `GET /api/monitoring/v1/health`

**Authentication:** Optional (public endpoint for load balancer health checks)

**Request Parameters:**

| Parameter | Type | Location | Required | Description |
|-----------|------|----------|-----------|-------------|
| `verbose` | `boolean` | Query | No | Include detailed component health information |
| `component` | `string` | Query | No | Filter health check to specific component |

**Example Request:**

```http
GET /api/monitoring/v1/health?verbose=true HTTP/2
Host: tachyon.example.com
Accept: application/vnd.tachyon.monitoring.v1+json
```

**Success Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "timestamp": "2026-02-07T21:51:48.971Z",
    "version": "1.0.0",
    "components": {
      "server": {
        "status": "healthy",
        "uptime_seconds": 86400,
        "last_check": "2026-02-07T21:51:48.971Z"
      },
      "database": {
        "status": "healthy",
        "connection_pool": {
          "active": 5,
          "idle": 10,
          "max": 50
        }
      },
      "search_index": {
        "status": "healthy",
        "document_count": 10000,
        "last_indexed": "2026-02-07T21:50:00.000Z"
      },
      "git_repository": {
        "status": "healthy",
        "branch": "main",
        "last_commit": "abc123def456"
      }
    }
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `status` | `string` | Overall system status: `healthy`, `degraded`, or `unhealthy` |
| `timestamp` | `string` | ISO 8601 timestamp of the health check |
| `version` | `string` | System version identifier |
| `components` | `object` | Health status of individual components |

**Component Status Values:**

| Status | Description | HTTP Status Code |
|--------|-------------|------------------|
| `healthy` | Component is operating normally | 200 |
| `degraded` | Component is operating with reduced functionality | 200 (overall) |
| `unhealthy` | Component is not operational | 503 |

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 503 Service Unavailable | `MONITORING_001` | System is currently unavailable |
| 500 Internal Server Error | `MONITORING_002` | Internal error during health check |

#### 3.2.2. Get Readiness

Retrieves the readiness status of the system for accepting traffic.

**Endpoint:** `GET /api/monitoring/v1/ready`

**Authentication:** Optional (public endpoint)

**Request Parameters:** None

**Example Request:**

```http
GET /api/monitoring/v1/ready HTTP/2
Host: tachyon.example.com
Accept: application/vnd.tachyon.monitoring.v1+json
```

**Success Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "ready": true,
    "timestamp": "2026-02-07T21:51:48.971Z",
    "checks": {
      "database": true,
      "search_index": true,
      "git_repository": true,
      "configuration_loaded": true
    }
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `ready` | `boolean` | Overall readiness status |
| `timestamp` | `string` | ISO 8601 timestamp of the readiness check |
| `checks` | `object` | Individual readiness check results |

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 503 Service Unavailable | `MONITORING_003` | System is not ready to accept traffic |

#### 3.2.3. Get Liveness

Retrieves the liveness status of the system for Kubernetes liveness probes.

**Endpoint:** `GET /api/monitoring/v1/live`

**Authentication:** Optional (public endpoint)

**Request Parameters:** None

**Example Request:**

```http
GET /api/monitoring/v1/live HTTP/2
Host: tachyon.example.com
Accept: application/vnd.tachyon.monitoring.v1+json
```

**Success Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "alive": true,
    "timestamp": "2026-02-07T21:51:48.971Z"
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `alive` | `boolean` | Liveness status |
| `timestamp` | `string` | ISO 8601 timestamp of the liveness check |

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 503 Service Unavailable | `MONITORING_004` | System is not alive |

#### 3.2.4. Get Component Health

Retrieves detailed health information for a specific component.

**Endpoint:** `GET /api/monitoring/v1/health/components/{component_id}`

**Authentication:** Required (Admin or Operator role)

**Path Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|-----------|-------------|
| `component_id` | `string` | Yes | Component identifier (e.g., `server`, `database`, `search_index`) |

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|-----------|-------------|
| `include_metrics` | `boolean` | No | Include component-specific metrics |

**Example Request:**

```http
GET /api/monitoring/v1/health/components/database?include_metrics=true HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Accept: application/vnd.tachyon.monitoring.v1+json
```

**Success Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "component_id": "database",
    "status": "healthy",
    "uptime_seconds": 86400,
    "last_check": "2026-02-07T21:51:48.971Z",
    "metrics": {
      "connection_pool": {
        "active": 5,
        "idle": 10,
        "max": 50,
        "utilization_percent": 30.0
      },
      "query_performance": {
        "avg_latency_ms": 10.5,
        "p95_latency_ms": 25.0,
        "p99_latency_ms": 50.0
      }
    }
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 404 Not Found | `MONITORING_005` | Component not found |
| 403 Forbidden | `MONITORING_006` | Insufficient permissions |

### 3.3. Health Check Configuration

Health check behavior can be configured via the monitoring configuration file:

```toml
[monitoring.health_check]
# Enable detailed health information
verbose = true

# Health check intervals
check_interval_seconds = 30

# Component health thresholds
[monitoring.health_check.thresholds]
database.connection_pool.utilization_percent = 80.0
search_index.document_count.min = 0
git_repository.last_commit.max_age_hours = 24

# Component failure detection
[monitoring.health_check.failure_detection]
consecutive_failures = 3
recovery_check_interval_seconds = 60
```

### 3.4. Integration with Orchestrators

The Health Check API is designed for integration with container orchestrators:

**Kubernetes Probes:**

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: tachyon-server
spec:
  containers:
  - name: server
    image: tachyon/server:1.0.0
    livenessProbe:
      httpGet:
        path: /api/monitoring/v1/live
        port: 8080
      initialDelaySeconds: 30
      periodSeconds: 10
    readinessProbe:
      httpGet:
        path: /api/monitoring/v1/ready
        port: 8080
      initialDelaySeconds: 5
      periodSeconds: 5
    startupProbe:
      httpGet:
        path: /api/monitoring/v1/health
        port: 8080
      initialDelaySeconds: 0
      periodSeconds: 5
      failureThreshold: 30
```

**Docker Health Check:**

```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/api/monitoring/v1/health || exit 1
```

**Load Balancer Health Check:**

```nginx
upstream tachyon_servers {
    server server1:8080 max_fails=3 fail_timeout=30s;
    server server2:8080 max_fails=3 fail_timeout=30s;
    
    check interval=3000 rise=2 fall=3 timeout=1000 type=http
        check_http_send "GET /api/monitoring/v1/health HTTP/1.1\r\nHost: tachyon.example.com\r\n\r\n"
        check_http_expect_alive http_2xx http_3xx;
}

---

## 4. DIAGNOSTICS API

### 4.1. Overview

The Diagnostics API provides endpoints for system diagnostics, performance profiling, and debugging information. These endpoints enable operators and administrators to investigate system behavior, identify performance bottlenecks, and troubleshoot issues.

**Related Requirements:**
- [REQ-SRV-106](../../.specs/04_future_state/reqs/server_requirements.md) through [REQ-SRV-120](../../.specs/04_future_state/reqs/server_requirements.md): Performance and resource management requirements

### 4.2. Endpoints

#### 4.2.1. Get System Diagnostics

Retrieves comprehensive system diagnostics including resource utilization, performance metrics, and configuration status.

**Endpoint:** `GET /api/monitoring/v1/diagnostics`

**Authentication:** Required (Admin role)

**Request Parameters:**

| Parameter | Type | Location | Required | Description |
|-----------|------|----------|-----------|-------------|
| `include_threads` | `boolean` | Query | No | Include thread dump information |
| `include_memory` | `boolean` | Query | No | Include detailed memory information |
| `include_config` | `boolean` | Query | No | Include configuration snapshot |

**Example Request:**

```http
GET /api/monitoring/v1/diagnostics?include_threads=true&include_memory=true HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Accept: application/vnd.tachyon.monitoring.v1+json
```

**Success Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "timestamp": "2026-02-07T21:51:48.971Z",
    "system": {
      "hostname": "tachyon-server-01",
      "os": "Linux",
      "os_version": "6.18.0",
      "architecture": "x86_64",
      "uptime_seconds": 86400,
      "cpu_count": 8,
      "total_memory_bytes": 17179869184,
      "available_memory_bytes": 8589934592
    },
    "process": {
      "pid": 12345,
      "parent_pid": 1,
      "thread_count": 32,
      "memory_usage_bytes": 536870912,
      "cpu_usage_percent": 15.5,
      "file_descriptors": 512,
      "max_file_descriptors": 65536
    },
    "runtime": {
      "tokio_version": "1.0.0",
      "worker_threads": 8,
      "blocking_threads": 4,
      "async_tasks": {
        "active": 45,
        "pending": 12,
        "completed": 1000000
      }
    },
    "memory": {
      "heap_allocated_bytes": 536870912,
      "heap_used_bytes": 32212254720,
      "stack_allocated_bytes": 8388608,
      "stack_used_bytes": 4194304
    },
    "threads": [
      {
        "id": 1,
        "name": "tokio-runtime-worker",
        "state": "running",
        "cpu_time_ns": 1234567890,
        "stack_size_bytes": 8388608
      }
    ],
    "configuration": {
      "server": {
        "host": "0.0.0.0",
        "port": 8080,
        "max_connections": 1000
      },
      "database": {
        "path": "/var/lib/tachyon/db.sqlite",
        "connection_pool_size": 50
      }
    }
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `system` | `object` | System-level information |
| `process` | `object` | Process-level information |
| `runtime` | `object` | Tokio runtime information |
| `memory` | `object` | Memory allocation information |
| `threads` | `array` | Thread information (if requested) |
| `configuration` | `object` | Configuration snapshot (if requested) |

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 403 Forbidden | `MONITORING_007` | Insufficient permissions |
| 500 Internal Server Error | `MONITORING_008` | Error collecting diagnostics |

#### 4.2.2. Get Performance Profile

Retrieves performance profiling data for analysis.

**Endpoint:** `GET /api/monitoring/v1/diagnostics/profile`

**Authentication:** Required (Admin role)

**Request Parameters:**

| Parameter | Type | Location | Required | Description |
|-----------|------|----------|-----------|-------------|
| `duration_seconds` | `integer` | Query | No | Profile duration (default: 10, max: 60) |
| `frequency_hz` | `integer` | Query | No | Sampling frequency in Hz (default: 100) |
| `type` | `string` | Query | No | Profile type: `cpu`, `memory`, `io` (default: `cpu`) |

**Example Request:**

```http
GET /api/monitoring/v1/diagnostics/profile?duration_seconds=30&frequency_hz=100&type=cpu HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Accept: application/vnd.tachyon.monitoring.v1+json
```

**Success Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "profile_id": "prof_abc123def456",
    "timestamp": "2026-02-07T21:51:48.971Z",
    "duration_seconds": 30,
    "frequency_hz": 100,
    "type": "cpu",
    "samples": [
      {
        "timestamp": "2026-02-07T21:51:48.971Z",
        "thread_id": 1,
        "function": "tokio_runtime_worker",
        "cpu_time_ns": 1234567,
        "stack_trace": [
          "tokio::runtime::worker::run",
          "tachyon::server::handle_request"
        ]
      }
    ],
    "summary": {
      "total_samples": 3000,
      "top_functions": [
        {
          "function": "tachyon::server::handle_request",
          "samples": 1500,
          "percentage": 50.0
        }
      ]
    }
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `profile_id` | `string` | Unique identifier for the profile |
| `samples` | `array` | Individual profiling samples |
| `summary` | `object` | Aggregated profile summary |

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 403 Forbidden | `MONITORING_009` | Insufficient permissions |
| 400 Bad Request | `MONITORING_010` | Invalid profile parameters |
| 500 Internal Server Error | `MONITORING_011` | Error collecting profile |

#### 4.2.3. Get Thread Dump

Retrieves a snapshot of all active threads and their stack traces.

**Endpoint:** `GET /api/monitoring/v1/diagnostics/threads`

**Authentication:** Required (Admin role)

**Request Parameters:**

| Parameter | Type | Location | Required | Description |
|-----------|------|----------|-----------|-------------|
| `include_stack` | `boolean` | Query | No | Include stack traces for each thread |

**Example Request:**

```http
GET /api/monitoring/v1/diagnostics/threads?include_stack=true HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Accept: application/vnd.tachyon.monitoring.v1+json
```

**Success Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "timestamp": "2026-02-07T21:51:48.971Z",
    "thread_count": 32,
    "threads": [
      {
        "id": 1,
        "name": "tokio-runtime-worker-0",
        "state": "running",
        "priority": 0,
        "cpu_time_ns": 1234567890,
        "stack_trace": [
          "tokio::runtime::worker::run",
          "tachyon::server::handle_request",
          "tachyon::api::documents::get_document"
        ]
      }
    ]
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 403 Forbidden | `MONITORING_012` | Insufficient permissions |
| 500 Internal Server Error | `MONITORING_013` | Error collecting thread dump |

#### 4.2.4. Get Memory Statistics

Retrieves detailed memory allocation statistics.

**Endpoint:** `GET /api/monitoring/v1/diagnostics/memory`

**Authentication:** Required (Admin role)

**Request Parameters:** None

**Example Request:**

```http
GET /api/monitoring/v1/diagnostics/memory HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Accept: application/vnd.tachyon.monitoring.v1+json
```

**Success Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "timestamp": "2026-02-07T21:51:48.971Z",
    "heap": {
      "allocated_bytes": 536870912,
      "used_bytes": 32212254720,
      "free_bytes": 21458616192,
      "fragmentation_percent": 5.2
    },
    "stack": {
      "allocated_bytes": 8388608,
      "used_bytes": 4194304,
      "thread_count": 32
    },
    "global_allocator": {
      "type": "jemalloc",
      "total_allocated_bytes": 54509569520,
      "total_freed_bytes": 82692692
    },
    "gc": {
      "enabled": false,
      "last_collection": null
    }
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 403 Forbidden | `MONITORING_014` | Insufficient permissions |
| 500 Internal Server Error | `MONITORING_015` | Error collecting memory statistics |

### 4.3. Diagnostics Configuration

Diagnostics behavior can be configured via the monitoring configuration file:

```toml
[monitoring.diagnostics]
# Enable diagnostics endpoints
enabled = true

# Profile collection limits
[monitoring.diagnostics.profiles]
max_duration_seconds = 60
max_frequency_hz = 1000
concurrent_profiles = 1

# Thread dump configuration
[monitoring.diagnostics.threads]
max_stack_depth = 64
include_idle_threads = false

# Memory statistics collection
[monitoring.diagnostics.memory]
collection_interval_seconds = 60
detailed_heap_stats = true
```

---

## 5. LOG AGGREGATION API

### 5.1. Overview

The Log Aggregation API provides endpoints for centralized log collection, querying, and management. This API enables structured logging across all Tachyon components with support for log levels, structured fields, and powerful querying capabilities.

**Related Requirements:**
- [REQ-SRV-046](../../.specs/04_future_state/reqs/server_requirements.md) through [REQ-SRV-060](../../.specs/04_future_state/reqs/server_requirements.md): Data processing and storage requirements

### 5.2. Log Levels

The following log levels are supported, following standard conventions:

| Level | Value | Description |
|-------|-------|-------------|
| `TRACE` | 0 | Very detailed information, typically only enabled during debugging |
| `DEBUG` | 10 | Detailed diagnostic information |
| `INFO` | 20 | General informational messages |
| `WARN` | 30 | Warning conditions that may indicate problems |
| `ERROR` | 40 | Error conditions that should be investigated |
| `FATAL` | 50 | Critical errors that may cause service failure |

### 5.3. Endpoints

#### 5.3.1. Ingest Logs

Submits log entries for aggregation and storage.

**Endpoint:** `POST /api/monitoring/v1/logs/ingest`

**Authentication:** Required (System or Admin role)

**Request Body:**

```json
{
  "logs": [
    {
      "timestamp": "2026-02-07T21:51:48.971Z",
      "level": "INFO",
      "component": "server",
      "message": "Document retrieved successfully",
      "fields": {
        "document_id": "doc_abc123",
        "user_id": "user_456",
        "latency_ms": 15.5
      },
      "context": {
        "request_id": "req_xyz789",
        "trace_id": "trace_abc123"
      }
    }
  ]
}
```

**Request Fields:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `logs` | `array` | Yes | Array of log entries (max 100 per request) |

**Log Entry Fields:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `timestamp` | `string` | Yes | ISO 8601 timestamp |
| `level` | `string` | Yes | Log level (TRACE, DEBUG, INFO, WARN, ERROR, FATAL) |
| `component` | `string` | Yes | Component identifier (e.g., server, desktop, web) |
| `message` | `string` | Yes | Log message |
| `fields` | `object` | No | Structured log fields |
| `context` | `object` | No | Request/trace context |

**Example Request:**

```http
POST /api/monitoring/v1/logs/ingest HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Content-Type: application/vnd.tachyon.monitoring.v1+json

{
  "logs": [
    {
      "timestamp": "2026-02-07T21:51:48.971Z",
      "level": "INFO",
      "component": "server",
      "message": "Document retrieved successfully",
      "fields": {
        "document_id": "doc_abc123",
        "user_id": "user_456",
        "latency_ms": 15.5
      }
    }
  ]
}
```

**Success Response (201 Created):**

```json
{
  "success": true,
  "data": {
    "ingested_count": 1,
    "batch_id": "batch_abc123def456",
    "timestamp": "2026-02-07T21:51:48.971Z"
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `ingested_count` | `integer` | Number of logs successfully ingested |
| `batch_id` | `string` | Unique identifier for the log batch |

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 400 Bad Request | `MONITORING_016` | Invalid log format |
| 403 Forbidden | `MONITORING_017` | Insufficient permissions |
| 413 Payload Too Large | `MONITORING_018` | Request body exceeds size limit |
| 500 Internal Server Error | `MONITORING_019` | Error ingesting logs |

#### 5.3.2. Query Logs

Retrieves log entries based on query criteria.

**Endpoint:** `GET /api/monitoring/v1/logs/query`

**Authentication:** Required (Admin, Operator, or Viewer role)

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|-----------|-------------|
| `level` | `string` | No | Filter by log level |
| `component` | `string` | No | Filter by component |
| `start_time` | `string` | No | Start of time range (ISO 8601) |
| `end_time` | `string` | No | End of time range (ISO 8601) |
| `query` | `string` | No | Full-text search query |
| `field_filter` | `string` | No | Field filter (e.g., `user_id=user_456`) |
| `limit` | `integer` | No | Maximum results (default: 100, max: 1000) |
| `offset` | `integer` | No | Result offset for pagination |
| `sort` | `string` | No | Sort field (default: timestamp) |
| `order` | `string` | No | Sort order: `asc` or `desc` (default: `desc`) |

**Example Request:**

```http
GET /api/monitoring/v1/logs/query?level=ERROR&start_time=2026-02-07T00:00:00.000Z&end_time=2026-02-07T23:59:59.999Z&limit=50 HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Accept: application/vnd.tachyon.monitoring.v1+json
```

**Success Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "logs": [
      {
        "id": "log_abc123def456",
        "timestamp": "2026-02-07T21:51:48.971Z",
        "level": "ERROR",
        "component": "server",
        "message": "Database connection failed",
        "fields": {
          "error_code": "DB_CONN_001",
          "retry_count": 3
        },
        "context": {
          "request_id": "req_xyz789"
        }
      }
    ],
    "total_count": 125,
    "page_info": {
      "limit": 50,
      "offset": 0,
      "has_more": true
    }
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `logs` | `array` | Array of log entries |
| `total_count` | `integer` | Total number of matching logs |
| `page_info` | `object` | Pagination information |

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 400 Bad Request | `MONITORING_020` | Invalid query parameters |
| 403 Forbidden | `MONITORING_021` | Insufficient permissions |

#### 5.3.3. Get Log Statistics

Retrieves aggregated statistics about log entries.

**Endpoint:** `GET /api/monitoring/v1/logs/stats`

**Authentication:** Required (Admin, Operator, or Viewer role)

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|-----------|-------------|
| `start_time` | `string` | No | Start of time range (ISO 8601) |
| `end_time` | `string` | No | End of time range (ISO 8601) |
| `group_by` | `string` | No | Grouping field: `level`, `component`, `hour` |
| `interval` | `string` | No | Time interval for time-series: `hour`, `day`, `week` |

**Example Request:**

```http
GET /api/monitoring/v1/logs/stats?group_by=level&interval=hour HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Accept: application/vnd.tachyon.monitoring.v1+json
```

**Success Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "time_range": {
      "start": "2026-02-07T00:00:00.000Z",
      "end": "2026-02-07T23:59:59.999Z"
    },
    "by_level": {
      "TRACE": 1000,
      "DEBUG": 5000,
      "INFO": 25000,
      "WARN": 500,
      "ERROR": 100,
      "FATAL": 5
    },
    "by_component": {
      "server": 28000,
      "desktop": 2500,
      "web": 2105
    },
    "by_hour": [
      {
        "hour": "2026-02-07T21:00:00.000Z",
        "counts": {
          "INFO": 1000,
          "ERROR": 5
        }
      }
    ]
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 400 Bad Request | `MONITORING_022` | Invalid query parameters |
| 403 Forbidden | `MONITORING_023` | Insufficient permissions |

#### 5.3.4. Export Logs

Exports log entries in various formats.

**Endpoint:** `GET /api/monitoring/v1/logs/export`

**Authentication:** Required (Admin or Operator role)

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|-----------|-------------|
| `format` | `string` | No | Export format: `json`, `csv`, `ndjson` (default: `json`) |
| `query` | `string` | No | Query filter (same as query endpoint) |

**Example Request:**

```http
GET /api/monitoring/v1/logs/export?format=csv&level=ERROR HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Accept: text/csv
```

**Success Response (200 OK):**

Returns log entries in the specified format. For CSV:

```csv
id,timestamp,level,component,message,fields,context
log_abc123def456,2026-02-07T21:51:48.971Z,ERROR,server,"Database connection failed","{""error_code"":""DB_CONN_001"",""retry_count"":3}","{""request_id"":""req_xyz789""}"
```

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 400 Bad Request | `MONITORING_024` | Invalid export parameters |
| 403 Forbidden | `MONITORING_025` | Insufficient permissions |

### 5.4. Log Aggregation Configuration

Log aggregation behavior can be configured via the monitoring configuration file:

```toml
[monitoring.logs]
# Log ingestion settings
[monitoring.logs.ingestion]
max_batch_size = 100
max_request_size_bytes = 10485760
async_processing = true

# Log retention
[monitoring.logs.retention]
default_retention_days = 90
level_retention = { TRACE = 1, DEBUG = 7, INFO = 90, WARN = 365, ERROR = 365, FATAL = 2555 }

# Log indexing
[monitoring.logs.indexing]
enabled = true
index_fields = ["level", "component", "user_id", "document_id"]
full_text_search = true
```

---

## 6. ALERTING API

### 6.1. Overview

The Alerting API provides endpoints for managing alert rules, alert notifications, and alert history. This API enables proactive monitoring through configurable alert conditions that trigger notifications when thresholds are exceeded or anomalies are detected.

**Related Requirements:**
- [REQ-SRV-046](../../.specs/04_future_state/reqs/server_requirements.md) through [REQ-SRV-060](../../.specs/04_future_state/reqs/server_requirements.md): Data processing and storage requirements

### 6.2. Alert Rule Types

The following alert rule types are supported:

| Type | Description | Example Condition |
|------|-------------|-------------------|
| `threshold` | Alert when metric exceeds threshold | `cpu_usage_percent > 80` |
| `rate` | Alert when rate exceeds threshold | `error_rate_per_minute > 10` |
| `pattern` | Alert when log pattern matches | `message contains "connection failed"` |
| `anomaly` | Alert when anomaly is detected | `latency_anomaly_detected` |
| `composite` | Alert when multiple conditions are met | `cpu_usage_percent > 80 AND memory_usage_percent > 70` |

### 6.3. Alert Severity Levels

| Level | Value | Description |
|-------|-------|-------------|
| `INFO` | 0 | Informational alert |
| `WARNING` | 1 | Warning condition requiring attention |
| `ERROR` | 2 | Error condition requiring investigation |
| `CRITICAL` | 3 | Critical condition requiring immediate action |

### 6.4. Endpoints

#### 6.4.1. Create Alert Rule

Creates a new alert rule.

**Endpoint:** `POST /api/monitoring/v1/alerts/rules`

**Authentication:** Required (Admin role)

**Request Body:**

```json
{
  "name": "High CPU Usage",
  "description": "Alert when CPU usage exceeds 80%",
  "type": "threshold",
  "enabled": true,
  "severity": "WARNING",
  "condition": {
    "metric": "cpu_usage_percent",
    "operator": ">",
    "threshold": 80.0,
    "duration_seconds": 300
  },
  "notification": {
    "channels": ["email", "slack"],
    "recipients": ["ops@tachyon.example.com"],
    "template": "cpu_high_usage"
  },
  "cooldown": {
    "duration_seconds": 1800
  }
}
```

**Request Fields:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `name` | `string` | Yes | Unique name for the alert rule |
| `description` | `string` | No | Human-readable description |
| `type` | `string` | Yes | Alert rule type |
| `enabled` | `boolean` | No | Whether the rule is enabled |
| `severity` | `string` | No | Alert severity level |
| `condition` | `object` | Yes | Alert condition specification |
| `notification` | `object` | Yes | Notification configuration |
| `cooldown` | `object` | No | Cooldown configuration |

**Condition Fields:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `metric` | `string` | Yes | Metric name to evaluate |
| `operator` | `string` | Yes | Comparison operator: `>`, `<`, `>=`, `<=`, `==`, `!=` |
| `threshold` | `number` | Yes | Threshold value |
| `duration_seconds` | `integer` | No | Duration condition must be met |

**Notification Channels:**

| Channel | Description |
|---------|-------------|
| `email` | Email notification |
| `slack` | Slack webhook notification |
| `webhook` | Generic webhook notification |
| `pagerduty` | PagerDuty integration |
| `sms` | SMS notification |

**Example Request:**

```http
POST /api/monitoring/v1/alerts/rules HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Content-Type: application/vnd.tachyon.monitoring.v1+json

{
  "name": "High CPU Usage",
  "description": "Alert when CPU usage exceeds 80%",
  "type": "threshold",
  "enabled": true,
  "severity": "WARNING",
  "condition": {
    "metric": "cpu_usage_percent",
    "operator": ">",
    "threshold": 80.0,
    "duration_seconds": 300
  },
  "notification": {
    "channels": ["email"],
    "recipients": ["ops@tachyon.example.com"]
  }
}
```

**Success Response (201 Created):**

```json
{
  "success": true,
  "data": {
    "rule_id": "rule_abc123def456",
    "name": "High CPU Usage",
    "description": "Alert when CPU usage exceeds 80%",
    "type": "threshold",
    "enabled": true,
    "severity": "WARNING",
    "created_at": "2026-02-07T21:51:48.971Z",
    "updated_at": "2026-02-07T21:51:48.971Z"
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 400 Bad Request | `MONITORING_026` | Invalid alert rule configuration |
| 409 Conflict | `MONITORING_027` | Alert rule name already exists |
| 403 Forbidden | `MONITORING_028` | Insufficient permissions |

#### 6.4.2. List Alert Rules

Retrieves all alert rules.

**Endpoint:** `GET /api/monitoring/v1/alerts/rules`

**Authentication:** Required (Admin, Operator, or Viewer role)

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|-----------|-------------|
| `enabled` | `boolean` | No | Filter by enabled status |
| `type` | `string` | No | Filter by rule type |
| `severity` | `string` | No | Filter by severity level |

**Example Request:**

```http
GET /api/monitoring/v1/alerts/rules?enabled=true HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Accept: application/vnd.tachyon.monitoring.v1+json
```

**Success Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "rules": [
      {
        "rule_id": "rule_abc123def456",
        "name": "High CPU Usage",
        "description": "Alert when CPU usage exceeds 80%",
        "type": "threshold",
        "enabled": true,
        "severity": "WARNING",
        "created_at": "2026-02-07T21:51:48.971Z",
        "updated_at": "2026-02-07T21:51:48.971Z"
      }
    ],
    "total_count": 1
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 403 Forbidden | `MONITORING_029` | Insufficient permissions |

#### 6.4.3. Get Alert Rule

Retrieves a specific alert rule.

**Endpoint:** `GET /api/monitoring/v1/alerts/rules/{rule_id}`

**Authentication:** Required (Admin, Operator, or Viewer role)

**Path Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|-----------|-------------|
| `rule_id` | `string` | Yes | Alert rule identifier |

**Example Request:**

```http
GET /api/monitoring/v1/alerts/rules/rule_abc123def456 HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Accept: application/vnd.tachyon.monitoring.v1+json
```

**Success Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "rule_id": "rule_abc123def456",
    "name": "High CPU Usage",
    "description": "Alert when CPU usage exceeds 80%",
    "type": "threshold",
    "enabled": true,
    "severity": "WARNING",
    "condition": {
      "metric": "cpu_usage_percent",
      "operator": ">",
      "threshold": 80.0,
      "duration_seconds": 300
    },
    "notification": {
      "channels": ["email"],
      "recipients": ["ops@tachyon.example.com"]
    },
    "cooldown": {
      "duration_seconds": 1800
    },
    "created_at": "2026-02-07T21:51:48.971Z",
    "updated_at": "2026-02-07T21:51:48.971Z"
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 404 Not Found | `MONITORING_030` | Alert rule not found |
| 403 Forbidden | `MONITORING_031` | Insufficient permissions |

#### 6.4.4. Update Alert Rule

Updates an existing alert rule.

**Endpoint:** `PUT /api/monitoring/v1/alerts/rules/{rule_id}`

**Authentication:** Required (Admin role)

**Path Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|-----------|-------------|
| `rule_id` | `string` | Yes | Alert rule identifier |

**Request Body:** Same as create alert rule

**Success Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "rule_id": "rule_abc123def456",
    "name": "High CPU Usage",
    "description": "Alert when CPU usage exceeds 85%",
    "type": "threshold",
    "enabled": true,
    "severity": "WARNING",
    "updated_at": "2026-02-07T21:52:48.971Z"
  },
  "error": null,
  "timestamp": "2026-02-07T21:52:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 400 Bad Request | `MONITORING_032` | Invalid alert rule configuration |
| 404 Not Found | `MONITORING_033` | Alert rule not found |
| 403 Forbidden | `MONITORING_034` | Insufficient permissions |

#### 6.4.5. Delete Alert Rule

Deletes an alert rule.

**Endpoint:** `DELETE /api/monitoring/v1/alerts/rules/{rule_id}`

**Authentication:** Required (Admin role)

**Path Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|-----------|-------------|
| `rule_id` | `string` | Yes | Alert rule identifier |

**Example Request:**

```http
DELETE /api/monitoring/v1/alerts/rules/rule_abc123def456 HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
```

**Success Response (204 No Content):**

No response body.

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 404 Not Found | `MONITORING_035` | Alert rule not found |
| 403 Forbidden | `MONITORING_036` | Insufficient permissions |

#### 6.4.6. List Alert Events

Retrieves alert events (triggered alerts).

**Endpoint:** `GET /api/monitoring/v1/alerts/events`

**Authentication:** Required (Admin, Operator, or Viewer role)

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|-----------|-------------|
| `rule_id` | `string` | No | Filter by alert rule |
| `severity` | `string` | No | Filter by severity level |
| `start_time` | `string` | No | Start of time range (ISO 8601) |
| `end_time` | `string` | No | End of time range (ISO 8601) |
| `limit` | `integer` | No | Maximum results (default: 100, max: 1000) |
| `offset` | `integer` | No | Result offset for pagination |

**Example Request:**

```http
GET /api/monitoring/v1/alerts/events?severity=CRITICAL&limit=50 HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Accept: application/vnd.tachyon.monitoring.v1+json
```

**Success Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "events": [
      {
        "event_id": "evt_abc123def456",
        "rule_id": "rule_abc123def456",
        "rule_name": "High CPU Usage",
        "severity": "WARNING",
        "triggered_at": "2026-02-07T21:51:48.971Z",
        "resolved_at": null,
        "condition": {
          "metric": "cpu_usage_percent",
          "value": 85.5,
          "threshold": 80.0
        },
        "context": {
          "hostname": "tachyon-server-01",
          "component": "server"
        }
      }
    ],
    "total_count": 25,
    "page_info": {
      "limit": 50,
      "offset": 0,
      "has_more": false
    }
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 400 Bad Request | `MONITORING_037` | Invalid query parameters |
| 403 Forbidden | `MONITORING_038` | Insufficient permissions |

### 6.5. Alerting Configuration

Alerting behavior can be configured via the monitoring configuration file:

```toml
[monitoring.alerts]
# Alert evaluation settings
[monitoring.alerts.evaluation]
interval_seconds = 60
max_concurrent_evaluations = 10

# Notification settings
[monitoring.alerts.notification]
retry_attempts = 3
retry_delay_seconds = 60
max_queue_size = 1000

# Alert retention
[monitoring.alerts.retention]
event_retention_days = 365
resolved_event_retention_days = 90
```

---

## 7. METRICS COLLECTION API

### 7.1. Overview

The Metrics Collection API provides endpoints for submitting application metrics and retrieving aggregated metric data. This API supports Prometheus-compatible metric formats and enables integration with monitoring systems like Grafana, Prometheus, and Datadog.

**Related Requirements:**
- [REQ-SRV-106](../../.specs/04_future_state/reqs/server_requirements.md) through [REQ-SRV-120](../../.specs/04_future_state/reqs/server_requirements.md): Performance and resource management requirements

### 7.2. Metric Types

The following metric types are supported:

| Type | Description | Example |
|------|-------------|---------|
| `counter` | Monotonically increasing value | `http_requests_total` |
| `gauge` | Value that can go up or down | `cpu_usage_percent` |
| `histogram` | Distribution of values | `request_duration_seconds` |
| `summary` | Count and sum of values | `request_size_bytes` |

### 7.3. Endpoints

#### 7.3.1. Ingest Metrics

Submits metric samples for aggregation and storage.

**Endpoint:** `POST /api/monitoring/v1/metrics/ingest`

**Authentication:** Required (System or Admin role)

**Request Body (Prometheus Exposition Format):**

```
# HELP tachyon_cpu_usage_seconds_total
# TYPE counter
tachyon_cpu_usage_seconds_total 12345.67

# HELP tachyon_memory_usage_bytes
# TYPE gauge
tachyon_memory_usage_bytes 536870912

# HELP tachyon_http_requests_duration_seconds
# TYPE histogram
tachyon_http_requests_duration_seconds_bucket{le="0.005"} 1000
tachyon_http_requests_duration_seconds_bucket{le="0.01"} 2500
tachyon_http_requests_duration_seconds_bucket{le="0.025"} 5000
tachyon_http_requests_duration_seconds_bucket{le="0.05"} 10000
tachyon_http_requests_duration_seconds_bucket{le="0.1"} 5000
tachyon_http_requests_duration_seconds_bucket{le="0.25"} 2000
tachyon_http_requests_duration_seconds_bucket{le="0.5"} 1000
tachyon_http_requests_duration_seconds_bucket{le="1"} 500
tachyon_http_requests_duration_seconds_bucket{le="2.5"} 200
tachyon_http_requests_duration_seconds_bucket{le="5"} 100
tachyon_http_requests_duration_seconds_bucket{le="10"} 50
tachyon_http_requests_duration_seconds_bucket{le="+Inf"} 25
tachyon_http_requests_duration_seconds_sum 1234.56
tachyon_http_requests_duration_seconds_count 26580
```

**Request Body (JSON Format):**

```json
{
  "metrics": [
    {
      "name": "tachyon_cpu_usage_seconds_total",
      "type": "counter",
      "value": 12345.67,
      "timestamp": "2026-02-07T21:51:48.971Z",
      "labels": {
        "host": "tachyon-server-01",
        "component": "server"
      }
    },
    {
      "name": "tachyon_memory_usage_bytes",
      "type": "gauge",
      "value": 536870912,
      "timestamp": "2026-02-07T21:51:48.971Z",
      "labels": {
        "host": "tachyon-server-01",
        "component": "server"
      }
    },
    {
      "name": "tachyon_http_requests_duration_seconds",
      "type": "histogram",
      "samples": [
        {
          "value": 0.015,
          "timestamp": "2026-02-07T21:51:48.971Z",
          "labels": {
            "endpoint": "/api/documents",
            "method": "GET"
          }
        }
      ],
      "summary": {
        "count": 26580,
        "sum": 1234.56,
        "buckets": [
          {"le": 0.005, "count": 1000},
          {"le": 0.01, "count": 2500},
          {"le": 0.025, "count": 5000}
        ]
      }
    }
  ]
}
```

**Metric Fields:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `name` | `string` | Yes | Metric name |
| `type` | `string` | Yes | Metric type (counter, gauge, histogram, summary) |
| `value` | `number` | Yes | Metric value (for counter and gauge) |
| `timestamp` | `string` | No | ISO 8601 timestamp |
| `labels` | `object` | No | Metric labels/dimensions |
| `samples` | `array` | No | Histogram samples |
| `summary` | `object` | No | Histogram/summary summary |

**Example Request (JSON):**

```http
POST /api/monitoring/v1/metrics/ingest HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Content-Type: application/vnd.tachyon.monitoring.v1+json

{
  "metrics": [
    {
      "name": "tachyon_cpu_usage_seconds_total",
      "type": "counter",
      "value": 12345.67,
      "timestamp": "2026-02-07T21:51:48.971Z",
      "labels": {
        "host": "tachyon-server-01",
        "component": "server"
      }
    }
  ]
}
```

**Success Response (201 Created):**

```json
{
  "success": true,
  "data": {
    "ingested_count": 1,
    "batch_id": "batch_abc123def456",
    "timestamp": "2026-02-07T21:51:48.971Z"
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 400 Bad Request | `MONITORING_039` | Invalid metric format |
| 403 Forbidden | `MONITORING_040` | Insufficient permissions |
| 413 Payload Too Large | `MONITORING_041` | Request body exceeds size limit |
| 500 Internal Server Error | `MONITORING_042` | Error ingesting metrics |

#### 7.3.2. Query Metrics

Retrieves metric data for analysis.

**Endpoint:** `GET /api/monitoring/v1/metrics/query`

**Authentication:** Required (Admin, Operator, or Viewer role)

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|-----------|-------------|
| `name` | `string` | No | Filter by metric name (supports wildcards) |
| `type` | `string` | No | Filter by metric type |
| `labels` | `string` | No | Label filter (e.g., `component=server`) |
| `start_time` | `string` | No | Start of time range (ISO 8601) |
| `end_time` | `string` | No | End of time range (ISO 8601) |
| `aggregation` | `string` | No | Aggregation function: `avg`, `sum`, `min`, `max`, `count` |
| `interval` | `string` | No | Time interval for aggregation: `1m`, `5m`, `1h`, `1d` |

**Example Request:**

```http
GET /api/monitoring/v1/metrics/query?name=tachyon_cpu_usage_*&aggregation=avg&interval=5m HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Accept: application/vnd.tachyon.monitoring.v1+json
```

**Success Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "time_range": {
      "start": "2026-02-07T21:00:00.000Z",
      "end": "2026-02-07T21:55:00.000Z"
    },
    "metrics": [
      {
        "name": "tachyon_cpu_usage_seconds_total",
        "type": "counter",
        "labels": {
          "host": "tachyon-server-01",
          "component": "server"
        },
        "data_points": [
          {
            "timestamp": "2026-02-07T21:00:00.000Z",
            "value": 12345.67
          },
          {
            "timestamp": "2026-02-07T21:05:00.000Z",
            "value": 12346.23
          }
        ]
      }
    ]
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 400 Bad Request | `MONITORING_043` | Invalid query parameters |
| 403 Forbidden | `MONITORING_044` | Insufficient permissions |

#### 7.3.3. Get Metric Metadata

Retrieves metadata about available metrics.

**Endpoint:** `GET /api/monitoring/v1/metadata`

**Authentication:** Required (Admin, Operator, or Viewer role)

**Query Parameters:** None

**Example Request:**

```http
GET /api/monitoring/v1/metadata HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Accept: application/vnd.tachyon.monitoring.v1+json
```

**Success Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "metrics": [
      {
        "name": "tachyon_cpu_usage_seconds_total",
        "type": "counter",
        "description": "Total CPU usage in seconds",
        "unit": "seconds",
        "labels": ["host", "component"]
      },
      {
        "name": "tachyon_memory_usage_bytes",
        "type": "gauge",
        "description": "Current memory usage in bytes",
        "unit": "bytes",
        "labels": ["host", "component"]
      }
    ]
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 403 Forbidden | `MONITORING_045` | Insufficient permissions |

#### 7.3.4. Prometheus Exposition

Exposes metrics in Prometheus format for scraping.

**Endpoint:** `GET /api/monitoring/v1/metrics/prometheus`

**Authentication:** Optional (public endpoint for Prometheus scraping)

**Query Parameters:** None

**Example Request:**

```http
GET /api/monitoring/v1/metrics/prometheus HTTP/1.1
Host: tachyon.example.com
Accept: text/plain
```

**Success Response (200 OK):**

```
# HELP tachyon_cpu_usage_seconds_total
# TYPE counter
tachyon_cpu_usage_seconds_total{host="tachyon-server-01",component="server"} 12345.67

# HELP tachyon_memory_usage_bytes
# TYPE gauge
tachyon_memory_usage_bytes{host="tachyon-server-01",component="server"} 536870912

# HELP tachyon_http_requests_duration_seconds
# TYPE histogram
tachyon_http_requests_duration_seconds_bucket{le="0.005",endpoint="/api/documents",method="GET"} 1000
tachyon_http_requests_duration_seconds_bucket{le="0.01",endpoint="/api/documents",method="GET"} 2500
tachyon_http_requests_duration_seconds_bucket{le="0.025",endpoint="/api/documents",method="GET"} 5000
tachyon_http_requests_duration_seconds_bucket{le="0.05",endpoint="/api/documents",method="GET"} 10000
tachyon_http_requests_duration_seconds_bucket{le="0.1",endpoint="/api/documents",method="GET"} 5000
tachyon_http_requests_duration_seconds_bucket{le="0.25",endpoint="/api/documents",method="GET"} 2000
tachyon_http_requests_duration_seconds_bucket{le="0.5",endpoint="/api/documents",method="GET"} 1000
tachyon_http_requests_duration_seconds_bucket{le="1",endpoint="/api/documents",method="GET"} 500
tachyon_http_requests_duration_seconds_bucket{le="2.5",endpoint="/api/documents",method="GET"} 200
tachyon_http_requests_duration_seconds_bucket{le="5",endpoint="/api/documents",method="GET"} 100
tachyon_http_requests_duration_seconds_bucket{le="10",endpoint="/api/documents",method="GET"} 50
tachyon_http_requests_duration_seconds_bucket{le="+Inf",endpoint="/api/documents",method="GET"} 25
tachyon_http_requests_duration_seconds_sum{endpoint="/api/documents",method="GET"} 1234.56
tachyon_http_requests_duration_seconds_count{endpoint="/api/documents",method="GET"} 26580
```

### 7.4. Metrics Collection Configuration

Metrics collection behavior can be configured via the monitoring configuration file:

```toml
[monitoring.metrics]
# Metrics ingestion settings
[monitoring.metrics.ingestion]
max_batch_size = 1000
max_request_size_bytes = 104857600
async_processing = true

# Metrics retention
[monitoring.metrics.retention]
raw_retention_days = 90
aggregated_retention_days = 365
downsampling_enabled = true

# Prometheus exposition
[monitoring.metrics.prometheus]
enabled = true
path = "/api/monitoring/v1/metrics/prometheus"
scrape_interval_seconds = 15
```

---

## 8. TRACE COLLECTION API

### 8.1. Overview

The Trace Collection API provides endpoints for distributed tracing, enabling end-to-end request tracking across the Tachyon system. This API supports OpenTelemetry-compatible trace formats and enables integration with tracing systems like Jaeger, Zipkin, and Grafana Tempo.

**Related Requirements:**
- [REQ-SRV-106](../../.specs/04_future_state/reqs/server_requirements.md) through [REQ-SRV-120](../../.specs/04_future_state/reqs/server_requirements.md): Performance and resource management requirements

### 8.2. Trace Concepts

The following concepts are fundamental to distributed tracing:

| Concept | Description |
|----------|-------------|
| **Trace** | A distributed execution path through a system, representing a single request or transaction |
| **Span** | A named, timed operation representing a unit of work within a trace |
| **Trace ID** | Unique identifier for a trace, propagated across all services |
| **Span ID** | Unique identifier for a span within a trace |
| **Parent Span ID** | Identifier of the parent span, establishing the span hierarchy |
| **Span Kind** | Type of span: `server`, `client`, `producer`, `consumer`, `internal` |
| **Span Status** | Status of span: `unset`, `ok`, `error` |

### 8.3. Endpoints

#### 8.3.1. Ingest Spans

Submits span data for trace aggregation.

**Endpoint:** `POST /api/monitoring/v1/traces/spans`

**Authentication:** Required (System or Admin role)

**Request Body (OpenTelemetry Format):**

```json
{
  "resource_spans": [
    {
      "resource": {
        "service.name": "tachyon-server",
        "service.version": "1.0.0",
        "host.name": "tachyon-server-01",
        "deployment.environment": "production"
      },
      "scope_spans": [
        {
          "scope": {
            "name": "tachyon.server.http"
          },
          "spans": [
            {
              "trace_id": "4bf92f3577b34da6a3ce929d0e0f47",
              "span_id": "00f067aa0ba902b7",
              "parent_span_id": "00f067aa0ba902b6",
              "name": "GET /api/documents",
              "kind": "server",
              "start_time_unix_nano": 16442689089710000000,
              "end_time_unix_nano": 16442689089860000000,
              "status": {
                "code": 1,
                "message": "OK"
              },
              "attributes": {
                "http.method": "GET",
                "http.url": "/api/documents",
                "http.status_code": 200,
                "http.user_agent": "Mozilla/5.0",
                "user.id": "user_456"
              },
              "events": [
                {
                  "name": "cache.lookup",
                  "timestamp_unix_nano": 16442689089750000000,
                  "attributes": {
                    "cache.key": "doc_abc123",
                    "cache.hit": true
                  }
                }
              ],
              "links": [
                {
                  "trace_id": "4bf92f3577b34da6a3ce929d0e0f47",
                  "span_id": "00f067aa0ba902b5",
                  "relation_type": "child_of"
                }
              ]
            }
          ]
        }
      ]
    }
  ]
}
```

**Span Fields:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `trace_id` | `string` | Yes | Trace identifier |
| `span_id` | `string` | Yes | Span identifier |
| `parent_span_id` | `string` | No | Parent span identifier |
| `name` | `string` | Yes | Span name |
| `kind` | `string` | No | Span kind |
| `start_time_unix_nano` | `integer` | Yes | Start time in nanoseconds since Unix epoch |
| `end_time_unix_nano` | `integer` | Yes | End time in nanoseconds since Unix epoch |
| `status` | `object` | No | Span status |
| `attributes` | `object` | No | Span attributes |
| `events` | `array` | No | Span events |
| `links` | `array` | No | Span links |

**Example Request:**

```http
POST /api/monitoring/v1/traces/spans HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Content-Type: application/vnd.tachyon.monitoring.v1+json

{
  "resource_spans": [
    {
      "resource": {
        "service.name": "tachyon-server"
      },
      "scope_spans": [
        {
          "scope": {
            "name": "tachyon.server.http"
          },
          "spans": [
            {
              "trace_id": "4bf92f3577b34da6a3ce929d0e0f47",
              "span_id": "00f067aa0ba902b7",
              "name": "GET /api/documents",
              "start_time_unix_nano": 16442689089710000000,
              "end_time_unix_nano": 16442689089860000000
            }
          ]
        }
      ]
    }
  ]
}
```

**Success Response (201 Created):**

```json
{
  "success": true,
  "data": {
    "ingested_count": 1,
    "batch_id": "batch_abc123def456",
    "timestamp": "2026-02-07T21:51:48.971Z"
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 400 Bad Request | `MONITORING_046` | Invalid span format |
| 403 Forbidden | `MONITORING_047` | Insufficient permissions |
| 413 Payload Too Large | `MONITORING_048` | Request body exceeds size limit |
| 500 Internal Server Error | `MONITORING_049` | Error ingesting spans |

#### 8.3.2. Query Traces

Retrieves trace data for analysis.

**Endpoint:** `GET /api/monitoring/v1/traces/query`

**Authentication:** Required (Admin, Operator, or Viewer role)

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|-----------|-------------|
| `trace_id` | `string` | No | Filter by trace ID |
| `service_name` | `string` | No | Filter by service name |
| `span_name` | `string` | No | Filter by span name |
| `start_time` | `string` | No | Start of time range (ISO 8601) |
| `end_time` | `string` | No | End of time range (ISO 8601) |
| `min_duration_ms` | `integer` | No | Minimum duration filter |
| `max_duration_ms` | `integer` | No | Maximum duration filter |
| `limit` | `integer` | No | Maximum results (default: 100, max: 1000) |
| `offset` | `integer` | No | Result offset for pagination |

**Example Request:**

```http
GET /api/monitoring/v1/traces/query?service_name=tachyon-server&min_duration_ms=100&limit=50 HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Accept: application/vnd.tachyon.monitoring.v1+json
```

**Success Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "traces": [
      {
        "trace_id": "4bf92f3577b34da6a3ce929d0e0f47",
        "root_span_id": "00f067aa0ba902b0",
        "service_name": "tachyon-server",
        "start_time": "2026-02-07T21:51:48.971Z",
        "end_time": "2026-02-07T21:51:48.986Z",
        "duration_ms": 15,
        "span_count": 3,
        "spans": [
          {
            "span_id": "00f067aa0ba902b0",
            "parent_span_id": null,
            "name": "GET /api/documents",
            "kind": "server",
            "start_time": "2026-02-07T21:51:48.971Z",
            "end_time": "2026-02-07T21:51:48.986Z",
            "duration_ms": 15,
            "status": {
              "code": 1,
              "message": "OK"
            },
            "attributes": {
              "http.method": "GET",
              "http.url": "/api/documents",
              "http.status_code": 200
            }
          }
        ]
      }
    ],
    "total_count": 25,
    "page_info": {
      "limit": 50,
      "offset": 0,
      "has_more": false
    }
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 400 Bad Request | `MONITORING_050` | Invalid query parameters |
| 403 Forbidden | `MONITORING_051` | Insufficient permissions |

#### 8.3.3. Get Trace by ID

Retrieves a specific trace.

**Endpoint:** `GET /api/monitoring/v1/traces/{trace_id}`

**Authentication:** Required (Admin, Operator, or Viewer role)

**Path Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|-----------|-------------|
| `trace_id` | `string` | Yes | Trace identifier |

**Example Request:**

```http
GET /api/monitoring/v1/traces/4bf92f3577b34da6a3ce929d0e0f47 HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Accept: application/vnd.tachyon.monitoring.v1+json
```

**Success Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "trace_id": "4bf92f3577b34da6a3ce929d0e0f47",
    "root_span_id": "00f067aa0ba902b0",
    "service_name": "tachyon-server",
    "start_time": "2026-02-07T21:51:48.971Z",
    "end_time": "2026-02-07T21:51:48.986Z",
    "duration_ms": 15,
    "span_count": 3,
    "spans": [
      {
        "span_id": "00f067aa0ba902b0",
        "parent_span_id": null,
        "name": "GET /api/documents",
        "kind": "server",
        "start_time": "2026-02-07T21:51:48.971Z",
        "end_time": "2026-02-07T21:51:48.986Z",
        "duration_ms": 15,
        "status": {
          "code": 1,
          "message": "OK"
        },
        "attributes": {
          "http.method": "GET",
          "http.url": "/api/documents",
          "http.status_code": 200
        },
        "events": [
          {
            "name": "cache.lookup",
            "timestamp": "2026-02-07T21:51:48.975Z",
            "attributes": {
              "cache.key": "doc_abc123",
              "cache.hit": true
            }
          }
        ]
      }
    ]
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 404 Not Found | `MONITORING_052` | Trace not found |
| 403 Forbidden | `MONITORING_053` | Insufficient permissions |

### 8.4. Trace Collection Configuration

Trace collection behavior can be configured via the monitoring configuration file:

```toml
[monitoring.traces]
# Trace ingestion settings
[monitoring.traces.ingestion]
max_batch_size = 500
max_request_size_bytes = 52428800
async_processing = true

# Trace retention
[monitoring.traces.retention]
detailed_retention_days = 7
sampled_retention_days = 30
sampling_rate = 0.1

# Trace sampling
[monitoring.traces.sampling]
enabled = true
strategy = "probabilistic"
default_rate = 0.1
service_specific_rates = { "tachyon-server" = 0.5, "tachyon-web" = 1.0 }
```

---

## 9. INCIDENT MANAGEMENT API

### 9.1. Overview

The Incident Management API provides endpoints for creating, updating, and resolving incidents. This API enables structured incident response workflows, incident tracking, and post-incident analysis.

**Related Requirements:**
- [REQ-SRV-106](../../.specs/04_future_state/reqs/server_requirements.md) through [REQ-SRV-120](../../.specs/04_future_state/reqs/server_requirements.md): Performance and resource management requirements

### 9.2. Incident Lifecycle

The following lifecycle states are supported for incidents:

| State | Description | Transitions |
|--------|-------------|-------------|
| `OPEN` | Incident has been created and is being investigated | From: `CREATED` |
| `IN_PROGRESS` | Active resolution is in progress | From: `OPEN` |
| `RESOLVED` | Incident has been resolved | From: `IN_PROGRESS` |
| `CLOSED` | Incident is closed and archived | From: `RESOLVED` |

### 9.3. Incident Severity Levels

| Level | Value | Description | Response Time Target |
|-------|-------|-------------|-------------------|
| `P1` | 1 | Critical - Service unavailable | 15 minutes |
| `P2` | 2 | Major - Significant impact | 1 hour |
| `P3` | 3 | Minor - Partial impact | 4 hours |
| `P4` | 4 | Low - Minimal impact | 24 hours |

### 9.4. Endpoints

#### 9.4.1. Create Incident

Creates a new incident.

**Endpoint:** `POST /api/monitoring/v1/incidents`

**Authentication:** Required (Admin or Operator role)

**Request Body:**

```json
{
  "title": "High CPU Usage Causing Slow Response Times",
  "description": "CPU usage exceeded 90% for extended period, causing degraded performance",
  "severity": "P2",
  "status": "OPEN",
  "component": "server",
  "affected_services": ["document-retrieval", "search"],
  "impacted_users": 1000,
  "detected_at": "2026-02-07T21:51:48.971Z",
  "assigned_to": "ops-team",
  "tags": ["performance", "cpu", "degradation"],
  "related_alert_ids": ["evt_abc123def456"],
  "metadata": {
    "hostname": "tachyon-server-01",
    "cpu_usage_percent": 92.5
  }
}
```

**Request Fields:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `title` | `string` | Yes | Incident title |
| `description` | `string` | No | Detailed description |
| `severity` | `string` | No | Severity level (P1, P2, P3, P4) |
| `status` | `string` | No | Initial status (default: OPEN) |
| `component` | `string` | No | Affected component |
| `affected_services` | `array` | No | List of affected services |
| `impacted_users` | `integer` | No | Estimated number of impacted users |
| `detected_at` | `string` | No | Detection timestamp (ISO 8601) |
| `assigned_to` | `string` | No | Assigned team or user |
| `tags` | `array` | No | Incident tags |
| `related_alert_ids` | `array` | No | Related alert event IDs |
| `metadata` | `object` | No | Additional incident metadata |

**Example Request:**

```http
POST /api/monitoring/v1/incidents HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Content-Type: application/vnd.tachyon.monitoring.v1+json

{
  "title": "High CPU Usage Causing Slow Response Times",
  "description": "CPU usage exceeded 90% for extended period",
  "severity": "P2",
  "component": "server"
}
```

**Success Response (201 Created):**

```json
{
  "success": true,
  "data": {
    "incident_id": "inc_abc123def456",
    "title": "High CPU Usage Causing Slow Response Times",
    "description": "CPU usage exceeded 90% for extended period, causing degraded performance",
    "severity": "P2",
    "status": "OPEN",
    "component": "server",
    "affected_services": ["document-retrieval", "search"],
    "impacted_users": 1000,
    "detected_at": "2026-02-07T21:51:48.971Z",
    "created_at": "2026-02-07T21:51:48.971Z",
    "created_by": "admin@tachyon.example.com",
    "assigned_to": "ops-team",
    "tags": ["performance", "cpu", "degradation"],
    "related_alert_ids": ["evt_abc123def456"],
    "metadata": {
      "hostname": "tachyon-server-01",
      "cpu_usage_percent": 92.5
    }
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 400 Bad Request | `MONITORING_054` | Invalid incident data |
| 403 Forbidden | `MONITORING_055` | Insufficient permissions |

#### 9.4.2. List Incidents

Retrieves incidents with filtering.

**Endpoint:** `GET /api/monitoring/v1/incidents`

**Authentication:** Required (Admin, Operator, or Viewer role)

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|-----------|-------------|
| `status` | `string` | No | Filter by status |
| `severity` | `string` | No | Filter by severity |
| `component` | `string` | No | Filter by component |
| `assigned_to` | `string` | No | Filter by assignee |
| `start_time` | `string` | No | Start of time range (ISO 8601) |
| `end_time` | `string` | No | End of time range (ISO 8601) |
| `limit` | `integer` | No | Maximum results (default: 100, max: 1000) |
| `offset` | `integer` | No | Result offset for pagination |
| `sort` | `string` | No | Sort field (default: detected_at) |
| `order` | `string` | No | Sort order: `asc` or `desc` (default: `desc`) |

**Example Request:**

```http
GET /api/monitoring/v1/incidents?status=OPEN&severity=P2&limit=50 HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Accept: application/vnd.tachyon.monitoring.v1+json
```

**Success Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "incidents": [
      {
        "incident_id": "inc_abc123def456",
        "title": "High CPU Usage Causing Slow Response Times",
        "severity": "P2",
        "status": "OPEN",
        "component": "server",
        "detected_at": "2026-02-07T21:51:48.971Z",
        "created_at": "2026-02-07T21:51:48.971Z",
        "assigned_to": "ops-team"
      }
    ],
    "total_count": 25,
    "page_info": {
      "limit": 50,
      "offset": 0,
      "has_more": false
    }
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 400 Bad Request | `MONITORING_056` | Invalid query parameters |
| 403 Forbidden | `MONITORING_057` | Insufficient permissions |

#### 9.4.3. Get Incident

Retrieves a specific incident.

**Endpoint:** `GET /api/monitoring/v1/incidents/{incident_id}`

**Authentication:** Required (Admin, Operator, or Viewer role)

**Path Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|-----------|-------------|
| `incident_id` | `string` | Yes | Incident identifier |

**Example Request:**

```http
GET /api/monitoring/v1/incidents/inc_abc123def456 HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Accept: application/vnd.tachyon.monitoring.v1+json
```

**Success Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "incident_id": "inc_abc123def456",
    "title": "High CPU Usage Causing Slow Response Times",
    "description": "CPU usage exceeded 90% for extended period, causing degraded performance",
    "severity": "P2",
    "status": "IN_PROGRESS",
    "component": "server",
    "affected_services": ["document-retrieval", "search"],
    "impacted_users": 1000,
    "detected_at": "2026-02-07T21:51:48.971Z",
    "created_at": "2026-02-07T21:51:48.971Z",
    "created_by": "admin@tachyon.example.com",
    "assigned_to": "ops-team",
    "tags": ["performance", "cpu", "degradation"],
    "timeline": [
      {
        "timestamp": "2026-02-07T21:51:48.971Z",
        "status": "OPEN",
        "description": "Incident created",
        "user": "admin@tachyon.example.com"
      },
      {
        "timestamp": "2026-02-07T22:00:00.000Z",
        "status": "IN_PROGRESS",
        "description": "Incident assigned to ops-team",
        "user": "admin@tachyon.example.com"
      }
    ],
    "related_alert_ids": ["evt_abc123def456"],
    "metadata": {
      "hostname": "tachyon-server-01",
      "cpu_usage_percent": 92.5
    }
  },
  "error": null,
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 404 Not Found | `MONITORING_058` | Incident not found |
| 403 Forbidden | `MONITORING_059` | Insufficient permissions |

#### 9.4.4. Update Incident

Updates an existing incident.

**Endpoint:** `PUT /api/monitoring/v1/incidents/{incident_id}`

**Authentication:** Required (Admin or Operator role)

**Path Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|-----------|-------------|
| `incident_id` | `string` | Yes | Incident identifier |

**Request Body:** Same as create incident (all fields optional)

**Example Request:**

```http
PUT /api/monitoring/v1/incidents/inc_abc123def456 HTTP/2
Host: tachyon.example.com
Authorization: Bearer <jwt_token>
Content-Type: application/vnd.tachyon.monitoring.v1+json

{
  "status": "RESOLVED",
  "resolution": "Increased server capacity and implemented auto-scaling",
  "resolved_at": "2026-02-07T23:00:00.000Z",
  "resolved_by": "ops@tachyon.example.com"
}
```

**Success Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "incident_id": "inc_abc123def456",
    "status": "RESOLVED",
    "resolution": "Increased server capacity and implemented auto-scaling",
    "resolved_at": "2026-02-07T23:00:00.000Z",
    "resolved_by": "ops@tachyon.example.com",
    "updated_at": "2026-02-07T23:00:00.000Z"
  },
  "error": null,
  "timestamp": "2026-02-07T23:00:00.000Z",
  "request_id": "req_abc123def456"
}
```

**Error Responses:**

| Status Code | Error Code | Description |
|-------------|------------|-------------|
| 400 Bad Request | `MONITORING_060` | Invalid incident data |
| 404 Not Found | `MONITORING_061` | Incident not found |
| 403 Forbidden | `MONITORING_062` | Insufficient permissions |

#### 9.4.5. Add Incident Comment

Adds a comment to an incident.

**Endpoint:** `POST /api/monitoring/v1/incidents/{incident_id}/comments`

**Authentication:** Required (Admin or Operator role)

**Path Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|-----------|-------------|
| `incident_id` | `string` | Yes | Incident identifier |

**Request Body:**

```json
{
  "comment": "Investigating potential memory leak in search service",
  "is_internal": true
}
```

**Request Fields:**

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `comment` | `string` | Yes | Comment text |
| `is_internal` | `boolean` | No | Whether comment is internal (default: false) |

**Success Response (201 Created):**

```json
{
  "success": true,
  "data": {
    "comment_id": "cmt_abc123def456",
    "incident_id": "inc_abc123def456",
    "comment": "Investigating potential memory leak in search service",
    "is_internal": true,
    "created_at": "2026-02-07T22:30:00.000Z",
    "created_by": "ops@tachyon.example.com"
  },
  "error": null,
  "timestamp": "2026-02-07T22:30:00.000Z",
  "request_id": "req_abc123def456"
}
```

### 9.5. Incident Management Configuration

Incident management behavior can be configured via the monitoring configuration file:

```toml
[monitoring.incidents]
# Incident creation settings
[monitoring.incidents.creation]
auto_create_from_alerts = true
alert_to_incident_mapping = { "CRITICAL" = "P1", "ERROR" = "P2", "WARNING" = "P3" }

# Incident lifecycle
[monitoring.incidents.lifecycle]
auto_escalation_enabled = true
escalation_timeouts = { "P1" = 3600, "P2" = 86400, "P3" = 172800 }
auto_closure_enabled = true
closure_timeouts = { "RESOLVED" = 604800 }

# Incident retention
[monitoring.incidents.retention]
open_incident_retention_days = 2555
closed_incident_retention_days = 2555
```

---

## 10. ERROR HANDLING

### 10.1. Overview

The Monitoring API implements comprehensive error handling following the fail-safe error handling principles established in [ADR-010](../../.specs/02_adrs/010_security_architecture.md). All errors are handled securely without exposing sensitive information or creating security vulnerabilities.

### 10.2. Error Response Format

All error responses follow a consistent format:

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "MONITORING_XXX",
    "message": "Human-readable error message",
    "details": { ... }
  },
  "timestamp": "2026-02-07T21:51:48.971Z",
  "request_id": "req_abc123def456"
}
```

**Error Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `code` | `string` | Unique error code for programmatic handling |
| `message` | `string` | Human-readable error message |
| `details` | `object` | Additional error context (optional) |

### 10.3. Error Codes

The following error codes are used across the Monitoring API:

| Error Code | HTTP Status | Category | Description |
|-------------|-------------|----------|-------------|
| `MONITORING_001` | 503 | Health Check | Health check failed |
| `MONITORING_002` | 500 | Health Check | Internal error during health check |
| `MONITORING_003` | 503 | Health Check | System is not ready to accept traffic |
| `MONITORING_004` | 503 | Health Check | System is not alive |
| `MONITORING_005` | 404 | Health Check | Component not found |
| `MONITORING_006` | 403 | Health Check | Insufficient permissions |
| `MONITORING_007` | 403 | Diagnostics | Insufficient permissions |
| `MONITORING_008` | 500 | Diagnostics | Error collecting diagnostics |
| `MONITORING_009` | 403 | Diagnostics | Insufficient permissions |
| `MONITORING_010` | 500 | Diagnostics | Error collecting thread dump |
| `MONITORING_011` | 500 | Diagnostics | Error collecting thread dump |
| `MONITORING_012` | 403 | Diagnostics | Insufficient permissions |
| `MONITORING_013` | 500 | Diagnostics | Error collecting thread dump |
| `MONITORING_014` | 403 | Diagnostics | Insufficient permissions |
| `MONITORING_015` | 500 | Diagnostics | Error collecting memory statistics |
| `MONITORING_016` | 400 | Log Aggregation | Invalid log format |
| `MONITORING_017` | 403 | Log Aggregation | Insufficient permissions |
| `MONITORING_018` | 413 | Log Aggregation | Request body exceeds size limit |
| `MONITORING_019` | 500 | Log Aggregation | Error ingesting logs |
| `MONITORING_020` | 400 | Log Aggregation | Invalid query parameters |
| `MONITORING_021` | 403 | Log Aggregation | Insufficient permissions |
| `MONITORING_022` | 400 | Log Aggregation | Invalid query parameters |
| `MONITORING_023` | 403 | Log Aggregation | Insufficient permissions |
| `MONITORING_024` | 400 | Log Aggregation | Invalid export parameters |
| `MONITORING_025` | 403 | Log Aggregation | Insufficient permissions |
| `MONITORING_026` | 400 | Alerting | Invalid alert rule configuration |
| `MONITORING_027` | 409 | Alerting | Alert rule name already exists |
| `MONITORING_028` | 403 | Alerting | Insufficient permissions |
| `MONITORING_029` | 400 | Alerting | Invalid query parameters |
| `MONITORING_030` | 403 | Alerting | Insufficient permissions |
| `MONITORING_031` | 400 | Alerting | Invalid query parameters |
| `MONITORING_032` | 400 | Alerting | Invalid alert rule configuration |
| `MONITORING_033` | 404 | Alerting | Alert rule not found |
| `MONITORING_034` | 403 | Alerting | Insufficient permissions |
| `MONITORING_035` | 404 | Alerting | Alert rule not found |
| `MONITORING_036` | 403 | Alerting | Insufficient permissions |
| `MONITORING_037` | 400 | Alerting | Invalid query parameters |
| `MONITORING_038` | 403 | Alerting | Insufficient permissions |
| `MONITORING_039` | 400 | Metrics | Invalid metric format |
| `MONITORING_040` | 403 | Metrics | Insufficient permissions |
| `MONITORING_041` | 413 | Metrics | Request body exceeds size limit |
| `MONITORING_042` | 500 | Metrics | Error ingesting metrics |
| `MONITORING_043` | 400 | Metrics | Invalid query parameters |
| `MONITORING_044` | 403 | Metrics | Insufficient permissions |
| `MONITORING_045` | 403 | Metrics | Insufficient permissions |
| `MONITORING_046` | 400 | Trace Collection | Invalid span format |
| `MONITORING_047` | 403 | Trace Collection | Insufficient permissions |
| `MONITORING_048` | 413 | Trace Collection | Request body exceeds size limit |
| `MONITORING_049` | 500 | Trace Collection | Error ingesting spans |
| `MONITORING_050` | 400 | Trace Collection | Invalid query parameters |
| `MONITORING_051` | 403 | Trace Collection | Insufficient permissions |
| `MONITORING_052` | 404 | Trace Collection | Trace not found |
| `MONITORING_053` | 403 | Trace Collection | Insufficient permissions |
| `MONITORING_054` | 400 | Incident Management | Invalid incident data |
| `MONITORING_055` | 403 | Incident Management | Insufficient permissions |
| `MONITORING_056` | 400 | Incident Management | Invalid query parameters |
| `MONITORING_057` | 403 | Incident Management | Insufficient permissions |
| `MONITORING_058` | 404 | Incident Management | Incident not found |
| `MONITORING_059` | 403 | Incident Management | Insufficient permissions |
| `MONITORING_060` | 400 | Incident Management | Invalid incident data |
| `MONITORING_061` | 404 | Incident Management | Incident not found |
| `MONITORING_062` | 403 | Incident Management | Insufficient permissions |
| `MONITORING_063` | 401 | Rate Limiting | Rate limit exceeded |
| `MONITORING_064` | 413 | Payload | Request body exceeds size limit |
| `MONITORING_065` | 401 | Authentication | Invalid or missing authentication |
| `MONITORING_066` | 403 | Authorization | Insufficient permissions |
| `MONITORING_067` | 400 | Validation | Invalid request parameters |
| `MONITORING_068` | 415 | Media Type | Unsupported media type |
| `MONITORING_069` | 404 | Version | Unsupported API version |
| `MONITORING_070` | 500 | Internal | Internal server error |

### 10.4. HTTP Status Codes

The following HTTP status codes are used by the Monitoring API:

| Status Code | Category | Description |
|-------------|----------|-------------|
| 200 OK | Success | Request completed successfully |
| 201 Created | Success | Resource created successfully |
| 204 No Content | Success | Request completed successfully with no response body |
| 400 Bad Request | Client Error | Invalid request parameters or body |
| 401 Unauthorized | Client Error | Authentication required or invalid |
| 403 Forbidden | Client Error | Insufficient permissions |
| 404 Not Found | Client Error | Resource not found |
| 409 Conflict | Client Error | Resource conflict (e.g., duplicate name) |
| 413 Payload Too Large | Client Error | Request body exceeds size limit |
| 415 Unsupported Media Type | Client Error | Unsupported content type |
| 500 Internal Server Error | Server Error | Internal server error |
| 503 Service Unavailable | Server Error | Service temporarily unavailable |

### 10.5. Error Handling Strategies

The following error handling strategies are implemented:

#### 10.5.1. Input Validation

All input is validated before processing:

- **Type Validation:** Ensure all fields match expected types
- **Range Validation:** Ensure numeric values are within valid ranges
- **Format Validation:** Ensure string values match expected formats (ISO 8601 timestamps)
- **Length Validation:** Ensure string values are within length limits
- **Enum Validation:** Ensure enum values are from allowed set

#### 10.5.2. Authentication and Authorization

Authentication and authorization errors are handled securely:

- **Generic Messages:** Authentication errors return generic messages to prevent information leakage
- **Rate Limiting:** Failed authentication attempts are rate limited to prevent brute force attacks
- **Token Validation:** JWT tokens are validated for signature and expiration
- **Permission Checks:** All resource access is validated against user roles

#### 10.5.3. Rate Limiting

Rate limiting is enforced to prevent abuse:

- **Per-User Limits:** Each user has independent rate limits
- **Per-IP Limits:** Each IP address has independent rate limits
- **Burst Allowance:** Temporary bursts are allowed within limits
- **Headers:** Rate limit information is included in response headers

#### 10.5.4. Error Logging

All errors are logged securely for audit and debugging:

- **Structured Logging:** Errors are logged with structured fields
- **Sanitization:** Sensitive information is removed from error logs
- **Correlation:** Errors are correlated with request IDs for tracing
- **Aggregation:** Error statistics are aggregated for monitoring

#### 10.5.5. Client Guidance

Error responses include guidance for clients:

- **Retry-After Headers:** Include recommended retry delay for rate-limited requests
- **Documentation Links:** Include links to relevant documentation
- **Troubleshooting Steps:** Include suggested troubleshooting steps where applicable

### 10.6. Error Handling Configuration

Error handling behavior can be configured via the monitoring configuration file:

```toml
[monitoring.error_handling]
# Error logging
[monitoring.error_handling.logging]
enabled = true
log_level = "ERROR"
include_stack_trace = false

# Error responses
[monitoring.error_handling.responses]
include_request_id = true
include_timestamp = true
sanitize_errors = true

# Rate limiting
[monitoring.error_handling.rate_limiting]
enabled = true
default_requests_per_minute = 100
default_burst = 10
```
```

---

## 11. REFERENCES

### 11.1. Internal Project References

The following internal project documents are referenced throughout this document:

| Document ID | Title | Path |
|-------------|-------|------|
| [TACHYON-STD-V1.0](../../.specs/01_standards/coding_standards.md) | Coding and Documentation Standards |  |
| [TACHYON-REQ-SRV-V1.0](../../.specs/04_future_state/reqs/server_requirements.md) | Server Application Requirements |  |
| [TACHYON-ADR-001-V1.0](../../.specs/02_adrs/001_rust_as_primary_language.md) | Rust as Primary Language |  |
| [TACHYON-ADR-010-V1.0](../../.specs/02_adrs/010_security_architecture.md) | Security Architecture |  |
| [TACHYON-TST-V1.0](../../.specs/04_future_state/test_plan.md) | Test Plan |  |

### 11.2. External Standards References

The following external standards are referenced throughout this document:

| Standard | Description | Reference |
|----------|-------------|----------|
| **ISO/IEC 26514:2021** | Systems and Software Engineering—Requirements for Designers and Developers of User Documentation | [1] |
| **ISO/IEC 12207:2017** | Systems and Software Engineering—Software Lifecycle Processes | [2] |
| **ISO/IEC 25010:2011** | System and Software Quality Requirements and Evaluations | [3] |
| **IEEE 829-2008** | Software Test Documentation | [4] |
| **IEEE 1063-2001** | Standard for Software User Documentation | [5] |
| **IEEE 1016-2009** | Standard for Information Technology | [6] |

### 11.3. Technical References

The following technical references are relevant to the Monitoring API:

| Reference | Description | URL |
|-----------|-------------|------|
| **OpenTelemetry Specification** | OpenTelemetry trace format specification | https://opentelemetry.io/docs/reference/specification/ |
| **Prometheus Data Model** | Prometheus metric exposition format | https://prometheus.io/docs/concepts/data_model/ |
| **OpenTracing Protocol** | OpenTracing trace protocol specification | https://opentracing.io/docs/ |
| **Jaeger Architecture** | Jaeger distributed tracing platform | https://www.jaegertracing.io/docs/ |
| **Zipkin API** | Zipkin distributed tracing API | https://zipkin.io/zipkin-api/ |

### 11.4. Bibliography

[1] The Rust Project, The Rust Reference, Online. Available: https://doc.rust-lang.org/reference/. [Accessed: 01-Feb-2026].

[2] Tokio Contributors, Tokio: Asynchronous Runtime for Rust, Online. Available: https://tokio.rs/. [Accessed: 01-Feb-2026].

[3] Axum Contributors, Axum: Web Framework, Online. Available: https://docs.rs/axum/. [Accessed: 01-Feb-2026].

[4] The OpenTelemetry Authors, OpenTelemetry Specification, Online. Available: https://opentelemetry.io/docs/reference/specification/. [Accessed: 01-Feb-2026].

[5] Prometheus Authors, Prometheus Data Model, Online. Available: https://prometheus.io/docs/concepts/data_model/. [Accessed: 01-Feb-2026].

[6] OpenTracing Authors, OpenTracing Protocol Specification, Online. Available: https://opentracing.io/docs/. [Accessed: 01-Feb-2026].

[7] International Organization for Standardization (ISO), ISO/IEC 26514:2021: Systems and Software Engineering—Requirements for Designers and Developers of User Documentation, Online. Available: https://www.iso.org/standard/68983.html. [Accessed: 01-Feb-2026].

[8] Institute of Electrical and Electronics Engineers (IEEE), IEEE 829-2008: Software Test Documentation, Online. Available: https://standards.ieee.org/standard/829/2008.html. [Accessed: 01-Feb-2026].

[9] Institute of Electrical and Electronics Engineers (IEEE), IEEE 1063-2001: Standard for Software User Documentation, Online. Available: https://standards.ieee.org/standard/1063/2001.html. [Accessed: 01-Feb-2026].

[10] Institute of Electrical and Electronics Engineers (IEEE), IEEE 1016-2009: Standard for Information Technology, Online. Available: https://standards.ieee.org/standard/1016-2009.html. [Accessed: 01-Feb-2026].

---

**Document Control Information**

- **Document ID:** TACHYON-API-014-V1.0
- **Title:** TACHYON: MONITORING API DOCUMENTATION
- **Version:** 1.0
- **Date:** February 2026
- **Status:** Proposed
- **Classification:** API Specification
- **Compliance Level:** ISO/IEC 26514:2021, IEEE 1063-2001

---

**Document Change History**

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | February 2026 | Initial document creation |

---

**Document Approval**

| Role | Name | Date | Status |
|-------|------|--------|-------------|
| Technical Writer | Tachyon Documentation Team | February 2026 | Approved |

---

**End of Document**

