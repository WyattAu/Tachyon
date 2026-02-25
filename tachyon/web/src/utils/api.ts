/**
 * API Client for Tachyon backend communication
 * Uses neverthrow for functional error handling
 */

import { Result, ResultAsync, err, ok, errAsync, okAsync } from 'neverthrow';

interface RequestOptions {
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  body?: unknown;
  headers?: Record<string, string>;
}

interface ApiResponse<T = unknown> {
  data: T;
  status: number;
  headers: Headers;
}

interface AuthStatus {
  authenticated: boolean;
  user?: {
    id: string;
    username: string;
    email?: string;
  };
}

export interface ApiError {
  type: 'network' | 'auth' | 'validation' | 'not_found' | 'server' | 'parse' | 'unknown';
  status: number;
  message: string;
  details?: unknown;
  timestamp: string;
}

/**
 * Create a structured API error
 */
function createApiError(
  type: ApiError['type'],
  status: number,
  message: string,
  details?: unknown
): ApiError {
  const error: ApiError = {
    type,
    status,
    message,
    timestamp: new Date().toISOString(),
  };
  if (details !== undefined) {
    error.details = details;
  }
  
  // Log error to console for debugging
  console.error('[API Error]', {
    type,
    status,
    message,
    details,
    timestamp: error.timestamp,
  });
  
  return error;
}

/**
 * Map HTTP status codes to error types
 */
function statusToErrorType(status: number): ApiError['type'] {
  if (status === 401 || status === 403) return 'auth';
  if (status === 404) return 'not_found';
  if (status === 400 || status === 422) return 'validation';
  if (status >= 500) return 'server';
  return 'unknown';
}

