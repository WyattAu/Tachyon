/**
 * API Client for Tachyon backend communication
 */

interface RequestOptions {
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  body?: any;
  headers?: Record<string, string>;
}

interface ApiResponse<T = any> {
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
   * Make an API request
   */
  async request<T = any>(endpoint: string, options: RequestOptions = {}): Promise<ApiResponse<T>> {
    const { method = 'GET', body, headers = {} } = options;

    const requestHeaders: Record<string, string> = {
      'Content-Type': 'application/json',
      ...headers,
    };

    if (this.token) {
      requestHeaders['Authorization'] = `Bearer ${this.token}`;
    }

    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      method,
      headers: requestHeaders,
      body: body ? JSON.stringify(body) : undefined,
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ message: 'Unknown error' }));
      throw new ApiError(response.status, error.message || 'Request failed', error);
    }

    const data = await response.json();
    return {
      data,
      status: response.status,
      headers: response.headers,
    };
  }

  /**
   * Check authentication status
   */
  async checkAuth(): Promise<AuthStatus> {
    try {
      const response = await this.request<AuthStatus>('/auth/status');
      return response.data;
    } catch {
      return { authenticated: false };
    }
  }

  /**
   * Login with credentials
   */
  async login(username: string, password: string): Promise<{ success: boolean; token?: string; user?: any }> {
    const response = await this.request<{ token: string; user: any }>('/auth/login', {
      method: 'POST',
      body: { username, password },
    });
    
    if (response.data.token) {
      this.setToken(response.data.token);
      return { success: true, token: response.data.token, user: response.data.user };
    }
    
    return { success: false };
  }

  /**
   * Logout
   */
  async logout(): Promise<void> {
    try {
      await this.request('/auth/logout', { method: 'POST' });
    } finally {
      this.setToken(null);
    }
  }

  /**
   * Get documents
   */
  async getDocuments(params?: { repositoryId?: string; page?: number; limit?: number }): Promise<any> {
    const query = new URLSearchParams();
    if (params?.repositoryId) query.set('repositoryId', params.repositoryId);
    if (params?.page) query.set('page', params.page.toString());
    if (params?.limit) query.set('limit', params.limit.toString());
    
    const queryString = query.toString();
    const endpoint = queryString ? `/documents?${queryString}` : '/documents';
    
    return this.request(endpoint);
  }

  /**
   * Get a single document
   */
  async getDocument(id: string): Promise<any> {
    return this.request(`/documents/${id}`);
  }

  /**
   * Create a document
   */
  async createDocument(data: { title: string; content: string; repositoryId?: string }): Promise<any> {
    return this.request('/documents', {
      method: 'POST',
      body: data,
    });
  }

  /**
   * Update a document
   */
  async updateDocument(id: string, data: { title?: string; content?: string }): Promise<any> {
    return this.request(`/documents/${id}`, {
      method: 'PUT',
      body: data,
    });
  }

  /**
   * Delete a document
   */
  async deleteDocument(id: string): Promise<any> {
    return this.request(`/documents/${id}`, {
      method: 'DELETE',
    });
  }

  /**
   * Search documents
   */
  async search(query: string, options?: { page?: number; limit?: number }): Promise<any> {
    const params = new URLSearchParams({ q: query });
    if (options?.page) params.set('page', options.page.toString());
    if (options?.limit) params.set('limit', options.limit.toString());
    
    return this.request(`/search?${params.toString()}`);
  }
}

/**
 * Custom API Error class
 */
export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
    public data?: any
  ) {
    super(message);
    this.name = 'ApiError';
  }
}
