# TACHYON: WEB CLIENT API SPECIFICATION

**Document ID:** TACHYON-API-011-V1.0
**Date:** February 2026
**Status:** Approved for Implementation
**Classification:** Technical Specification
**Compliance Level:** ISO/IEC 26514:2021, IEEE 1063-2001

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [Client API Design Principles](#2-client-api-design-principles)
3. [Client Initialization](#3-client-initialization)
4. [HTTP Client Methods](#4-http-client-methods)
5. [WebSocket Client Methods](#5-websocket-client-methods)
6. [WebSocket Subscription Methods](#6-websocket-subscription-methods)
7. [Error Handling](#7-error-handling)
8. [Client Security](#8-client-security)
9. [Client Performance](#9-client-performance)
10. [References](#10-references)

---

## 1. INTRODUCTION

### 1.1. Document Purpose

This document provides a comprehensive specification of the Tachyon Web Client API, defining the type-safe client-side interfaces for communication between the Leptos-based web frontend and the Axum-based server component. The Web Client API specification encompasses both HTTP/2 RESTful client methods and WebSocket real-time communication abstractions, ensuring type-safe, performant, and secure interactions across the application architecture.

### 1.2. Scope

The Web Client API specification covers:
- HTTP/2 RESTful client methods for server communication
- WebSocket client abstractions for real-time bidirectional communication
- Client initialization and configuration
- Type-safe request/response interfaces
- Subscription management for real-time events
- Error handling and recovery protocols
- Security and authentication mechanisms
- Performance optimization strategies

Out of scope:
- Server endpoint definitions (covered in REST API specification)
- Server internal APIs (covered in server API specification)
- Desktop application client APIs (covered in desktop API specification)

### 1.3. Document Dependencies

This document depends on the following documents:
- [TACHYON-STD-V1.0](../../.adrs/ - Coding and Documentation Standards
- [TACHYON-REQ-WEB-V1.0](../../.adrs/ - Web Frontend Requirements
- [TACHYON-DES-WD-V1.0](../../.adrs/ - Web Frontend Design
- [TACHYON-ADR-004-V1.0](../../.adrs/adr-004-debounce-window.md) - ADR-004: Leptos for Web Frontend
- [TACHYON-ADR-005-V1.0](../../.adrs/adr-005-last-write-wins-conflict-resolution.md) - ADR-005: Bun for JavaScript Runtime
- [TACHYON-TMA-V1.0](../../.adrs/ - Threat Model Analysis

### 1.4. Terminology

| Term | Definition |
|------|------------|
| **ApiClient** | Type-safe HTTP client abstraction for server communication |
| **WebSocketClient** | Real-time bidirectional communication client for live updates |
| **Signal** | Leptos reactive primitive for state management |
| **Subscription** | Active listener for WebSocket events |
| **Request Deduplication** | Preventing redundant concurrent requests |
| **Response Caching** | In-memory caching of GET responses with TTL |

---

## 2. CLIENT API DESIGN PRINCIPLES

### 2.1. Architectural Principles

The Tachyon Web Client API adheres to the following architectural principles:

#### 2.1.1. Type Safety First

All client interfaces are defined with strict type safety using TypeScript. The type system ensures compile-time correctness and prevents entire classes of runtime errors.

**Implementation:**
- TypeScript interfaces for all request/response types
- Strict null checking enabled in TypeScript configuration
- Type guards for runtime validation
- Generic type parameters for reusable abstractions

**Rationale:** Type safety reduces bugs, improves developer experience, and enables confident refactoring [REQ-WEB-037].

#### 2.1.2. Reactive State Management

The client API leverages Leptos's fine-grained reactivity model for efficient state updates. State changes propagate automatically to dependent components without manual re-rendering.

**Implementation:**
- Leptos signals for reactive state
- Automatic dependency tracking
- Minimal DOM updates
- No virtual DOM overhead

**Rationale:** Fine-grained reactivity provides superior performance compared to virtual DOM diffing [ADR-004].

#### 2.1.3. Performance Optimization

The client API is designed for optimal performance with minimal latency and efficient resource utilization.

**Implementation:**
- Request deduplication for concurrent identical requests
- Response caching with configurable TTL
- Request cancellation using AbortController
- Automatic retry logic with exponential backoff
- Lazy loading of resources

**Rationale:** Performance optimization ensures responsive user experience and efficient resource usage [REQ-WEB-066, REQ-WEB-067].

#### 2.1.4. Security by Design

Security considerations are integrated into all client API design decisions, not added as an afterthought.

**Implementation:**
- Automatic authentication token injection
- Secure credential storage (HttpOnly cookies, localStorage)
- Input validation and sanitization
- CSRF protection
- Secure WebSocket connections (WSS)

**Rationale:** Security by design prevents vulnerabilities and protects sensitive data [TACHYON-TMA-V1.0].

#### 2.1.5. Developer Experience

The client API prioritizes developer experience through intuitive interfaces, comprehensive error messages, and clear documentation.

**Implementation:**
- Intuitive method signatures
- Comprehensive TypeScript types
- JSDoc comments for all public APIs
- Clear error messages with actionable guidance
- Consistent naming conventions

**Rationale:** Excellent developer experience accelerates development and reduces errors [REQ-WEB-088, REQ-WEB-089].

### 2.2. API Design Patterns

#### 2.2.1. Promise-Based Asynchronous Operations

All client operations return Promises for asynchronous execution, enabling modern async/await syntax.

**Pattern:**
```typescript
const document = await apiClient.getDocument("doc-123");
```

**Rationale:** Promise-based async provides clean, readable code and proper error handling.

#### 2.2.2. Configuration Object Pattern

Complex operations accept configuration objects for flexibility and readability.

**Pattern:**
```typescript
const documents = await apiClient.listDocuments({
  limit: 20,
  offset: 0,
  sortBy: "modified_at",
  sortOrder: "desc"
});
```

**Rationale:** Configuration objects provide flexibility while maintaining readability.

#### 2.2.3. Event Emitter Pattern

WebSocket client implements event emitter pattern for real-time event handling.

**Pattern:**
```typescript
wsClient.on("document_update", (event) => {
  console.log("Document updated:", event.payload);
});
```

**Rationale:** Event emitter pattern provides flexible real-time event handling.

#### 2.2.4. Builder Pattern

Complex queries use builder pattern for fluent, readable query construction.

**Pattern:**
```typescript
const results = await apiClient.searchDocuments()
  .withQuery("tachyon")
  .inRepository("repo-123")
  .withLimit(10)
  .execute();
```

**Rationale:** Builder pattern provides fluent, readable query construction.

### 2.3. Error Handling Philosophy

The client API follows a consistent error handling philosophy:

1. **Explicit Error Types:** All errors are typed with specific error codes and messages.
2. **Actionable Information:** Error messages provide actionable guidance for resolution.
3. **Context Preservation:** Errors include relevant context for debugging.
4. **Graceful Degradation:** The client handles errors gracefully without crashing.

**Rationale:** Consistent error handling philosophy improves reliability and developer experience.

---

## 3. CLIENT INITIALIZATION

### 3.1. ApiClient Configuration

The [`ApiClient`](.docs/api/web_client_api_specification.md) is the primary interface for HTTP/2 RESTful communication with the server. It provides type-safe methods for all server endpoints with built-in error handling, caching, and retry logic.

#### 3.1.1. Configuration Interface

```typescript
/**
 * Configuration options for ApiClient initialization.
 * 
 * @interface ApiClientConfig
 * @description Defines the configuration parameters for creating an ApiClient instance.
 */
export interface ApiClientConfig {
  /**
   * Base URL for the server API.
   * @type {string}
   * @description The base URL for all API requests. Must include protocol (http:// or https://).
   * @example "https://api.tachyon.example.com"
   */
  baseUrl: string;
  
  /**
   * Authentication token for API requests.
   * @type {string | null}
   * @description JWT or other authentication token. If null, requests are made without authentication.
   * @default null
   */
  authToken?: string | null;
  
  /**
   * Default timeout for API requests in milliseconds.
   * @type {number}
   * @description Maximum time to wait for a response before timing out.
   * @default 30000
   */
  timeout?: number;
  
  /**
   * Enable response caching for GET requests.
   * @type {boolean}
   * @description If true, GET responses are cached in memory with configurable TTL.
   * @default true
   */
  enableCache?: boolean;
  
  /**
   * Cache time-to-live in milliseconds.
   * @type {number}
   * @description How long cached responses remain valid before being refreshed.
   * @default 60000
   */
  cacheTTL?: number;
  
  /**
   * Enable automatic retry for failed requests.
   * @type {boolean}
   * @description If true, failed requests are automatically retried with exponential backoff.
   * @default true
   */
  enableRetry?: boolean;
  
  /**
   * Maximum number of retry attempts.
   * @type {number}
   * @description Maximum number of times to retry a failed request.
   * @default 3
   */
  maxRetries?: number;
  
  /**
   * Initial retry delay in milliseconds.
   * @type {number}
   * @description Base delay for exponential backoff retry logic.
   * @default 1000
   */
  retryDelay?: number;
  
  /**
   * Enable request deduplication.
   * @type {boolean}
   * @description If true, concurrent identical requests are deduplicated.
   * @default true
   */
  enableDeduplication?: boolean;
  
  /**
   * Custom headers to include in all requests.
   * @type {Record<string, string>}
   * @description Additional headers to include in all API requests.
   * @default {}
   */
  defaultHeaders?: Record<string, string>;
  
  /**
   * Request interceptor function.
   * @type {(config: RequestConfig) => RequestConfig | Promise<RequestConfig>}
   * @description Function to transform request configuration before sending.
   */
  requestInterceptor?: (config: RequestConfig) => RequestConfig | Promise<RequestConfig>;
  
  /**
   * Response interceptor function.
   * @type {(response: Response) => Response | Promise<Response>}
   * @description Function to transform response data before returning.
   */
  responseInterceptor?: (response: Response) => Response | Promise<Response>;
  
  /**
   * Error handler function.
   * @type {(error: ApiError) => void}
   * @description Function to handle API errors globally.
   */
  errorHandler?: (error: ApiError) => void;
  
  /**
   * Enable request logging for debugging.
   * @type {boolean}
   * @description If true, all requests and responses are logged to console.
   * @default false
   */
  enableLogging?: boolean;
  
  /**
   * Log function for request/response logging.
   * @type {(message: string, ...args: unknown[]) => void}
   * @description Custom log function. Defaults to console.log.
   * @default console.log
   */
  logFunction?: (message: string, ...args: unknown[]) => void;
}
```

#### 3.1.2. ApiClient Constructor

```typescript
/**
 * ApiClient - Type-safe HTTP client for server communication.
 * 
 * @class ApiClient
 * @description Provides type-safe methods for HTTP/2 RESTful communication with the server.
 * Includes built-in error handling, caching, retry logic, and request deduplication.
 */
export class ApiClient {
  private config: Required<ApiClientConfig>;
  private cache: Map<string, { data: unknown; timestamp: number }>;
  private pendingRequests: Map<string, Promise<unknown>>;
  private abortControllers: Map<string, AbortController>;
  
  /**
   * Creates a new ApiClient instance.
   * 
   * @constructor
   * @param {ApiClientConfig} config - Configuration options for the client.
   * @throws {TypeError} If baseUrl is not provided or is not a valid URL.
   * 
   * @example
   * ```typescript
   * const client = new ApiClient({
   *   baseUrl: "https://api.tachyon.example.com",
   *   authToken: "your-jwt-token",
   *   enableCache: true,
   *   cacheTTL: 60000
   * });
   * ```
   */
  constructor(config: ApiClientConfig) {
    // Validate baseUrl
    if (!config.baseUrl) {
      throw new TypeError("baseUrl is required");
    }
    
    try {
      new URL(config.baseUrl);
    } catch {
      throw new TypeError("baseUrl must be a valid URL");
    }
    
    // Merge with defaults
    this.config = {
      baseUrl: config.baseUrl,
      authToken: config.authToken ?? null,
      timeout: config.timeout ?? 30000,
      enableCache: config.enableCache ?? true,
      cacheTTL: config.cacheTTL ?? 60000,
      enableRetry: config.enableRetry ?? true,
      maxRetries: config.maxRetries ?? 3,
      retryDelay: config.retryDelay ?? 1000,
      enableDeduplication: config.enableDeduplication ?? true,
      defaultHeaders: config.defaultHeaders ?? {},
      requestInterceptor: config.requestInterceptor ?? ((c) => c),
      responseInterceptor: config.responseInterceptor ?? ((r) => r),
      errorHandler: config.errorHandler ?? ((e) => console.error("API Error:", e)),
      enableLogging: config.enableLogging ?? false,
      logFunction: config.logFunction ?? console.log
    };
    
    // Initialize internal state
    this.cache = new Map();
    this.pendingRequests = new Map();
    this.abortControllers = new Map();
    
    this.log("ApiClient initialized with config:", this.config);
  }
  
  /**
   * Internal logging method.
   * 
   * @private
   * @param {string} message - Log message.
   * @param {...unknown[]} args - Additional arguments to log.
   */
  private log(message: string, ...args: unknown[]): void {
    if (this.config.enableLogging) {
      this.config.logFunction(`[ApiClient] ${message}`, ...args);
    }
  }
  
  /**
   * Generates a unique cache key for a request.
   * 
   * @private
   * @param {string} method - HTTP method.
   * @param {string} url - Request URL.
   * @param {Record<string, unknown>} [params] - Query parameters.
   * @param {unknown} [body] - Request body.
   * @returns {string} Unique cache key.
   */
  private getCacheKey(
    method: string,
    url: string,
    params?: Record<string, unknown>,
    body?: unknown
  ): string {
    const paramsStr = params ? JSON.stringify(params) : "";
    const bodyStr = body ? JSON.stringify(body) : "";
    return `${method}:${url}:${paramsStr}:${bodyStr}`;
  }
  
  /**
   * Checks if a cached response is still valid.
   * 
   * @private
   * @param {number} timestamp - Cache timestamp.
   * @returns {boolean} True if cache is valid.
   */
  private isCacheValid(timestamp: number): boolean {
    return Date.now() - timestamp < this.config.cacheTTL;
  }
  
  /**
   * Makes an HTTP request with retry logic and error handling.
   * 
   * @private
   * @template T - Expected response type.
   * @param {string} method - HTTP method.
   * @param {string} endpoint - API endpoint.
   * @param {RequestOptions} options - Request options.
   * @returns {Promise<T>} Response data.
   * @throws {ApiError} If request fails after all retries.
   */
  private async request<T>(
    method: string,
    endpoint: string,
    options: RequestOptions = {}
  ): Promise<T> {
    const url = new URL(endpoint, this.config.baseUrl);
    
    // Add query parameters
    if (options.params) {
      Object.entries(options.params).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          url.searchParams.append(key, String(value));
        }
      });
    }
    
    // Check cache for GET requests
    const cacheKey = this.getCacheKey(method, url.toString(), options.params, options.body);
    if (method === "GET" && this.config.enableCache) {
      const cached = this.cache.get(cacheKey);
      if (cached && this.isCacheValid(cached.timestamp)) {
        this.log("Cache hit for:", cacheKey);
        return cached.data as T;
      }
    }
    
    // Check for deduplication
    if (this.config.enableDeduplication) {
      const pending = this.pendingRequests.get(cacheKey);
      if (pending) {
        this.log("Deduplicating request for:", cacheKey);
        return pending as Promise<T>;
      }
    }
    
    // Create AbortController for timeout
    const abortController = new AbortController();
    const timeoutId = setTimeout(() => abortController.abort(), this.config.timeout);
    this.abortControllers.set(cacheKey, abortController);
    
    // Build request headers
    const headers: HeadersInit = {
      "Content-Type": "application/json",
      ...this.config.defaultHeaders,
      ...options.headers
    };
    
    // Add authentication token
    if (this.config.authToken) {
      headers["Authorization"] = `Bearer ${this.config.authToken}`;
    }
    
    // Build request config
    const requestConfig: RequestConfig = {
      method,
      url: url.toString(),
      headers,
      body: options.body ? JSON.stringify(options.body) : undefined,
      signal: abortController.signal
    };
    
    // Apply request interceptor
    const finalConfig = await this.config.requestInterceptor(requestConfig);
    
    // Create request promise
    const requestPromise = this.executeRequestWithRetry<T>(finalConfig, cacheKey);
    
    // Store for deduplication
    if (this.config.enableDeduplication) {
      this.pendingRequests.set(cacheKey, requestPromise);
    }
    
    try {
      const result = await requestPromise;
      return result;
    } finally {
      // Cleanup
      clearTimeout(timeoutId);
      this.abortControllers.delete(cacheKey);
      this.pendingRequests.delete(cacheKey);
    }
  }
  
  /**
   * Executes a request with retry logic.
   * 
   * @private
   * @template T - Expected response type.
   * @param {RequestConfig} config - Request configuration.
   * @param {string} cacheKey - Cache key for the request.
   * @returns {Promise<T>} Response data.
   */
  private async executeRequestWithRetry<T>(
    config: RequestConfig,
    cacheKey: string
  ): Promise<T> {
    let lastError: Error | null = null;
    
    for (let attempt = 0; attempt <= this.config.maxRetries; attempt++) {
      try {
        this.log(`Request attempt ${attempt + 1}:`, config.method, config.url);
        
        const response = await fetch(config.url, {
          method: config.method,
          headers: config.headers,
          body: config.body,
          signal: config.signal
        });
        
        // Apply response interceptor
        const processedResponse = await this.config.responseInterceptor(response);
        
        // Handle non-success responses
        if (!processedResponse.ok) {
          const error = await this.handleError(processedResponse);
          throw error;
        }
        
        // Parse response
        const data = await processedResponse.json();
        
        // Cache GET responses
        if (config.method === "GET" && this.config.enableCache) {
          this.cache.set(cacheKey, { data, timestamp: Date.now() });
        }
        
        this.log("Request successful:", config.method, config.url);
        return data as T;
        
      } catch (error) {
        lastError = error as Error;
        
        // Don't retry on abort or certain errors
        if (error instanceof DOMException && error.name === "AbortError") {
          throw new ApiError({
            code: "TIMEOUT",
            message: "Request timed out",
            details: { url: config.url, timeout: this.config.timeout },
            timestamp: Date.now(),
            requestId: this.generateRequestId()
          });
        }
        
        // Retry with exponential backoff
        if (attempt < this.config.maxRetries && this.config.enableRetry) {
          const delay = this.config.retryDelay * Math.pow(2, attempt);
          this.log(`Retrying in ${delay}ms...`);
          await this.sleep(delay);
        }
      }
    }
    
    // All retries failed
    throw lastError || new ApiError({
      code: "UNKNOWN_ERROR",
      message: "Request failed after all retries",
      timestamp: Date.now(),
      requestId: this.generateRequestId()
    });
  }
  
  /**
   * Handles API error responses.
   * 
   * @private
   * @param {Response} response - Fetch response object.
   * @returns {Promise<ApiError>} Parsed API error.
   */
  private async handleError(response: Response): Promise<ApiError> {
    let errorData: unknown;
    
    try {
      errorData = await response.json();
    } catch {
      errorData = null;
    }
    
    const error: ApiError = new ApiError({
      code: (errorData as { code?: string })?.code || "HTTP_ERROR",
      message: (errorData as { message?: string })?.message || response.statusText,
      details: (errorData as { details?: Record<string, unknown> })?.details,
      timestamp: Date.now(),
      requestId: this.generateRequestId(),
      statusCode: response.status
    });
    
    // Call error handler
    this.config.errorHandler(error);
    
    return error;
  }
  
  /**
   * Sleep utility for retry delays.
   * 
   * @private
   * @param {number} ms - Milliseconds to sleep.
   * @returns {Promise<void>} Promise that resolves after delay.
   */
  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
  
  /**
   * Generates a unique request ID.
   * 
   * @private
   * @returns {string} Unique request ID.
   */
  private generateRequestId(): string {
    return `req_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
  
  /**
   * Cancels all pending requests.
   * 
   * @public
   * @description Aborts all in-flight requests using their AbortControllers.
   */
  public cancelAllRequests(): void {
    this.log("Cancelling all requests");
    this.abortControllers.forEach(controller => controller.abort());
    this.abortControllers.clear();
  }
  
  /**
   * Clears the response cache.
   * 
   * @public
   * @description Removes all cached responses from memory.
   */
  public clearCache(): void {
    this.log("Clearing cache");
    this.cache.clear();
  }
  
  /**
   * Updates the authentication token.
   * 
   * @public
   * @param {string | null} token - New authentication token.
   * @description Updates the authentication token used for subsequent requests.
   */
  public setAuthToken(token: string | null): void {
    this.config.authToken = token;
    this.log("Auth token updated");
  }
  
  /**
   * Gets the current authentication token.
   * 
   * @public
   * @returns {string | null} Current authentication token.
   */
  public getAuthToken(): string | null {
    return this.config.authToken;
  }
}

/**
 * Request configuration interface.
 * 
 * @interface RequestConfig
 * @description Configuration for HTTP requests.
 */
export interface RequestConfig {
  /** HTTP method. */
  method: string;
  /** Request URL. */
  url: string;
  /** Request headers. */
  headers: HeadersInit;
  /** Request body. */
  body?: string;
  /** Abort signal for cancellation. */
  signal?: AbortSignal;
}

/**
 * Request options interface.
 * 
 * @interface RequestOptions
 * @description Options for API requests.
 */
export interface RequestOptions {
  /** Query parameters. */
  params?: Record<string, unknown>;
  /** Request body. */
  body?: unknown;
  /** Additional headers. */
  headers?: Record<string, string>;
}
```

### 3.2. WebSocketClient Configuration

The [`WebSocketClient`](.docs/api/web_client_api_specification.md) provides a type-safe interface for WebSocket communication with the server. It handles connection management, automatic reconnection, message queuing, and event subscriptions.

#### 3.2.1. Configuration Interface

```typescript
/**
 * Configuration options for WebSocketClient initialization.
 * 
 * @interface WebSocketClientConfig
 * @description Defines the configuration parameters for creating a WebSocketClient instance.
 */
export interface WebSocketClientConfig {
  /**
   * WebSocket server URL.
   * @type {string}
   * @description The WebSocket server URL. Must use ws:// or wss:// protocol.
   * @example "wss://api.tachyon.example.com/ws"
   */
  url: string;
  
  /**
   * Authentication token for WebSocket connection.
   * @type {string | null}
   * @description JWT or other authentication token for the WebSocket connection.
   * @default null
   */
  authToken?: string | null;
  
  /**
   * Enable automatic reconnection.
   * @type {boolean}
   * @description If true, automatically reconnect on disconnection.
   * @default true
   */
  autoReconnect?: boolean;
  
  /**
   * Maximum reconnection attempts.
   * @type {number}
   * @description Maximum number of reconnection attempts before giving up.
   * @default 10
   */
  maxReconnectAttempts?: number;
  
  /**
   * Initial reconnection delay in milliseconds.
   * @type {number}
   * @description Base delay for exponential backoff reconnection.
   * @default 1000
   */
  reconnectDelay?: number;
  
  /**
   * Maximum reconnection delay in milliseconds.
   * @type {number}
   * @description Maximum delay between reconnection attempts.
   * @default 30000
   */
  maxReconnectDelay?: number;
  
  /**
   * Enable message queuing during disconnection.
   * @type {boolean}
   * @description If true, messages sent while disconnected are queued and sent on reconnection.
   * @default true
   */
  enableMessageQueue?: boolean;
  
  /**
   * Maximum queue size.
   * @type {number}
   * @description Maximum number of messages to queue while disconnected.
   * @default 100
   */
  maxQueueSize?: number;
  
  /**
   * Heartbeat interval in milliseconds.
   * @type {number}
   * @description Interval for sending ping messages to keep connection alive.
   * @default 30000
   */
  heartbeatInterval?: number;
  
  /**
   * Connection timeout in milliseconds.
   * @type {number}
   * @description Maximum time to wait for connection before timing out.
   * @default 10000
   */
  connectionTimeout?: number;
  
  /**
   * Custom headers for WebSocket connection.
   * @type {Record<string, string>}
   * @description Additional headers to include in WebSocket connection.
   * @default {}
   */
  headers?: Record<string, string>;
  
  /**
   * Enable connection logging for debugging.
   * @type {boolean}
   * @description If true, all WebSocket events are logged to console.
   * @default false
   */
  enableLogging?: boolean;
  
  /**
   * Log function for WebSocket logging.
   * @type {(message: string, ...args: unknown[]) => void}
   * @description Custom log function. Defaults to console.log.
   * @default console.log
   */
  logFunction?: (message: string, ...args: unknown[]) => void;
}
```

#### 3.2.2. WebSocketClient Constructor

```typescript
/**
 * WebSocketClient - Type-safe WebSocket client for real-time communication.
 * 
 * @class WebSocketClient
 * @description Provides type-safe WebSocket communication with automatic reconnection,
 * message queuing, and event subscription management.
 */
export class WebSocketClient {
  private config: Required<WebSocketClientConfig>;
  private ws: WebSocket | null = null;
  private connectionState: ConnectionState = "disconnected";
  private reconnectAttempts = 0;
  private reconnectTimer: number | null = null;
  private heartbeatTimer: number | null = null;
  private messageQueue: QueuedMessage[] = [];
  private eventHandlers: Map<EventType, EventHandler[]>;
  private subscriptions: Map<string, Subscription>;
  private pendingSubscriptions: Map<string, PendingSubscription>;
  private messageIdCounter = 0;
  
  /**
   * Creates a new WebSocketClient instance.
   * 
   * @constructor
   * @param {WebSocketClientConfig} config - Configuration options for the client.
   * @throws {TypeError} If url is not provided or is not a valid WebSocket URL.
   * 
   * @example
   * ```typescript
   * const wsClient = new WebSocketClient({
   *   url: "wss://api.tachyon.example.com/ws",
   *   authToken: "your-jwt-token",
   *   autoReconnect: true
   * });
   * ```
   */
  constructor(config: WebSocketClientConfig) {
    // Validate URL
    if (!config.url) {
      throw new TypeError("url is required");
    }
    
    try {
      const url = new URL(config.url);
      if (url.protocol !== "ws:" && url.protocol !== "wss:") {
        throw new TypeError("url must use ws:// or wss:// protocol");
      }
    } catch {
      throw new TypeError("url must be a valid WebSocket URL");
    }
    
    // Merge with defaults
    this.config = {
      url: config.url,
      authToken: config.authToken ?? null,
      autoReconnect: config.autoReconnect ?? true,
      maxReconnectAttempts: config.maxReconnectAttempts ?? 10,
      reconnectDelay: config.reconnectDelay ?? 1000,
      maxReconnectDelay: config.maxReconnectDelay ?? 30000,
      enableMessageQueue: config.enableMessageQueue ?? true,
      maxQueueSize: config.maxQueueSize ?? 100,
      heartbeatInterval: config.heartbeatInterval ?? 30000,
      connectionTimeout: config.connectionTimeout ?? 10000,
      headers: config.headers ?? {},
      enableLogging: config.enableLogging ?? false,
      logFunction: config.logFunction ?? console.log
    };
    
    // Initialize internal state
    this.eventHandlers = new Map();
    this.subscriptions = new Map();
    this.pendingSubscriptions = new Map();
    
    this.log("WebSocketClient initialized with config:", this.config);
  }
  
  /**
   * Internal logging method.
   * 
   * @private
   * @param {string} message - Log message.
   * @param {...unknown[]} args - Additional arguments to log.
   */
  private log(message: string, ...args: unknown[]): void {
    if (this.config.enableLogging) {
      this.config.logFunction(`[WebSocketClient] ${message}`, ...args);
    }
  }
  
  /**
   * Generates a unique message ID.
   * 
   * @private
   * @returns {string} Unique message ID.
   */
  private generateMessageId(): string {
    return `msg_${Date.now()}_${++this.messageIdCounter}`;
  }
  
  /**
   * Calculates reconnection delay with exponential backoff.
   * 
   * @private
   * @returns {number} Delay in milliseconds.
   */
  private getReconnectDelay(): number {
    const delay = this.config.reconnectDelay * Math.pow(2, this.reconnectAttempts);
    return Math.min(delay, this.config.maxReconnectDelay);
  }
  
  /**
   * Starts the heartbeat timer.
   * 
   * @private
   */
  private startHeartbeat(): void {
    this.stopHeartbeat();
    this.heartbeatTimer = window.setInterval(() => {
      if (this.connectionState === "connected") {
        this.sendPing();
      }
    }, this.config.heartbeatInterval);
  }
  
  /**
   * Stops the heartbeat timer.
   * 
   * @private
   */
  private stopHeartbeat(): void {
    if (this.heartbeatTimer !== null) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
  }
  
  /**
   * Sends a ping message to the server.
   * 
   * @private
   */
  private sendPing(): void {
    const message: WebSocketMessage = {
      type: "ping",
      payload: {},
      timestamp: Date.now(),
      messageId: this.generateMessageId()
    };
    this.sendRaw(message);
  }
  
  /**
   * Sends a raw WebSocket message.
   * 
   * @private
   * @param {WebSocketMessage} message - Message to send.
   */
  private sendRaw(message: WebSocketMessage): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
      this.log("Sent message:", message);
    } else {
      this.log("Cannot send message, WebSocket not connected");
    }
  }
  
  /**
   * Processes a received WebSocket message.
   * 
   * @private
   * @param {WebSocketMessage} message - Received message.
   */
  private processMessage(message: WebSocketMessage): void {
    this.log("Received message:", message);
    
    // Handle special message types
    switch (message.type) {
      case "pong":
        // Heartbeat response, no action needed
        break;
      
      case "subscription_ack":
        this.handleSubscriptionAck(message);
        break;
      
      case "subscription_error":
        this.handleSubscriptionError(message);
        break;
      
      default:
        // Emit to event handlers
        this.emit(message.type, message);
    }
  }
  
  /**
   * Handles subscription acknowledgment.
   * 
   * @private
   * @param {WebSocketMessage} message - Subscription acknowledgment message.
   */
  private handleSubscriptionAck(message: WebSocketMessage): void {
    const ack = message.payload as SubscriptionAck;
    
    // Remove from pending
    this.pendingSubscriptions.delete(ack.subscriptionId);
    
    // Add to active subscriptions
    this.subscriptions.set(ack.subscriptionId, {
      id: ack.subscriptionId,
      channel: ack.channel,
      createdAt: Date.now(),
      lastEventAt: null
    });
    
    this.log("Subscription acknowledged:", ack.subscriptionId);
  }
  
  /**
   * Handles subscription error.
   * 
   * @private
   * @param {WebSocketMessage} message - Subscription error message.
   */
  private handleSubscriptionError(message: WebSocketMessage): void {
    const error = message.payload as SubscriptionError;
    
    // Remove from pending
    this.pendingSubscriptions.delete(error.subscriptionId);
    
    this.log("Subscription error:", error);
    
    // Emit error event
    this.emit("subscription_error", message);
  }
  
  /**
   * Emits an event to all registered handlers.
   * 
   * @private
   * @param {EventType} eventType - Event type.
   * @param {WebSocketMessage} message - Event message.
   */
  private emit(eventType: EventType, message: WebSocketMessage): void {
    const handlers = this.eventHandlers.get(eventType);
    if (handlers) {
      handlers.forEach(handler => {
        try {
          handler(message);
        } catch (error) {
          this.log("Error in event handler:", error);
        }
      });
    }
  }
  
  /**
   * Flushes the message queue.
   * 
   * @private
   */
  private flushMessageQueue(): void {
    if (this.messageQueue.length === 0) {
      return;
    }
    
    this.log("Flushing message queue:", this.messageQueue.length, "messages");
    
    const queue = [...this.messageQueue];
    this.messageQueue = [];
    
    queue.forEach(queued => {
      this.sendRaw(queued.message);
    });
  }
  
  /**
   * Attempts to reconnect to the WebSocket server.
   * 
   * @private
   */
  private reconnect(): void {
    if (!this.config.autoReconnect) {
      this.log("Auto-reconnect disabled, not reconnecting");
      return;
    }
    
    if (this.reconnectAttempts >= this.config.maxReconnectAttempts) {
      this.log("Max reconnection attempts reached, giving up");
      this.connectionState = "disconnected";
      this.emit("connection_failed", {
        type: "connection_failed",
        payload: { attempts: this.reconnectAttempts },
        timestamp: Date.now(),
        messageId: this.generateMessageId()
      });
      return;
    }
    
    this.reconnectAttempts++;
    const delay = this.getReconnectDelay();
    
    this.log(`Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);
    
    this.reconnectTimer = window.setTimeout(() => {
      this.connect().catch(error => {
        this.log("Reconnection failed:", error);
      });
    }, delay);
  }
  
  /**
   * Gets the current connection state.
   * 
   * @public
   * @returns {ConnectionState} Current connection state.
   */
  public getConnectionState(): ConnectionState {
    return this.connectionState;
  }
}

/**
 * Connection state type.
 * 
 * @type ConnectionState
 * @description Possible WebSocket connection states.
 */
export type ConnectionState =
  | "disconnected"
  | "connecting"
  | "connected"
  | "reconnecting"
  | "disconnecting";

/**
 * Event type for WebSocket messages.
 * 
 * @type EventType
 * @description Possible WebSocket event types.
 */
export type EventType =
  | "connect"
  | "disconnect"
  | "reconnect"
  | "connection_failed"
  | "document_update"
  | "repository_update"
  | "user_presence"
  | "sync_status"
  | "subscription_error"
  | "error";

/**
 * Event handler function type.
 * 
 * @type EventHandler
 * @description Function to handle WebSocket events.
 */
export type EventHandler = (message: WebSocketMessage) => void;

/**
 * Subscription interface.
 * 
 * @interface Subscription
 * @description Active subscription information.
 */
export interface Subscription {
  /** Subscription ID. */
  id: string;
  /** Subscription channel. */
  channel: string;
  /** Creation timestamp. */
  createdAt: number;
  /** Last event timestamp. */
  lastEventAt: number | null;
}

/**
 * Queued message interface.
 * 
 * @interface QueuedMessage
 * @description Message queued while disconnected.
 */
export interface QueuedMessage {
  /** Message to send. */
  message: WebSocketMessage;
  /** Queue timestamp. */
  timestamp: number;
}

/**
 * Pending subscription interface.
 * 
 * @interface PendingSubscription
 * @description Subscription waiting for acknowledgment.
 */
export interface PendingSubscription {
  /** Subscription ID. */
  id: string;
  /** Subscription channel. */
  channel: string;
  /** Callback for acknowledgment. */
  callback: (error: Error | null) => void;
}

/**
 * Subscription acknowledgment payload.
 * 
 * @interface SubscriptionAck
 * @description Subscription acknowledgment from server.
 */
export interface SubscriptionAck {
  /** Subscription ID. */
  subscriptionId: string;
  /** Subscription channel. */
  channel: string;
}

/**
 * Subscription error payload.
 * 
 * @interface SubscriptionError
 * @description Subscription error from server.
 */
export interface SubscriptionError {
  /** Subscription ID. */
  subscriptionId: string;
  /** Error code. */
  code: string;
  /** Error message. */
  message: string;
}

/**
 * WebSocket message interface.
 * 
 * @interface WebSocketMessage
 * @description Standard WebSocket message format.
 */
export interface WebSocketMessage {
  /** Message type. */
  type: string;
  /** Message payload. */
  payload: unknown;
  /** Message timestamp. */
  timestamp: number;
  /** Unique message ID. */
  messageId: string;
}

---

## 4. HTTP CLIENT METHODS

### 4.1. Document API Methods

The Document API provides methods for managing documents in the Tachyon system. All methods return type-safe responses with comprehensive error handling.

#### 4.1.1. Document Type Definitions

```typescript
/**
 * Document entity.
 * 
 * @interface Document
 * @description Represents a document in the Tachyon system.
 */
export interface Document {
  /** Unique document identifier. */
  id: string;
  /** Document title. */
  title: string;
  /** Document content (Markdown). */
  content: string;
  /** Repository ID. */
  repositoryId: string;
  /** Document path (relative to repository). */
  path: string;
  /** Document language/format. */
  language: string;
  /** Creation timestamp (ISO 8601). */
  createdAt: string;
  /** Last modification timestamp (ISO 8601). */
  modifiedAt: string;
  /** Creator user ID. */
  createdBy: string;
  /** Last modifier user ID. */
  modifiedBy: string;
  /** Document tags. */
  tags: string[];
  /** Document metadata. */
  metadata: Record<string, unknown>;
  /** Document status. */
  status: DocumentStatus;
  /** Word count. */
  wordCount: number;
  /** Character count. */
  characterCount: number;
}

/**
 * Document status enumeration.
 * 
 * @type DocumentStatus
 * @description Possible document statuses.
 */
export type DocumentStatus =
  | "draft"
  | "published"
  | "archived"
  | "deleted";

/**
 * Document history entry.
 * 
 * @interface DocumentHistoryEntry
 * @description Represents a document version in history.
 */
export interface DocumentHistoryEntry {
  /** Version identifier. */
  versionId: string;
  /** Document ID. */
  documentId: string;
  /** Version number. */
  versionNumber: number;
  /** Document content at this version. */
  content: string;
  /** Change summary. */
  summary: string;
  /** Author user ID. */
  authorId: string;
  /** Creation timestamp (ISO 8601). */
  createdAt: string;
  /** Changes made in this version. */
  changes: DocumentChange[];
}

/**
 * Document change type.
 * 
 * @interface DocumentChange
 * @description Represents a change in document version.
 */
export interface DocumentChange {
  /** Type of change. */
  type: "insert" | "delete" | "replace";
  /** Position of change. */
  position: number;
  /** Length of change. */
  length: number;
  /** Content that was added. */
  added: string;
  /** Content that was removed. */
  removed: string;
}

/**
 * List documents options.
 * 
 * @interface ListDocumentsOptions
 * @description Options for listing documents.
 */
export interface ListDocumentsOptions {
  /** Repository ID to filter by. */
  repositoryId?: string;
  /** Search query. */
  query?: string;
  /** Tag filter. */
  tag?: string;
  /** Status filter. */
  status?: DocumentStatus;
  /** Language filter. */
  language?: string;
  /** Maximum number of results. */
  limit?: number;
  /** Offset for pagination. */
  offset?: number;
  /** Sort field. */
  sortBy?: SortField;
  /** Sort order. */
  sortOrder?: "asc" | "desc";
}

/**
 * Sort field enumeration.
 * 
 * @type SortField
 * @description Fields available for sorting.
 */
export type SortField =
  | "title"
  | "createdAt"
  | "modifiedAt"
  | "wordCount"
  | "characterCount";

/**
 * List documents response.
 * 
 * @interface ListDocumentsResponse
 * @description Response from list documents endpoint.
 */
export interface ListDocumentsResponse {
  /** Array of documents. */
  documents: Document[];
  /** Total count (for pagination). */
  total: number;
  /** Current page offset. */
  offset: number;
  /** Page size (limit). */
  limit: number;
}

/**
 * Create document options.
 * 
 * @interface CreateDocumentOptions
 * @description Options for creating a document.
 */
export interface CreateDocumentOptions {
  /** Document title. */
  title: string;
  /** Document content (Markdown). */
  content: string;
  /** Repository ID. */
  repositoryId: string;
  /** Document path (relative to repository). */
  path?: string;
  /** Document language/format. */
  language?: string;
  /** Document tags. */
  tags?: string[];
  /** Document metadata. */
  metadata?: Record<string, unknown>;
}

/**
 * Update document options.
 * 
 * @interface UpdateDocumentOptions
 * @description Options for updating a document.
 */
export interface UpdateDocumentOptions {
  /** Document title. */
  title?: string;
  /** Document content (Markdown). */
  content?: string;
  /** Document path (relative to repository). */
  path?: string;
  /** Document language/format. */
  language?: string;
  /** Document tags. */
  tags?: string[];
  /** Document metadata. */
  metadata?: Record<string, unknown>;
  /** Document status. */
  status?: DocumentStatus;
  /** Change summary for history. */
  summary?: string;
}

/**
 * Document history options.
 * 
 * @interface DocumentHistoryOptions
 * @description Options for retrieving document history.
 */
export interface DocumentHistoryOptions {
  /** Maximum number of versions to return. */
  limit?: number;
  /** Offset for pagination. */
  offset?: number;
  /** Include full content. */
  includeContent?: boolean;
}
```

#### 4.1.2. Document API Methods

```typescript
/**
 * ApiClient Document API methods.
 * 
 * @description Methods for document management operations.
 */
export class ApiClient {
  // ... (previous constructor and methods)

  /**
   * Retrieves a document by ID.
   * 
   * @public
   * @param {string} documentId - Document ID to retrieve.
   * @returns {Promise<Document>} Document data.
   * @throws {ApiError} If document not found or access denied.
   * 
   * @example
   * ```typescript
   * const document = await apiClient.getDocument("doc-123");
   * console.log(document.title, document.content);
   * ```
   */
  public async getDocument(documentId: string): Promise<Document> {
    this.validateDocumentId(documentId);
    return this.request<Document>("GET", `/api/documents/${documentId}`);
  }

  /**
   * Lists documents with optional filtering and pagination.
   * 
   * @public
   * @param {ListDocumentsOptions} [options] - List options.
   * @returns {Promise<ListDocumentsResponse>} List of documents.
   * @throws {ApiError} If request fails.
   * 
   * @example
   * ```typescript
   * const result = await apiClient.listDocuments({
   *   repositoryId: "repo-123",
   *   limit: 20,
   *   sortBy: "modifiedAt",
   *   sortOrder: "desc"
   * });
   * console.log(result.documents);
   * ```
   */
  public async listDocuments(
    options: ListDocumentsOptions = {}
  ): Promise<ListDocumentsResponse> {
    return this.request<ListDocumentsResponse>("GET", "/api/documents", {
      params: options
    });
  }

  /**
   * Creates a new document.
   * 
   * @public
   * @param {CreateDocumentOptions} options - Document creation options.
   * @returns {Promise<Document>} Created document data.
   * @throws {ApiError} If creation fails.
   * 
   * @example
   * ```typescript
   * const document = await apiClient.createDocument({
   *   title: "My Document",
   *   content: "# Hello World",
   *   repositoryId: "repo-123",
   *   language: "markdown"
   * });
   * ```
   */
  public async createDocument(
    options: CreateDocumentOptions
  ): Promise<Document> {
    this.validateCreateDocumentOptions(options);
    return this.request<Document>("POST", "/api/documents", { body: options });
  }

  /**
   * Updates an existing document.
   * 
   * @public
   * @param {string} documentId - Document ID to update.
   * @param {UpdateDocumentOptions} options - Update options.
   * @returns {Promise<Document>} Updated document data.
   * @throws {ApiError} If update fails.
   * 
   * @example
   * ```typescript
   * const document = await apiClient.updateDocument("doc-123", {
   *   title: "Updated Title",
   *   content: "# Updated Content",
   *   summary: "Updated title and content"
   * });
   * ```
   */
  public async updateDocument(
    documentId: string,
    options: UpdateDocumentOptions
  ): Promise<Document> {
    this.validateDocumentId(documentId);
    this.validateUpdateDocumentOptions(options);
    return this.request<Document>("PUT", `/api/documents/${documentId}`, {
      body: options
    });
  }

  /**
   * Deletes a document.
   * 
   * @public
   * @param {string} documentId - Document ID to delete.
   * @returns {Promise<void>} Resolves when deleted.
   * @throws {ApiError} If deletion fails.
   * 
   * @example
   * ```typescript
   * await apiClient.deleteDocument("doc-123");
   * ```
   */
  public async deleteDocument(documentId: string): Promise<void> {
    this.validateDocumentId(documentId);
    return this.request<void>("DELETE", `/api/documents/${documentId}`);
  }

  /**
   * Retrieves document history/versions.
   * 
   * @public
   * @param {string} documentId - Document ID.
   * @param {DocumentHistoryOptions} [options] - History options.
   * @returns {Promise<DocumentHistoryEntry[]>} Document history.
   * @throws {ApiError} If retrieval fails.
   * 
   * @example
   * ```typescript
   * const history = await apiClient.getDocumentHistory("doc-123", {
   *   limit: 10,
   *   includeContent: true
   * });
   * ```
   */
  public async getDocumentHistory(
    documentId: string,
    options: DocumentHistoryOptions = {}
  ): Promise<DocumentHistoryEntry[]> {
    this.validateDocumentId(documentId);
    return this.request<DocumentHistoryEntry[]>(
      "GET",
      `/api/documents/${documentId}/history`,
      { params: options }
    );
  }

  /**
   * Restores a document to a previous version.
   * 
   * @public
   * @param {string} documentId - Document ID.
   * @param {string} versionId - Version ID to restore.
   * @returns {Promise<Document>} Restored document data.
   * @throws {ApiError} If restoration fails.
   * 
   * @example
   * ```typescript
   * const document = await apiClient.restoreDocument("doc-123", "ver-456");
   * ```
   */
  public async restoreDocument(
    documentId: string,
    versionId: string
  ): Promise<Document> {
    this.validateDocumentId(documentId);
    this.validateVersionId(versionId);
    return this.request<Document>(
      "POST",
      `/api/documents/${documentId}/restore/${versionId}`
    );
  }

  /**
   * Validates document ID format.
   * 
   * @private
   * @param {string} documentId - Document ID to validate.
   * @throws {ValidationError} If invalid.
   */
  private validateDocumentId(documentId: string): void {
    if (!documentId || typeof documentId !== "string") {
      throw new ValidationError("Document ID is required and must be a string");
    }
    if (documentId.length < 1 || documentId.length > 100) {
      throw new ValidationError("Document ID must be between 1 and 100 characters");
    }
  }

  /**
   * Validates create document options.
   * 
   * @private
   * @param {CreateDocumentOptions} options - Options to validate.
   * @throws {ValidationError} If invalid.
   */
  private validateCreateDocumentOptions(options: CreateDocumentOptions): void {
    if (!options.title || typeof options.title !== "string") {
      throw new ValidationError("Title is required and must be a string");
    }
    if (options.title.length < 1 || options.title.length > 500) {
      throw new ValidationError("Title must be between 1 and 500 characters");
    }
    if (!options.content || typeof options.content !== "string") {
      throw new ValidationError("Content is required and must be a string");
    }
    if (!options.repositoryId || typeof options.repositoryId !== "string") {
      throw new ValidationError("Repository ID is required and must be a string");
    }
  }

  /**
   * Validates update document options.
   * 
   * @private
   * @param {UpdateDocumentOptions} options - Options to validate.
   * @throws {ValidationError} If invalid.
   */
  private validateUpdateDocumentOptions(options: UpdateDocumentOptions): void {
    if (options.title !== undefined && typeof options.title !== "string") {
      throw new ValidationError("Title must be a string if provided");
    }
    if (options.content !== undefined && typeof options.content !== "string") {
      throw new ValidationError("Content must be a string if provided");
    }
  }

  /**
   * Validates version ID format.
   * 
   * @private
   * @param {string} versionId - Version ID to validate.
   * @throws {ValidationError} If invalid.
   */
  private validateVersionId(versionId: string): void {
    if (!versionId || typeof versionId !== "string") {
      throw new ValidationError("Version ID is required and must be a string");
    }
  }
}

/**
 * Validation error.
 * 
 * @class ValidationError
 * @description Error for validation failures.
 */
export class ValidationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ValidationError";
  }
}
```

### 4.2. Repository API Methods

The Repository API provides methods for managing repositories in the Tachyon system.

#### 4.2.1. Repository Type Definitions

```typescript
/**
 * Repository entity.
 * 
 * @interface Repository
 * @description Represents a repository in the Tachyon system.
 */
export interface Repository {
  /** Unique repository identifier. */
  id: string;
  /** Repository name. */
  name: string;
  /** Repository description. */
  description: string;
  /** Repository type. */
  type: RepositoryType;
  /** Repository URL (for remote repositories). */
  url?: string;
  /** Local repository path. */
  localPath: string;
  /** Default branch name. */
  defaultBranch: string;
  /** Current branch name. */
  currentBranch: string;
  /** Creation timestamp (ISO 8601). */
  createdAt: string;
  /** Last sync timestamp (ISO 8601). */
  lastSyncAt?: string;
  /** Owner user ID. */
  ownerId: string;
  /** Repository status. */
  status: RepositoryStatus;
  /** Repository settings. */
  settings: RepositorySettings;
  /** Statistics. */
  stats: RepositoryStats;
}

/**
 * Repository type enumeration.
 * 
 * @type RepositoryType
 * @description Possible repository types.
 */
export type RepositoryType =
  | "local"
  | "git"
  | "svn"
  | "mercurial";

/**
 * Repository status enumeration.
 * 
 * @type RepositoryStatus
 * @description Possible repository statuses.
 */
export type RepositoryStatus =
  | "active"
  | "syncing"
  | "error"
  | "disconnected";

/**
 * Repository settings.
 * 
 * @interface RepositorySettings
 * @description Repository configuration settings.
 */
export interface RepositorySettings {
  /** Auto-sync enabled. */
  autoSync: boolean;
  /** Sync interval in seconds. */
  syncInterval: number;
  /** Auto-pull enabled. */
  autoPull: boolean;
  /** Auto-push enabled. */
  autoPush: boolean;
  /** Conflict resolution strategy. */
  conflictResolution: ConflictResolutionStrategy;
  /** Enable indexing. */
  enableIndexing: boolean;
  /** Indexing schedule. */
  indexingSchedule?: string;
}

/**
 * Conflict resolution strategy.
 * 
 * @type ConflictResolutionStrategy
 * @description Strategies for resolving sync conflicts.
 */
export type ConflictResolutionStrategy =
  | "local_wins"
  | "remote_wins"
  | "manual"
  | "merge";

/**
 * Repository statistics.
 * 
 * @interface RepositoryStats
 * @description Repository usage statistics.
 */
export interface RepositoryStats {
  /** Total documents. */
  documentCount: number;
  /** Total size in bytes. */
  totalSize: number;
  /** Last indexed timestamp. */
  lastIndexedAt?: string;
  /** Branch count. */
  branchCount: number;
  /** Commit count. */
  commitCount: number;
}

/**
 * List repositories options.
 * 
 * @interface ListRepositoriesOptions
 * @description Options for listing repositories.
 */
export interface ListRepositoriesOptions {
  /** Repository type filter. */
  type?: RepositoryType;
  /** Status filter. */
  status?: RepositoryStatus;
  /** Search query. */
  query?: string;
  /** Maximum number of results. */
  limit?: number;
  /** Offset for pagination. */
  offset?: number;
  /** Sort field. */
  sortBy?: "name" | "createdAt" | "lastSyncAt";
  /** Sort order. */
  sortOrder?: "asc" | "desc";
}

/**
 * List repositories response.
 * 
 * @interface ListRepositoriesResponse
 * @description Response from list repositories endpoint.
 */
export interface ListRepositoriesResponse {
  /** Array of repositories. */
  repositories: Repository[];
  /** Total count (for pagination). */
  total: number;
  /** Current page offset. */
  offset: number;
  /** Page size (limit). */
  limit: number;
}

/**
 * Create repository options.
 * 
 * @interface CreateRepositoryOptions
 * @description Options for creating a repository.
 */
export interface CreateRepositoryOptions {
  /** Repository name. */
  name: string;
  /** Repository description. */
  description?: string;
  /** Repository type. */
  type: RepositoryType;
  /** Repository URL (for remote repositories). */
  url?: string;
  /** Local repository path. */
  localPath: string;
  /** Default branch name. */
  defaultBranch?: string;
  /** Repository settings. */
  settings?: Partial<RepositorySettings>;
}

/**
 * Update repository options.
 * 
 * @interface UpdateRepositoryOptions
 * @description Options for updating a repository.
 */
export interface UpdateRepositoryOptions {
  /** Repository name. */
  name?: string;
  /** Repository description. */
  description?: string;
  /** Default branch name. */
  defaultBranch?: string;
  /** Repository settings. */
  settings?: Partial<RepositorySettings>;
}

/**
 * Repository sync status.
 * 
 * @interface RepositorySyncStatus
 * @description Current sync status of a repository.
 */
export interface RepositorySyncStatus {
  /** Repository ID. */
  repositoryId: string;
  /** Sync status. */
  status: "idle" | "syncing" | "error";
  /** Sync progress (0-100). */
  progress: number;
  /** Current operation. */
  currentOperation?: string;
  /** Error message if status is error. */
  error?: string;
  /** Last sync timestamp. */
  lastSyncAt?: string;
  /** Next scheduled sync timestamp. */
  nextSyncAt?: string;
}
```

#### 4.2.2. Repository API Methods

```typescript
/**
 * ApiClient Repository API methods.
 * 
 * @description Methods for repository management operations.
 */
export class ApiClient {
  // ... (previous methods)

  /**
   * Retrieves a repository by ID.
   * 
   * @public
   * @param {string} repositoryId - Repository ID to retrieve.
   * @returns {Promise<Repository>} Repository data.
   * @throws {ApiError} If repository not found or access denied.
   * 
   * @example
   * ```typescript
   * const repository = await apiClient.getRepository("repo-123");
   * console.log(repository.name, repository.type);
   * ```
   */
  public async getRepository(repositoryId: string): Promise<Repository> {
    this.validateRepositoryId(repositoryId);
    return this.request<Repository>("GET", `/api/repositories/${repositoryId}`);
  }

  /**
   * Lists repositories with optional filtering and pagination.
   * 
   * @public
   * @param {ListRepositoriesOptions} [options] - List options.
   * @returns {Promise<ListRepositoriesResponse>} List of repositories.
   * @throws {ApiError} If request fails.
   * 
   * @example
   * ```typescript
   * const result = await apiClient.listRepositories({
   *   type: "git",
   *   limit: 20
   * });
   * console.log(result.repositories);
   * ```
   */
  public async listRepositories(
    options: ListRepositoriesOptions = {}
  ): Promise<ListRepositoriesResponse> {
    return this.request<ListRepositoriesResponse>("GET", "/api/repositories", {
      params: options
    });
  }

  /**
   * Creates a new repository.
   * 
   * @public
   * @param {CreateRepositoryOptions} options - Repository creation options.
   * @returns {Promise<Repository>} Created repository data.
   * @throws {ApiError} If creation fails.
   * 
   * @example
   * ```typescript
   * const repository = await apiClient.createRepository({
   *   name: "My Repo",
   *   type: "git",
   *   url: "https://github.com/user/repo.git",
   *   localPath: "/path/to/repo"
   * });
   * ```
   */
  public async createRepository(
    options: CreateRepositoryOptions
  ): Promise<Repository> {
    this.validateCreateRepositoryOptions(options);
    return this.request<Repository>("POST", "/api/repositories", {
      body: options
    });
  }

  /**
   * Updates an existing repository.
   * 
   * @public
   * @param {string} repositoryId - Repository ID to update.
   * @param {UpdateRepositoryOptions} options - Update options.
   * @returns {Promise<Repository>} Updated repository data.
   * @throws {ApiError} If update fails.
   * 
   * @example
   * ```typescript
   * const repository = await apiClient.updateRepository("repo-123", {
   *   name: "Updated Name",
   *   description: "Updated description"
   * });
   * ```
   */
  public async updateRepository(
    repositoryId: string,
    options: UpdateRepositoryOptions
  ): Promise<Repository> {
    this.validateRepositoryId(repositoryId);
    return this.request<Repository>("PUT", `/api/repositories/${repositoryId}`, {
      body: options
    });
  }

  /**
   * Deletes a repository.
   * 
   * @public
   * @param {string} repositoryId - Repository ID to delete.
   * @returns {Promise<void>} Resolves when deleted.
   * @throws {ApiError} If deletion fails.
   * 
   * @example
   * ```typescript
   * await apiClient.deleteRepository("repo-123");
   * ```
   */
  public async deleteRepository(repositoryId: string): Promise<void> {
    this.validateRepositoryId(repositoryId);
    return this.request<void>("DELETE", `/api/repositories/${repositoryId}`);
  }

  /**
   * Triggers a repository sync.
   * 
   * @public
   * @param {string} repositoryId - Repository ID to sync.
   * @returns {Promise<RepositorySyncStatus>} Sync status.
   * @throws {ApiError} If sync fails.
   * 
   * @example
   * ```typescript
   * const status = await apiClient.syncRepository("repo-123");
   * console.log(status.status, status.progress);
   * ```
   */
  public async syncRepository(
    repositoryId: string
  ): Promise<RepositorySyncStatus> {
    this.validateRepositoryId(repositoryId);
    return this.request<RepositorySyncStatus>(
      "POST",
      `/api/repositories/${repositoryId}/sync`
    );
  }

  /**
   * Gets repository sync status.
   * 
   * @public
   * @param {string} repositoryId - Repository ID.
   * @returns {Promise<RepositorySyncStatus>} Sync status.
   * @throws {ApiError} If retrieval fails.
   * 
   * @example
   * ```typescript
   * const status = await apiClient.getRepositoryStatus("repo-123");
   * console.log(status.status, status.progress);
   * ```
   */
  public async getRepositoryStatus(
    repositoryId: string
  ): Promise<RepositorySyncStatus> {
    this.validateRepositoryId(repositoryId);
    return this.request<RepositorySyncStatus>(
      "GET",
      `/api/repositories/${repositoryId}/status`
    );
  }

  /**
   * Validates repository ID format.
   * 
   * @private
   * @param {string} repositoryId - Repository ID to validate.
   * @throws {ValidationError} If invalid.
   */
  private validateRepositoryId(repositoryId: string): void {
    if (!repositoryId || typeof repositoryId !== "string") {
      throw new ValidationError("Repository ID is required and must be a string");
    }
  }

  /**
   * Validates create repository options.
   * 
   * @private
   * @param {CreateRepositoryOptions} options - Options to validate.
   * @throws {ValidationError} If invalid.
   */
  private validateCreateRepositoryOptions(options: CreateRepositoryOptions): void {
    if (!options.name || typeof options.name !== "string") {
      throw new ValidationError("Name is required and must be a string");
    }
    if (options.name.length < 1 || options.name.length > 200) {
      throw new ValidationError("Name must be between 1 and 200 characters");
    }
    if (!options.type || typeof options.type !== "string") {
      throw new ValidationError("Type is required and must be a string");
    }
    if (!options.localPath || typeof options.localPath !== "string") {
      throw new ValidationError("Local path is required and must be a string");
    }
  }
}
```

### 4.3. Search API Methods

The Search API provides methods for searching documents and repositories in the Tachyon system.

#### 4.3.1. Search Type Definitions

```typescript
/**
 * Search result item.
 * 
 * @interface SearchResultItem
 * @description Represents a single search result.
 */
export interface SearchResultItem {
  /** Result type. */
  type: "document" | "repository";
  /** Result ID. */
  id: string;
  /** Result title. */
  title: string;
  /** Result excerpt/snippet. */
  excerpt: string;
  /** Relevance score (0-1). */
  score: number;
  /** Repository ID (for documents). */
  repositoryId?: string;
  /** Document path (for documents). */
  path?: string;
  /** Highlighted matches. */
  highlights: SearchHighlight[];
}

/**
 * Search highlight.
 * 
 * @interface SearchHighlight
 * @description Represents a highlighted match in search results.
 */
export interface SearchHighlight {
  /** Field name. */
  field: string;
  /** Highlighted text. */
  text: string;
  /** Match position. */
  position: number;
}

/**
 * Search results.
 * 
 * @interface SearchResults
 * @description Response from search endpoints.
 */
export interface SearchResults {
  /** Search results. */
  results: SearchResultItem[];
  /** Total count. */
  total: number;
  /** Search query. */
  query: string;
  /** Search execution time in milliseconds. */
  executionTime: number;
}

/**
 * Search documents options.
 * 
 * @interface SearchDocumentsOptions
 * @description Options for searching documents.
 */
export interface SearchDocumentsOptions {
  /** Search query. */
  query: string;
  /** Repository ID filter. */
  repositoryId?: string;
  /** Tag filter. */
  tag?: string;
  /** Language filter. */
  language?: string;
  /** Maximum number of results. */
  limit?: number;
  /** Offset for pagination. */
  offset?: number;
  /** Minimum score threshold. */
  minScore?: number;
  /** Fields to search. */
  fields?: SearchField[];
  /** Enable fuzzy search. */
  fuzzy?: boolean;
}

/**
 * Search field enumeration.
 * 
 * @type SearchField
 * @description Fields available for searching.
 */
export type SearchField =
  | "title"
  | "content"
  | "tags"
  | "metadata";

/**
 * Search repositories options.
 * 
 * @interface SearchRepositoriesOptions
 * @description Options for searching repositories.
 */
export interface SearchRepositoriesOptions {
  /** Search query. */
  query: string;
  /** Repository type filter. */
  type?: RepositoryType;
  /** Maximum number of results. */
  limit?: number;
  /** Offset for pagination. */
  offset?: number;
  /** Minimum score threshold. */
  minScore?: number;
  /** Fields to search. */
  fields?: ("name" | "description")[];
  /** Enable fuzzy search. */
  fuzzy?: boolean;
}

/**
 * Search suggestion.
 * 
 * @interface SearchSuggestion
 * @description Represents a search suggestion.
 */
export interface SearchSuggestion {
  /** Suggestion text. */
  text: string;
  /** Suggestion type. */
  type: "query" | "document" | "repository";
  /** Result ID (for document/repository suggestions). */
  id?: string;
  /** Frequency/popularity score. */
  score: number;
}
```

#### 4.3.2. Search API Methods

```typescript
/**
 * ApiClient Search API methods.
 * 
 * @description Methods for search operations.
 */
export class ApiClient {
  // ... (previous methods)

  /**
   * Searches documents.
   * 
   * @public
   * @param {SearchDocumentsOptions} options - Search options.
   * @returns {Promise<SearchResults>} Search results.
   * @throws {ApiError} If search fails.
   * 
   * @example
   * ```typescript
   * const results = await apiClient.searchDocuments({
   *   query: "tachyon documentation",
   *   limit: 20,
   *   fuzzy: true
   * });
   * console.log(results.results);
   * ```
   */
  public async searchDocuments(
    options: SearchDocumentsOptions
  ): Promise<SearchResults> {
    this.validateSearchOptions(options);
    return this.request<SearchResults>("POST", "/api/search/documents", {
      body: options
    });
  }

  /**
   * Searches repositories.
   * 
   * @public
   * @param {SearchRepositoriesOptions} options - Search options.
   * @returns {Promise<SearchResults>} Search results.
   * @throws {ApiError} If search fails.
   * 
   * @example
   * ```typescript
   * const results = await apiClient.searchRepositories({
   *   query: "documentation repo",
   *   limit: 10
   * });
   * console.log(results.results);
   * ```
   */
  public async searchRepositories(
    options: SearchRepositoriesOptions
  ): Promise<SearchResults> {
    this.validateSearchOptions(options);
    return this.request<SearchResults>("POST", "/api/search/repositories", {
      body: options
    });
  }

  /**
   * Gets search suggestions.
   * 
   * @public
   * @param {string} query - Partial query.
   * @param {number} [limit] - Maximum number of suggestions.
   * @returns {Promise<SearchSuggestion[]>} Search suggestions.
   * @throws {ApiError} If retrieval fails.
   * 
   * @example
   * ```typescript
   * const suggestions = await apiClient.getSearchSuggestions("tach", 10);
   * console.log(suggestions);
   * ```
   */
  public async getSearchSuggestions(
    query: string,
    limit: number = 10
  ): Promise<SearchSuggestion[]> {
    if (!query || typeof query !== "string") {
      throw new ValidationError("Query is required and must be a string");
    }
    if (limit < 1 || limit > 100) {
      throw new ValidationError("Limit must be between 1 and 100");
    }
    return this.request<SearchSuggestion[]>(
      "GET",
      "/api/search/suggestions",
      { params: { query, limit } }
    );
  }

  /**
   * Validates search options.
   * 
   * @private
   * @param {SearchDocumentsOptions | SearchRepositoriesOptions} options - Options to validate.
   * @throws {ValidationError} If invalid.
   */
  private validateSearchOptions(
    options: SearchDocumentsOptions | SearchRepositoriesOptions
  ): void {
    if (!options.query || typeof options.query !== "string") {
      throw new ValidationError("Query is required and must be a string");
    }
    if (options.query.length < 1 || options.query.length > 1000) {
      throw new ValidationError("Query must be between 1 and 1000 characters");
    }
    if (options.limit !== undefined) {
      if (options.limit < 1 || options.limit > 100) {
        throw new ValidationError("Limit must be between 1 and 100");
      }
    }
  }
}

---

## 5. WEBSOCKET CLIENT METHODS

### 5.1. Connection Management

The WebSocketClient provides methods for managing the WebSocket connection to the server.

#### 5.1.1. Connection Methods

```typescript
/**
 * WebSocketClient Connection methods.
 * 
 * @description Methods for WebSocket connection management.
 */
export class WebSocketClient {
  // ... (previous constructor and private methods)

  /**
   * Connects to the WebSocket server.
   * 
   * @public
   * @returns {Promise<void>} Resolves when connected.
   * @throws {Error} If connection fails or times out.
   * 
   * @example
   * ```typescript
   * await wsClient.connect();
   * console.log("Connected to WebSocket server");
   * ```
   */
  public async connect(): Promise<void> {
    if (this.connectionState === "connected" || this.connectionState === "connecting") {
      this.log("Already connected or connecting");
      return;
    }

    this.connectionState = "connecting";
    this.log("Connecting to:", this.config.url);

    return new Promise((resolve, reject) => {
      // Build WebSocket URL with auth token
      let wsUrl = this.config.url;
      if (this.config.authToken) {
        const url = new URL(wsUrl);
        url.searchParams.append("token", this.config.authToken);
        wsUrl = url.toString();
      }

      // Create WebSocket connection
      this.ws = new WebSocket(wsUrl);

      // Set connection timeout
      const timeoutId = setTimeout(() => {
        this.ws?.close();
        reject(new Error("Connection timeout"));
      }, this.config.connectionTimeout);

      // Handle connection open
      this.ws.onopen = () => {
        clearTimeout(timeoutId);
        this.connectionState = "connected";
        this.reconnectAttempts = 0;
        this.startHeartbeat();
        this.flushMessageQueue();
        
        this.log("Connected to WebSocket server");
        
        // Emit connect event
        this.emit("connect", {
          type: "connect",
          payload: { timestamp: Date.now() },
          timestamp: Date.now(),
          messageId: this.generateMessageId()
        });

        resolve();
      };

      // Handle connection error
      this.ws.onerror = (event) => {
        clearTimeout(timeoutId);
        this.log("WebSocket error:", event);
        this.connectionState = "disconnected";
        reject(new Error("WebSocket connection error"));
      };

      // Handle connection close
      this.ws.onclose = (event) => {
        clearTimeout(timeoutId);
        this.stopHeartbeat();
        
        this.log("WebSocket closed:", event.code, event.reason);
        
        // Emit disconnect event
        this.emit("disconnect", {
          type: "disconnect",
          payload: { code: event.code, reason: event.reason },
          timestamp: Date.now(),
          messageId: this.generateMessageId()
        });

        // Attempt reconnection
        if (this.connectionState !== "disconnecting") {
          this.connectionState = "reconnecting";
          this.reconnect();
        } else {
          this.connectionState = "disconnected";
        }
      };

      // Handle incoming messages
      this.ws.onmessage = (event) => {
        try {
          const message: WebSocketMessage = JSON.parse(event.data);
          this.processMessage(message);
        } catch (error) {
          this.log("Failed to parse message:", error);
        }
      };
    });
  }

  /**
   * Disconnects from the WebSocket server.
   * 
   * @public
   * @param {number} [code=1000] - Close code.
   * @param {string} [reason] - Close reason.
   * @returns {Promise<void>} Resolves when disconnected.
   * 
   * @example
   * ```typescript
   * await wsClient.disconnect(1000, "User logged out");
   * ```
   */
  public async disconnect(code: number = 1000, reason?: string): Promise<void> {
    if (this.connectionState === "disconnected") {
      this.log("Already disconnected");
      return;
    }

    this.connectionState = "disconnecting";
    this.log("Disconnecting...");

    // Clear reconnection timer
    if (this.reconnectTimer !== null) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    // Close WebSocket connection
    if (this.ws) {
      this.ws.close(code, reason || "");
      this.ws = null;
    }

    // Clear all subscriptions
    this.subscriptions.clear();
    this.pendingSubscriptions.clear();

    this.connectionState = "disconnected";
    this.log("Disconnected");
  }

  /**
   * Sends a message to the server.
   * 
   * @public
   * @param {string} type - Message type.
   * @param {unknown} payload - Message payload.
   * @returns {void}
   * @throws {Error} If WebSocket is not connected.
   * 
   * @example
   * ```typescript
   * wsClient.send("document_update", { documentId: "doc-123", content: "Updated" });
   * ```
   */
  public send(type: string, payload: unknown): void {
    const message: WebSocketMessage = {
      type,
      payload,
      timestamp: Date.now(),
      messageId: this.generateMessageId()
    };

    if (this.connectionState === "connected" && this.ws?.readyState === WebSocket.OPEN) {
      this.sendRaw(message);
    } else if (this.config.enableMessageQueue) {
      // Queue message for later
      if (this.messageQueue.length < this.config.maxQueueSize) {
        this.messageQueue.push({ message, timestamp: Date.now() });
        this.log("Message queued:", message);
      } else {
        this.log("Message queue full, dropping message");
        throw new Error("Message queue full");
      }
    } else {
      throw new Error("WebSocket is not connected");
    }
  }

  /**
   * Registers an event handler for a specific event type.
   * 
   * @public
   * @param {EventType} eventType - Event type to listen for.
   * @param {EventHandler} handler - Event handler function.
   * @returns {() => void} Unsubscribe function.
   * 
   * @example
   * ```typescript
   * const unsubscribe = wsClient.on("document_update", (event) => {
   *   console.log("Document updated:", event.payload);
   * });
   * // Later...
   * unsubscribe();
   * ```
   */
  public on(eventType: EventType, handler: EventHandler): () => void {
    if (!this.eventHandlers.has(eventType)) {
      this.eventHandlers.set(eventType, []);
    }
    this.eventHandlers.get(eventType)!.push(handler);

    this.log("Registered handler for event:", eventType);

    // Return unsubscribe function
    return () => this.off(eventType, handler);
  }

  /**
   * Removes an event handler.
   * 
   * @public
   * @param {EventType} eventType - Event type.
   * @param {EventHandler} handler - Event handler to remove.
   * @returns {void}
   * 
   * @example
   * ```typescript
   * wsClient.off("document_update", myHandler);
   * ```
   */
  public off(eventType: EventType, handler: EventHandler): void {
    const handlers = this.eventHandlers.get(eventType);
    if (handlers) {
      const index = handlers.indexOf(handler);
      if (index > -1) {
        handlers.splice(index, 1);
        this.log("Removed handler for event:", eventType);
      }
    }
  }

  /**
   * Removes all event handlers for an event type.
   * 
   * @public
   * @param {EventType} eventType - Event type.
   * @returns {void}
   * 
   * @example
   * ```typescript
   * wsClient.removeAllListeners("document_update");
   * ```
   */
  public removeAllListeners(eventType: EventType): void {
    this.eventHandlers.delete(eventType);
    this.log("Removed all listeners for event:", eventType);
  }
}
```

---

## 6. WEBSOCKET SUBSCRIPTION METHODS

### 6.1. Subscription Management

The WebSocketClient provides methods for managing subscriptions to real-time events.

#### 6.1.1. Subscription Type Definitions

```typescript
/**
 * Subscription options.
 * 
 * @interface SubscribeOptions
 * @description Options for creating a subscription.
 */
export interface SubscribeOptions {
  /** Subscription channel. */
  channel: string;
  /** Filter criteria. */
  filter?: Record<string, unknown>;
  /** Subscription options. */
  options?: SubscriptionOptionsConfig;
}

/**
 * Subscription options configuration.
 * 
 * @interface SubscriptionOptionsConfig
 * @description Additional subscription configuration.
 */
export interface SubscriptionOptionsConfig {
  /** Include full document content in updates. */
  includeContent?: boolean;
  /** Include diff in updates. */
  includeDiff?: boolean;
  /** Maximum update frequency (ms). */
  throttleMs?: number;
  /** Batch updates. */
  batchUpdates?: boolean;
  /** Batch window (ms). */
  batchWindowMs?: number;
}

/**
 * Unsubscribe options.
 * 
 * @interface UnsubscribeOptions
 * @description Options for unsubscribing.
 */
export interface UnsubscribeOptions {
  /** Subscription ID. */
  subscriptionId?: string;
  /** Subscription channel. */
  channel?: string;
}
```

#### 6.1.2. Subscription Methods

```typescript
/**
 * WebSocketClient Subscription methods.
 * 
 * @description Methods for managing WebSocket subscriptions.
 */
export class WebSocketClient {
  // ... (previous methods)

  /**
   * Subscribes to a channel for real-time updates.
   * 
   * @public
   * @param {SubscribeOptions} options - Subscription options.
   * @returns {Promise<string>} Subscription ID.
   * @throws {Error} If subscription fails.
   * 
   * @example
   * ```typescript
   * const subscriptionId = await wsClient.subscribe({
   *   channel: "document:doc-123",
   *   options: { includeContent: true }
   * });
   * console.log("Subscribed:", subscriptionId);
   * ```
   */
  public async subscribe(options: SubscribeOptions): Promise<string> {
    if (this.connectionState !== "connected") {
      throw new Error("WebSocket is not connected");
    }

    this.validateSubscribeOptions(options);

    const subscriptionId = `sub_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    return new Promise((resolve, reject) => {
      // Store pending subscription
      this.pendingSubscriptions.set(subscriptionId, {
        id: subscriptionId,
        channel: options.channel,
        callback: (error) => {
          if (error) {
            reject(error);
          } else {
            resolve(subscriptionId);
          }
        }
      });

      // Send subscription request
      const message: WebSocketMessage = {
        type: "subscribe",
        payload: {
          subscriptionId,
          channel: options.channel,
          filter: options.filter,
          options: options.options
        },
        timestamp: Date.now(),
        messageId: this.generateMessageId()
      };

      this.sendRaw(message);

      // Set timeout for subscription acknowledgment
      setTimeout(() => {
        if (this.pendingSubscriptions.has(subscriptionId)) {
          this.pendingSubscriptions.delete(subscriptionId);
          reject(new Error("Subscription timeout"));
        }
      }, 10000);
    });
  }

  /**
   * Unsubscribes from a channel.
   * 
   * @public
   * @param {UnsubscribeOptions} options - Unsubscribe options.
   * @returns {Promise<void>} Resolves when unsubscribed.
   * @throws {Error} If unsubscription fails.
   * 
   * @example
   * ```typescript
   * await wsClient.unsubscribe({ subscriptionId: "sub-123" });
   * ```
   */
  public async unsubscribe(options: UnsubscribeOptions): Promise<void> {
    if (this.connectionState !== "connected") {
      throw new Error("WebSocket is not connected");
    }

    let subscriptionId: string | undefined;

    if (options.subscriptionId) {
      subscriptionId = options.subscriptionId;
    } else if (options.channel) {
      // Find subscription by channel
      for (const [id, sub] of this.subscriptions.entries()) {
        if (sub.channel === options.channel) {
          subscriptionId = id;
          break;
        }
      }
    }

    if (!subscriptionId) {
      throw new Error("Subscription not found");
    }

    // Send unsubscribe request
    const message: WebSocketMessage = {
      type: "unsubscribe",
      payload: { subscriptionId },
      timestamp: Date.now(),
      messageId: this.generateMessageId()
    };

    this.sendRaw(message);

    // Remove from active subscriptions
    this.subscriptions.delete(subscriptionId);

    this.log("Unsubscribed:", subscriptionId);
  }

  /**
   * Gets all active subscriptions.
   * 
   * @public
   * @returns {Subscription[]} Array of active subscriptions.
   * 
   * @example
   * ```typescript
   * const subscriptions = wsClient.getSubscriptions();
   * console.log("Active subscriptions:", subscriptions);
   * ```
   */
  public getSubscriptions(): Subscription[] {
    return Array.from(this.subscriptions.values());
  }

  /**
   * Gets a specific subscription by ID.
   * 
   * @public
   * @param {string} subscriptionId - Subscription ID.
   * @returns {Subscription | undefined} Subscription or undefined.
   * 
   * @example
   * ```typescript
   * const subscription = wsClient.getSubscription("sub-123");
   * if (subscription) {
   *   console.log("Subscription:", subscription);
   * }
   * ```
   */
  public getSubscription(subscriptionId: string): Subscription | undefined {
    return this.subscriptions.get(subscriptionId);
  }

  /**
   * Unsubscribes from all active subscriptions.
   * 
   * @public
   * @returns {Promise<void>} Resolves when all unsubscribed.
   * 
   * @example
   * ```typescript
   * await wsClient.unsubscribeAll();
   * ```
   */
  public async unsubscribeAll(): Promise<void> {
    const subscriptionIds = Array.from(this.subscriptions.keys());

    for (const subscriptionId of subscriptionIds) {
      try {
        await this.unsubscribe({ subscriptionId });
      } catch (error) {
        this.log("Failed to unsubscribe:", subscriptionId, error);
      }
    }
  }

  /**
   * Validates subscribe options.
   * 
   * @private
   * @param {SubscribeOptions} options - Options to validate.
   * @throws {ValidationError} If invalid.
   */
  private validateSubscribeOptions(options: SubscribeOptions): void {
    if (!options.channel || typeof options.channel !== "string") {
      throw new ValidationError("Channel is required and must be a string");
    }
    if (options.channel.length < 1 || options.channel.length > 500) {
      throw new ValidationError("Channel must be between 1 and 500 characters");
    }
  }
}
```

---

## 7. ERROR HANDLING

### 7.1. Error Types

The client API defines comprehensive error types for handling various failure scenarios.

#### 7.1.1. Error Type Definitions

```typescript
/**
 * API error class.
 * 
 * @class ApiError
 * @description Represents an API error with structured information.
 */
export class ApiError extends Error {
  /** Error code. */
  public readonly code: ErrorCode;
  /** Error message. */
  public readonly message: string;
  /** Additional error details. */
  public readonly details?: Record<string, unknown>;
  /** Error timestamp. */
  public readonly timestamp: number;
  /** Request ID. */
  public readonly requestId: string;
  /** HTTP status code. */
  public readonly statusCode?: number;

  /**
   * Creates a new ApiError.
   * 
   * @constructor
   * @param {ApiErrorOptions} options - Error options.
   */
  constructor(options: ApiErrorOptions) {
    super(options.message);
    this.name = "ApiError";
    this.code = options.code;
    this.message = options.message;
    this.details = options.details;
    this.timestamp = options.timestamp;
    this.requestId = options.requestId;
    this.statusCode = options.statusCode;
  }

  /**
   * Converts error to JSON.
   * 
   * @public
   * @returns {Record<string, unknown>} Error as JSON.
   */
  public toJSON(): Record<string, unknown> {
    return {
      name: this.name,
      code: this.code,
      message: this.message,
      details: this.details,
      timestamp: this.timestamp,
      requestId: this.requestId,
      statusCode: this.statusCode
    };
  }
}

/**
 * API error options.
 * 
 * @interface ApiErrorOptions
 * @description Options for creating an ApiError.
 */
export interface ApiErrorOptions {
  /** Error code. */
  code: ErrorCode;
  /** Error message. */
  message: string;
  /** Additional error details. */
  details?: Record<string, unknown>;
  /** Error timestamp. */
  timestamp: number;
  /** Request ID. */
  requestId: string;
  /** HTTP status code. */
  statusCode?: number;
}

/**
 * Error code enumeration.
 * 
 * @type ErrorCode
 * @description Possible error codes.
 */
export type ErrorCode =
  // Network errors
  | "NETWORK_ERROR"
  | "TIMEOUT"
  | "CONNECTION_ERROR"
  
  // HTTP errors
  | "HTTP_ERROR"
  | "BAD_REQUEST"
  | "UNAUTHORIZED"
  | "FORBIDDEN"
  | "NOT_FOUND"
  | "CONFLICT"
  | "UNPROCESSABLE_ENTITY"
  | "TOO_MANY_REQUESTS"
  | "INTERNAL_SERVER_ERROR"
  | "SERVICE_UNAVAILABLE"
  
  // Client errors
  | "VALIDATION_ERROR"
  | "INVALID_REQUEST"
  | "MISSING_PARAMETER"
  | "INVALID_PARAMETER"
  
  // WebSocket errors
  | "WEBSOCKET_ERROR"
  | "SUBSCRIPTION_ERROR"
  | "CONNECTION_LOST"
  
  // Unknown errors
  | "UNKNOWN_ERROR";

/**
 * Network error class.
 * 
 * @class NetworkError
 * @description Represents a network-related error.
 */
export class NetworkError extends ApiError {
  constructor(message: string, details?: Record<string, unknown>) {
    super({
      code: "NETWORK_ERROR",
      message,
      details,
      timestamp: Date.now(),
      requestId: `net_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
    });
    this.name = "NetworkError";
  }
}

/**
 * Timeout error class.
 * 
 * @class TimeoutError
 * @description Represents a request timeout error.
 */
export class TimeoutError extends ApiError {
  constructor(message: string, details?: Record<string, unknown>) {
    super({
      code: "TIMEOUT",
      message,
      details,
      timestamp: Date.now(),
      requestId: `timeout_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
    });
    this.name = "TimeoutError";
  }
}

/**
 * Authentication error class.
 * 
 * @class AuthenticationError
 * @description Represents an authentication failure.
 */
export class AuthenticationError extends ApiError {
  constructor(message: string, details?: Record<string, unknown>) {
    super({
      code: "UNAUTHORIZED",
      message,
      details,
      timestamp: Date.now(),
      requestId: `auth_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      statusCode: 401
    });
    this.name = "AuthenticationError";
  }
}

/**
 * Authorization error class.
 * 
 * @class AuthorizationError
 * @description Represents an authorization failure.
 */
export class AuthorizationError extends ApiError {
  constructor(message: string, details?: Record<string, unknown>) {
    super({
      code: "FORBIDDEN",
      message,
      details,
      timestamp: Date.now(),
      requestId: `authz_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      statusCode: 403
    });
    this.name = "AuthorizationError";
  }
}

/**
 * Not found error class.
 * 
 * @class NotFoundError
 * @description Represents a resource not found error.
 */
export class NotFoundError extends ApiError {
  constructor(message: string, details?: Record<string, unknown>) {
    super({
      code: "NOT_FOUND",
      message,
      details,
      timestamp: Date.now(),
      requestId: `notfound_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      statusCode: 404
    });
    this.name = "NotFoundError";
  }
}

/**
 * Conflict error class.
 * 
 * @class ConflictError
 * @description Represents a conflict error.
 */
export class ConflictError extends ApiError {
  constructor(message: string, details?: Record<string, unknown>) {
    super({
      code: "CONFLICT",
      message,
      details,
      timestamp: Date.now(),
      requestId: `conflict_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      statusCode: 409
    });
    this.name = "ConflictError";
  }
}

/**
 * Rate limit error class.
 * 
 * @class RateLimitError
 * @description Represents a rate limit error.
 */
export class RateLimitError extends ApiError {
  /** Retry after timestamp. */
  public readonly retryAfter?: number;

  constructor(message: string, details?: Record<string, unknown>) {
    super({
      code: "TOO_MANY_REQUESTS",
      message,
      details,
      timestamp: Date.now(),
      requestId: `ratelimit_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      statusCode: 429
    });
    this.name = "RateLimitError";
    this.retryAfter = (details?.retryAfter as number) ?? undefined;
  }
}

/**
 * WebSocket error class.
 * 
 * @class WebSocketError
 * @description Represents a WebSocket error.
 */
export class WebSocketError extends ApiError {
  constructor(message: string, details?: Record<string, unknown>) {
    super({
      code: "WEBSOCKET_ERROR",
      message,
      details,
      timestamp: Date.now(),
      requestId: `ws_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
    });
    this.name = "WebSocketError";
  }
}
```

#### 7.1.2. Error Handling Utilities

```typescript
/**
 * Error handling utilities.
 * 
 * @description Utility functions for error handling.
 */
export class ErrorHandler {
  /**
   * Determines if an error is retryable.
   * 
   * @public
   * @static
   * @param {Error} error - Error to check.
   * @returns {boolean} True if retryable.
   */
  public static isRetryable(error: Error): boolean {
    if (error instanceof NetworkError) {
      return true;
    }
    if (error instanceof TimeoutError) {
      return true;
    }
    if (error instanceof ApiError) {
      return [502, 503, 504].includes(error.statusCode ?? 0);
    }
    return false;
  }

  /**
   * Gets a user-friendly error message.
   * 
   * @public
   * @static
   * @param {Error} error - Error to format.
   * @returns {string} User-friendly message.
   */
  public static getUserMessage(error: Error): string {
    if (error instanceof AuthenticationError) {
      return "Please log in to continue";
    }
    if (error instanceof AuthorizationError) {
      return "You don't have permission to perform this action";
    }
    if (error instanceof NotFoundError) {
      return "The requested resource was not found";
    }
    if (error instanceof RateLimitError) {
      return "Too many requests. Please try again later";
    }
    if (error instanceof NetworkError) {
      return "Network error. Please check your connection";
    }
    if (error instanceof TimeoutError) {
      return "Request timed out. Please try again";
    }
    if (error instanceof ApiError) {
      return error.message;
    }
    return "An unexpected error occurred";
  }

  /**
   * Logs an error with context.
   * 
   * @public
   * @static
   * @param {Error} error - Error to log.
   * @param {Record<string, unknown>} [context] - Additional context.
   */
  public static log(error: Error, context?: Record<string, unknown>): void {
    const logData: Record<string, unknown> = {
      error: error.message,
      name: error.name,
      stack: error.stack,
      ...context
    };

    if (error instanceof ApiError) {
      logData.code = error.code;
      logData.requestId = error.requestId;
      logData.statusCode = error.statusCode;
    }

    console.error("[ErrorHandler]", logData);
  }
}

---

## 8. CLIENT SECURITY

### 8.1. Authentication

The client API provides secure authentication mechanisms for protecting sensitive operations.

#### 8.1.1. Authentication Methods

```typescript
/**
 * Authentication manager.
 * 
 * @class AuthenticationManager
 * @description Manages authentication tokens and credentials.
 */
export class AuthenticationManager {
  private token: string | null = null;
  private refreshToken: string | null = null;
  private tokenExpiry: number | null = null;
  private storageKey = "tachyon_auth";

  /**
   * Creates a new AuthenticationManager.
   * 
   * @constructor
   */
  constructor() {
    this.loadFromStorage();
  }

  /**
   * Sets the authentication token.
   * 
   * @public
   * @param {string} token - JWT token.
   * @param {number} expiresIn - Token expiry time in seconds.
   * @param {string} [refreshToken] - Refresh token.
   * @returns {void}
   * 
   * @example
   * ```typescript
   * authManager.setToken("jwt-token", 3600, "refresh-token");
   * ```
   */
  public setToken(token: string, expiresIn: number, refreshToken?: string): void {
    this.token = token;
    this.refreshToken = refreshToken ?? null;
    this.tokenExpiry = Date.now() + (expiresIn * 1000);
    this.saveToStorage();
  }

  /**
   * Gets the current authentication token.
   * 
   * @public
   * @returns {string | null} Current token or null if not authenticated.
   */
  public getToken(): string | null {
    // Check if token is expired
    if (this.tokenExpiry && Date.now() >= this.tokenExpiry) {
      this.clearToken();
      return null;
    }
    return this.token;
  }

  /**
   * Gets the refresh token.
   * 
   * @public
   * @returns {string | null} Refresh token or null.
   */
  public getRefreshToken(): string | null {
    return this.refreshToken;
  }

  /**
   * Checks if the user is authenticated.
   * 
   * @public
   * @returns {boolean} True if authenticated.
   */
  public isAuthenticated(): boolean {
    return this.getToken() !== null;
  }

  /**
   * Checks if the token is expired or will expire soon.
   * 
   * @public
   * @param {number} [threshold=300] - Threshold in seconds.
   * @returns {boolean} True if token is expired or will expire soon.
   */
  public isTokenExpired(threshold: number = 300): boolean {
    if (!this.tokenExpiry) {
      return true;
    }
    return Date.now() >= (this.tokenExpiry - (threshold * 1000));
  }

  /**
   * Clears the authentication token.
   * 
   * @public
   * @returns {void}
   */
  public clearToken(): void {
    this.token = null;
    this.refreshToken = null;
    this.tokenExpiry = null;
    this.saveToStorage();
  }

  /**
   * Saves authentication data to storage.
   * 
   * @private
   */
  private saveToStorage(): void {
    try {
      const data = {
        token: this.token,
        refreshToken: this.refreshToken,
        tokenExpiry: this.tokenExpiry
      };
      localStorage.setItem(this.storageKey, JSON.stringify(data));
    } catch (error) {
      console.error("Failed to save auth data:", error);
    }
  }

  /**
   * Loads authentication data from storage.
   * 
   * @private
   */
  private loadFromStorage(): void {
    try {
      const data = localStorage.getItem(this.storageKey);
      if (data) {
        const parsed = JSON.parse(data);
        this.token = parsed.token;
        this.refreshToken = parsed.refreshToken;
        this.tokenExpiry = parsed.tokenExpiry;
      }
    } catch (error) {
      console.error("Failed to load auth data:", error);
    }
  }
}
```

### 8.2. Authorization

The client API includes authorization checks for protecting resources based on user permissions.

#### 8.2.1. Authorization Type Definitions

```typescript
/**
 * Permission enumeration.
 * 
 * @type Permission
 * @description Available permissions.
 */
export type Permission =
  | "documents:read"
  | "documents:write"
  | "documents:delete"
  | "repositories:read"
  | "repositories:write"
  | "repositories:delete"
  | "repositories:sync"
  | "search:read"
  | "admin:*";

/**
 * Role enumeration.
 * 
 * @type Role
 * @description Available user roles.
 */
export type Role =
  | "viewer"
  | "editor"
  | "admin"
  | "owner";

/**
 * User permissions.
 * 
 * @interface UserPermissions
 * @description User's permissions and roles.
 */
export interface UserPermissions {
  /** User ID. */
  userId: string;
  /** User roles. */
  roles: Role[];
  /** User permissions. */
  permissions: Permission[];
  /** Repository-specific permissions. */
  repositoryPermissions: Map<string, Permission[]>;
}
```

#### 8.2.2. Authorization Methods

```typescript
/**
 * Authorization manager.
 * 
 * @class AuthorizationManager
 * @description Manages authorization checks and permissions.
 */
export class AuthorizationManager {
  private permissions: UserPermissions | null = null;

  /**
   * Sets the user permissions.
   * 
   * @public
   * @param {UserPermissions} permissions - User permissions.
   * @returns {void}
   */
  public setPermissions(permissions: UserPermissions): void {
    this.permissions = permissions;
  }

  /**
   * Checks if the user has a specific permission.
   * 
   * @public
   * @param {Permission} permission - Permission to check.
   * @returns {boolean} True if user has permission.
   * 
   * @example
   * ```typescript
   * if (authzManager.hasPermission("documents:write")) {
   *   // User can write documents
   * }
   * ```
   */
  public hasPermission(permission: Permission): boolean {
    if (!this.permissions) {
      return false;
    }

    // Admin has all permissions
    if (this.permissions.roles.includes("admin")) {
      return true;
    }

    // Check direct permission
    if (this.permissions.permissions.includes(permission)) {
      return true;
    }

    return false;
  }

  /**
   * Checks if the user has a specific permission for a repository.
   * 
   * @public
   * @param {string} repositoryId - Repository ID.
   * @param {Permission} permission - Permission to check.
   * @returns {boolean} True if user has permission.
   * 
   * @example
   * ```typescript
   * if (authzManager.hasRepositoryPermission("repo-123", "repositories:write")) {
   *   // User can write to this repository
   * }
   * ```
   */
  public hasRepositoryPermission(
    repositoryId: string,
    permission: Permission
  ): boolean {
    if (!this.permissions) {
      return false;
    }

    // Admin has all permissions
    if (this.permissions.roles.includes("admin")) {
      return true;
    }

    // Check repository-specific permission
    const repoPerms = this.permissions.repositoryPermissions.get(repositoryId);
    if (repoPerms && repoPerms.includes(permission)) {
      return true;
    }

    // Check global permission
    if (this.permissions.permissions.includes(permission)) {
      return true;
    }

    return false;
  }

  /**
   * Checks if the user has a specific role.
   * 
   * @public
   * @param {Role} role - Role to check.
   * @returns {boolean} True if user has role.
   */
  public hasRole(role: Role): boolean {
    if (!this.permissions) {
      return false;
    }
    return this.permissions.roles.includes(role);
  }

  /**
   * Gets all user permissions.
   * 
   * @public
   * @returns {UserPermissions | null} User permissions.
   */
  public getPermissions(): UserPermissions | null {
    return this.permissions;
  }

  /**
   * Clears the user permissions.
   * 
   * @public
   * @returns {void}
   */
  public clearPermissions(): void {
    this.permissions = null;
  }
}
```

---

## 9. CLIENT PERFORMANCE

### 9.1. Latency Requirements

The client API is designed to meet strict latency requirements for optimal user experience.

#### 9.1.1. Latency Targets

| Operation | Target Latency | Maximum Latency | Measurement Method |
|------------|----------------|------------------|---------------------|
| Document retrieval | < 100ms | 500ms | Time-to-first-byte |
| Document listing | < 150ms | 1000ms | Full response time |
| Document creation | < 200ms | 2000ms | Full response time |
| Document update | < 200ms | 2000ms | Full response time |
| Search query | < 200ms | 1500ms | Full response time |
| WebSocket connection | < 500ms | 3000ms | Connection established |
| WebSocket message | < 50ms | 200ms | Message delivery |
| Subscription creation | < 100ms | 1000ms | Acknowledgment received |

**Rationale:** These latency targets ensure responsive user experience and meet performance requirements [REQ-WEB-066, REQ-WEB-067].

#### 9.1.2. Performance Monitoring

```typescript
/**
 * Performance monitor.
 * 
 * @class PerformanceMonitor
 * @description Monitors and tracks API performance metrics.
 */
export class PerformanceMonitor {
  private metrics: Map<string, PerformanceMetric[]> = new Map();
  private maxMetricsPerKey = 100;

  /**
   * Records a performance metric.
   * 
   * @public
   * @param {string} key - Metric key (e.g., "api.documents.get").
   * @param {number} duration - Duration in milliseconds.
   * @param {boolean} [success] - Whether the operation succeeded.
   * @returns {void}
   * 
   * @example
   * ```typescript
   * const start = performance.now();
   * await apiClient.getDocument("doc-123");
   * performanceMonitor.record("api.documents.get", performance.now() - start, true);
   * ```
   */
  public record(key: string, duration: number, success: boolean = true): void {
    if (!this.metrics.has(key)) {
      this.metrics.set(key, []);
    }

    const metrics = this.metrics.get(key)!;
    metrics.push({
      duration,
      success,
      timestamp: Date.now()
    });

    // Keep only recent metrics
    if (metrics.length > this.maxMetricsPerKey) {
      metrics.shift();
    }
  }

  /**
   * Gets statistics for a metric.
   * 
   * @public
   * @param {string} key - Metric key.
   * @returns {PerformanceStats | null} Statistics or null if no metrics.
   */
  public getStats(key: string): PerformanceStats | null {
    const metrics = this.metrics.get(key);
    if (!metrics || metrics.length === 0) {
      return null;
    }

    const durations = metrics.map(m => m.duration).sort((a, b) => a - b);
    const sum = durations.reduce((a, b) => a + b, 0);
    const successCount = metrics.filter(m => m.success).length;

    return {
      count: metrics.length,
      successRate: successCount / metrics.length,
      avg: sum / durations.length,
      min: durations[0],
      max: durations[durations.length - 1],
      p50: durations[Math.floor(durations.length * 0.5)],
      p95: durations[Math.floor(durations.length * 0.95)],
      p99: durations[Math.floor(durations.length * 0.99)]
    };
  }

  /**
   * Gets all metric keys.
   * 
   * @public
   * @returns {string[]} Array of metric keys.
   */
  public getKeys(): string[] {
    return Array.from(this.metrics.keys());
  }

  /**
   * Clears all metrics.
   * 
   * @public
   * @returns {void}
   */
  public clear(): void {
    this.metrics.clear();
  }
}

/**
 * Performance metric.
 * 
 * @interface PerformanceMetric
 * @description Single performance measurement.
 */
export interface PerformanceMetric {
  /** Duration in milliseconds. */
  duration: number;
  /** Whether the operation succeeded. */
  success: boolean;
  /** Timestamp. */
  timestamp: number;
}

/**
 * Performance statistics.
 * 
 * @interface PerformanceStats
 * @description Aggregated performance statistics.
 */
export interface PerformanceStats {
  /** Number of measurements. */
  count: number;
  /** Success rate (0-1). */
  successRate: number;
  /** Average duration. */
  avg: number;
  /** Minimum duration. */
  min: number;
  /** Maximum duration. */
  max: number;
  /** 50th percentile. */
  p50: number;
  /** 95th percentile. */
  p95: number;
  /** 99th percentile. */
  p99: number;
}
```

### 9.2. Caching Strategies

The client API implements intelligent caching strategies to reduce server load and improve response times.

#### 9.2.1. Cache Configuration

```typescript
/**
 * Cache configuration.
 * 
 * @interface CacheConfig
 * @description Configuration for response caching.
 */
export interface CacheConfig {
  /** Enable caching. */
  enabled: boolean;
  /** Default TTL in milliseconds. */
  defaultTTL: number;
  /** Maximum cache size. */
  maxSize: number;
  /** Per-endpoint TTL overrides. */
  endpointTTLs: Map<string, number>;
}

/**
 * Cache entry.
 * 
 * @interface CacheEntry
 * @description Cached response entry.
 */
export interface CacheEntry<T> {
  /** Cached data. */
  data: T;
  /** Cache timestamp. */
  timestamp: number;
  /** TTL in milliseconds. */
  ttl: number;
  /** Access count. */
  accessCount: number;
  /** Last access timestamp. */
  lastAccess: number;
}
```

#### 9.2.2. Cache Implementation

```typescript
/**
 * Response cache.
 * 
 * @class ResponseCache
 * @description In-memory response cache with LRU eviction.
 */
export class ResponseCache {
  private cache: Map<string, CacheEntry<unknown>> = new Map();
  private config: CacheConfig;

  /**
   * Creates a new ResponseCache.
   * 
   * @constructor
   * @param {Partial<CacheConfig>} [config] - Cache configuration.
   */
  constructor(config: Partial<CacheConfig> = {}) {
    this.config = {
      enabled: config.enabled ?? true,
      defaultTTL: config.defaultTTL ?? 60000,
      maxSize: config.maxSize ?? 1000,
      endpointTTLs: config.endpointTTLs ?? new Map()
    };
  }

  /**
   * Gets a cached response.
   * 
   * @public
   * @template T - Response type.
   * @param {string} key - Cache key.
   * @returns {T | null} Cached data or null if not found/expired.
   * 
   * @example
   * ```typescript
   * const cached = cache.get<Document>("api.documents.doc-123");
   * if (cached) {
   *   console.log("Cache hit:", cached);
   * }
   * ```
   */
  public get<T>(key: string): T | null {
    if (!this.config.enabled) {
      return null;
    }

    const entry = this.cache.get(key);
    if (!entry) {
      return null;
    }

    // Check if expired
    if (Date.now() - entry.timestamp > entry.ttl) {
      this.cache.delete(key);
      return null;
    }

    // Update access stats
    entry.accessCount++;
    entry.lastAccess = Date.now();

    return entry.data as T;
  }

  /**
   * Sets a cached response.
   * 
   * @public
   * @template T - Response type.
   * @param {string} key - Cache key.
   * @param {T} data - Data to cache.
   * @param {number} [ttl] - Custom TTL.
   * @returns {void}
   * 
   * @example
   * ```typescript
   * cache.set("api.documents.doc-123", document, 120000);
   * ```
   */
  public set<T>(key: string, data: T, ttl?: number): void {
    if (!this.config.enabled) {
      return;
    }

    // Evict if at capacity
    if (this.cache.size >= this.config.maxSize) {
      this.evictLRU();
    }

    // Determine TTL
    const entryTTL = ttl ?? this.config.defaultTTL;

    this.cache.set(key, {
      data,
      timestamp: Date.now(),
      ttl: entryTTL,
      accessCount: 0,
      lastAccess: Date.now()
    });
  }

  /**
   * Invalidates a cache entry.
   * 
   * @public
   * @param {string} key - Cache key to invalidate.
   * @returns {boolean} True if entry was found and removed.
   */
  public invalidate(key: string): boolean {
    return this.cache.delete(key);
  }

  /**
   * Invalidates all cache entries matching a pattern.
   * 
   * @public
   * @param {RegExp} pattern - Pattern to match.
   * @returns {number} Number of entries invalidated.
   */
  public invalidatePattern(pattern: RegExp): number {
    let count = 0;
    for (const key of this.cache.keys()) {
      if (pattern.test(key)) {
        this.cache.delete(key);
        count++;
      }
    }
    return count;
  }

  /**
   * Clears all cache entries.
   * 
   * @public
   * @returns {void}
   */
  public clear(): void {
    this.cache.clear();
  }

  /**
   * Gets cache statistics.
   * 
   * @public
   * @returns {CacheStats} Cache statistics.
   */
  public getStats(): CacheStats {
    let totalAccessCount = 0;
    let hitCount = 0;

    for (const entry of this.cache.values()) {
      totalAccessCount += entry.accessCount;
      if (entry.accessCount > 0) {
        hitCount++;
      }
    }

    return {
      size: this.cache.size,
      maxSize: this.config.maxSize,
      hitRate: hitCount / this.cache.size,
      totalAccessCount
    };
  }

  /**
   * Evicts the least recently used entry.
   * 
   * @private
   */
  private evictLRU(): void {
    let lruKey: string | null = null;
    let lruTime = Infinity;

    for (const [key, entry] of this.cache.entries()) {
      if (entry.lastAccess < lruTime) {
        lruTime = entry.lastAccess;
        lruKey = key;
      }
    }

    if (lruKey) {
      this.cache.delete(lruKey);
    }
  }
}

/**
 * Cache statistics.
 * 
 * @interface CacheStats
 * @description Cache performance statistics.
 */
export interface CacheStats {
  /** Current cache size. */
  size: number;
  /** Maximum cache size. */
  maxSize: number;
  /** Cache hit rate (0-1). */
  hitRate: number;
  /** Total access count. */
  totalAccessCount: number;
}
```

---

## 10. REFERENCES

### 10.1. Normative References

1. **[TACHYON-STD-V1.0](../../.adrs/ - Coding and Documentation Standards
   - Defines coding standards and documentation requirements for Tachyon project
   - Compliance: ISO/IEC 26514:2021, IEEE 1063-2001

2. **[TACHYON-REQ-WEB-V1.0](../../.adrs/ - Web Frontend Requirements
   - Defines functional and non-functional requirements for the web frontend
   - Covers: REQ-WEB-001 through REQ-WEB-090

3. **[TACHYON-DES-WD-V1.0](../../.adrs/ - Web Frontend Design
   - Defines design elements and component architecture for the web frontend
   - Covers: DES-WD-001 through DES-WD-007

4. **[TACHYON-ADR-004-V1.0](../../.adrs/adr-004-debounce-window.md)** - ADR-004: Leptos for Web Frontend
   - Architectural Decision Record for selecting Leptos as the web framework
   - Rationale: Fine-grained reactivity, SSR support, Rust ecosystem

5. **[TACHYON-ADR-005-V1.0](../../.adrs/adr-005-last-write-wins-conflict-resolution.md)** - ADR-005: Bun for JavaScript Runtime
   - Architectural Decision Record for selecting Bun as the JavaScript runtime
   - Rationale: Performance, TypeScript support, bundling capabilities

6. **[TACHYON-TMA-V1.0](../../.adrs/ - Threat Model Analysis
   - Defines security threats and mitigation strategies
   - Covers: Client-side security, authentication, authorization

### 10.2. Informative References

1. **ISO/IEC 26514:2021** - Systems and software engineering — Requirements for designers and developers of user documentation
   - Provides requirements for the design and development of user documentation
   - Used as basis for documentation standards

2. **IEEE 1063-2001** - IEEE Standard for Software User Documentation
   - Defines requirements for software user documentation
   - Used as basis for documentation standards

3. **IEEE 829-2008** - IEEE Standard for Software and System Test Documentation
   - Defines requirements for test documentation
   - Referenced for testing documentation standards

4. **TypeScript 5.x** - TypeScript Language Specification
   - Defines TypeScript language syntax and semantics
   - Used for type definitions and interfaces

5. **Leptos 0.8.x** - Leptos Framework Documentation
   - Provides documentation for the Leptos reactive web framework
   - Referenced for component design and state management

6. **Bun 1.2.x** - Bun Runtime Documentation
   - Provides documentation for the Bun JavaScript runtime
   - Referenced for build tooling and runtime features

7. **RFC 6265** - HTTP State Management Mechanism (Cookies)
   - Defines HTTP cookie specification
   - Referenced for authentication and session management

8. **RFC 6455** - The WebSocket Protocol
   - Defines WebSocket protocol specification
   - Referenced for WebSocket client implementation

9. **RFC 7519** - JSON Web Token (JWT)
   - Defines JWT specification
   - Referenced for authentication token handling

10. **RFC 7540** - Hypertext Transfer Protocol Version 2 (HTTP/2)
    - Defines HTTP/2 protocol specification
    - Referenced for HTTP/2 client implementation

### 10.3. Related Documents

1. **[TACHYON-API-003-V1.0](web_api_specification.md)** - Web API Specification
   - Defines server-side API endpoints and protocols
   - Complementary to this client API specification

2. **[TACHYON-API-006-V1.0](rest_api_specification.md)** - REST API Specification
   - Defines RESTful API endpoints for server communication
   - Referenced for HTTP client method implementations

3. **[TACHYON-API-010-V1.0](websocket_api_specification.md)** - WebSocket API Specification
   - Defines WebSocket protocol and message formats
   - Referenced for WebSocket client implementation

4. **[TACHYON-API-008-V1.0](server_api_specification.md)** - Server API Specification
   - Defines server-side internal APIs
   - Referenced for understanding server capabilities

### 10.4. Glossary

| Term | Definition |
|-------|------------|
| **ApiClient** | Type-safe HTTP client abstraction for server communication |
| **WebSocketClient** | Real-time bidirectional communication client for live updates |
| **Signal** | Leptos reactive primitive for state management |
| **Subscription** | Active listener for WebSocket events |
| **Request Deduplication** | Preventing redundant concurrent requests |
| **Response Caching** | In-memory caching of GET responses with TTL |
| **TTL** | Time-to-live - duration for which cached data is valid |
| **LRU** | Least Recently Used - cache eviction strategy |
| **JWT** | JSON Web Token - authentication token format |
| **SSR** | Server-Side Rendering - initial HTML rendering on server |
| **Hydration** | Process of attaching event listeners to server-rendered HTML |
| **WASM** | WebAssembly - binary instruction format for web |

---

## DOCUMENT HISTORY

| Version | Date | Author | Description |
|---------|-------|---------|-------------|
| 1.0 | February 2026 | Technical Writer | Initial release of Web Client API Specification |

---

**END OF DOCUMENT**