export class ApiClient {
  private baseUrl: string;
  private token: string | null = null;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
    // Try to load token from localStorage
    this.token = localStorage.getItem('tachyon_token');
  }

  /**
   * Set authentication token
   */
  setToken(token: string | null): void {
    this.token = token;
    if (token) {
      localStorage.setItem('tachyon_token', token);
    } else {
      localStorage.removeItem('tachyon_token');
    }
  }

  /**
   * Get current token
   */
  getToken(): string | null {
    return this.token;
  }

  /**
   * Make an API request with neverthrow error handling
   */
  request<T = unknown>(endpoint: string, options: RequestOptions = {}): ResultAsync<ApiResponse<T>, ApiError> {
    const { method = 'GET', body, headers = {} } = options;

    const requestHeaders: Record<string, string> = {
      'Content-Type': 'application/json',
      ...headers,
    };

    if (this.token) {
      requestHeaders['Authorization'] = `Bearer ${this.token}`;
    }

    const url = `${this.baseUrl}${endpoint}`;
    
    console.log('[API Request]', { method, url, hasBody: !!body });

    return ResultAsync.fromPromise(
      fetch(url, {
        method,
        headers: requestHeaders,
        body: body ? JSON.stringify(body) : undefined,
      }),
      (error) => {
        console.error('[API Network Error]', error);
        return createApiError('network', 0, 'Network request failed', error);
      }
    ).andThen((response) => {
      if (!response.ok) {
        return ResultAsync.fromPromise(
          response.json().catch(() => ({ message: 'Unknown error' })),
          () => createApiError('parse', response.status, 'Failed to parse error response')
        ).andThen((errorBody) => 
          errAsync(createApiError(
            statusToErrorType(response.status),
            response.status,
            errorBody.message || 'Request failed',
            errorBody
          ))
        );
      }

      return ResultAsync.fromPromise(
        response.json(),
        (error) => {
          console.error('[API Parse Error]', error);
          return createApiError('parse', response.status, 'Failed to parse response JSON', error);
        }
      ).map((data) => ({
        data,
        status: response.status,
        headers: response.headers,
      }));
    });
  }

  /**
   * Check authentication status
   */
  checkAuth(): ResultAsync<AuthStatus, ApiError> {
    return this.request<AuthStatus>('/api/v1/auth/status')
      .map((response) => response.data)
      .orElse(() => okAsync({ authenticated: false }));
  }

  /**
   * Login with credentials
   */
  login(username: string, password: string): ResultAsync<{ success: boolean; token?: string; user?: unknown }, ApiError> {
    return this.request<{ token: string; user: unknown }>('/api/v1/auth/login', {
      method: 'POST',
      body: { username, password },
    }).map((response) => {
      if (response.data.token) {
        this.setToken(response.data.token);
        return { success: true, token: response.data.token, user: response.data.user };
      }
      return { success: false };
    });
  }

  /**
   * Logout
   */
  logout(): ResultAsync<void, ApiError> {
    return this.request('/api/v1/auth/logout', { method: 'POST' })
      .map(() => {
        this.setToken(null);
      })
      .orElse((error) => {
        // Always clear token on logout, even if API call fails
        this.setToken(null);
        console.warn('[API] Logout API call failed, but token cleared locally:', error.message);
        return okAsync(undefined);
      });
  }

  /**
   * Get documents
   */
  getDocuments(params?: { repositoryId?: string; page?: number; limit?: number }): ResultAsync<ApiResponse<{ documents: unknown[]; total: number }>, ApiError> {
    const query = new URLSearchParams();
    if (params?.repositoryId) query.set('repositoryId', params.repositoryId);
    if (params?.page) query.set('page', params.page.toString());
    if (params?.limit) query.set('limit', params.limit.toString());
    
    const queryString = query.toString();
    const endpoint = queryString ? `/api/v1/documents?${queryString}` : '/api/v1/documents';
    
    return this.request(endpoint);
  }

  /**
   * Get a single document
   */
  getDocument(id: string): ResultAsync<ApiResponse<{ id: string; title: string; content: string }>, ApiError> {
    return this.request(`/api/v1/documents/${id}`);
  }

  /**
   * Create a document
   */
  createDocument(data: { title: string; content: string; repositoryId?: string }): ResultAsync<ApiResponse<{ id: string; title: string }>, ApiError> {
    return this.request('/api/v1/documents', {
      method: 'POST',
      body: data,
    });
  }

  /**
   * Update a document
   */
  updateDocument(id: string, data: { title?: string; content?: string }): ResultAsync<ApiResponse<{ id: string; title: string }>, ApiError> {
    return this.request(`/api/v1/documents/${id}`, {
      method: 'PUT',
      body: data,
    });
  }

  /**
   * Delete a document
   */
  deleteDocument(id: string): ResultAsync<ApiResponse<void>, ApiError> {
    return this.request(`/api/v1/documents/${id}`, {
      method: 'DELETE',
    });
  }

  /**
   * Search documents
   */
  search(query: string, options?: { page?: number; limit?: number }): ResultAsync<ApiResponse<{ results: unknown[]; total: number }>, ApiError> {
    const params = new URLSearchParams();
    params.set('search', query);
    if (options?.page) params.set('page', options.page.toString());
    if (options?.limit) params.set('page_size', options.limit.toString());
    
    return this.request(`/api/v1/documents?${params.toString()}`);
  }

  /**
   * Get repositories
   */
  getRepositories(): ResultAsync<ApiResponse<{ repositories: unknown[] }>, ApiError> {
    return this.request('/api/v1/repositories');
  }

  /**
   * Get health status
   */
  getHealth(): ResultAsync<ApiResponse<{ status: string; version: string }>, ApiError> {
    return this.request('/api/v1/health');
  }
}

// Global API client instance
let apiClient: ApiClient | null = null;

/**
 * Get or create the global API client
 */
export function getApiClient(baseUrl?: string): ApiClient {
  if (!apiClient && baseUrl) {
    apiClient = new ApiClient(baseUrl);
  }
  if (!apiClient) {
    throw new Error('API client not initialized. Call getApiClient with a baseUrl first.');
  }
  return apiClient;
}

/**
 * Initialize the global API client
 */
export function initApiClient(baseUrl: string): ApiClient {
  apiClient = new ApiClient(baseUrl);
  console.log('[API] Client initialized with baseUrl:', baseUrl);
  return apiClient;
}
