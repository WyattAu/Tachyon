/**
 * Tachyon Web Frontend
 * 
 * A modern knowledge management web interface built with:
 * - CodeMirror 6 for markdown editing
 * - HTMX for dynamic interactions
 * - TailwindCSS for styling
 * - neverthrow for functional error handling
 */

// Import styles
import './styles/main.css';

// Import components
import { initializeEditor } from './components/editor';
import { initializeSearch } from './components/search';
import { initializeNavigation } from './components/navigation';
import { initializeTheme } from './components/theme';

// Import utilities
import { initApiClient, getApiClient } from './utils/api';
import type { ApiError } from './utils/api';
import { EventBus } from './utils/events';

// Global application state
interface TachyonAppState {
  events: EventBus;
  isAuthenticated: boolean;
  user: {
    id: string;
    username: string;
    email?: string;
  } | null;
  currentDocument: {
    id: string;
    title: string;
    content: string;
  } | null;
  errors: ApiError[];
}

// Declare global window extension
declare global {
  interface Window {
    Tachyon: TachyonAppState;
  }
}

/**
 * Handle API errors with console logging and optional user notification
 */
function handleApiError(error: ApiError, showNotification = true): void {
  // Store error for debugging
  window.Tachyon.errors.push(error);
  
  // Log detailed error to console
  console.error('[Tachyon Error]', {
    type: error.type,
    status: error.status,
    message: error.message,
    details: error.details,
    timestamp: error.timestamp,
  });
  
  // Show user notification if requested
  if (showNotification) {
    const userMessage = getUserFriendlyErrorMessage(error);
    showErrorNotification(userMessage);
  }
}

/**
 * Convert API error to user-friendly message
 */
function getUserFriendlyErrorMessage(error: ApiError): string {
  switch (error.type) {
    case 'network':
      return 'Network error. Please check your connection and try again.';
    case 'auth':
      return 'Authentication required. Please log in to continue.';
    case 'not_found':
      return 'The requested resource was not found.';
    case 'validation':
      return error.message || 'Invalid request. Please check your input.';
    case 'server':
      return 'A server error occurred. Please try again later.';
    case 'parse':
      return 'An error occurred while processing the response.';
    default:
      return 'An unexpected error occurred. Please try again.';
  }
}

/**
 * Initialize the Tachyon web application
 */
async function initializeApp(): Promise<void> {
  console.log('[Tachyon] Initializing Web Application...');
  const startTime = performance.now();

  // Create event bus for component communication
  const events = new EventBus();

  // Initialize global state
  window.Tachyon = {
    events,
    isAuthenticated: false,
    user: null,
    currentDocument: null,
    errors: [],
  };

  try {
    // Initialize API client
    const apiBaseUrl = (window as any).TACHYON_API_URL || '';
    initApiClient(apiBaseUrl);
    console.log('[Tachyon] API client initialized');

    // Initialize theme (dark/light mode)
    await initializeTheme();
    console.log('[Tachyon] Theme initialized');

    // Check authentication status
    const api = getApiClient();
    const authResult = await api.checkAuth();
    
    authResult.match(
      (authStatus) => {
        if (authStatus.authenticated) {
          window.Tachyon.isAuthenticated = true;
          window.Tachyon.user = authStatus.user || null;
          console.log('[Tachyon] User authenticated:', authStatus.user?.username);
        } else {
          console.log('[Tachyon] User not authenticated');
        }
      },
      (error) => {
        handleApiError(error, false);
        console.warn('[Tachyon] Auth check failed, proceeding as unauthenticated');
      }
    );

    // Initialize components
    initializeNavigation();
    console.log('[Tachyon] Navigation initialized');

    if (document.getElementById('editor')) {
      try {
        await initializeEditor();
        console.log('[Tachyon] Editor initialized');
      } catch (error) {
        console.error('[Tachyon] Failed to initialize editor:', error);
      }
    }

    if (document.getElementById('search')) {
      try {
        initializeSearch();
        console.log('[Tachyon] Search initialized');
      } catch (error) {
        console.error('[Tachyon] Failed to initialize search:', error);
      }
    }

    // Set up HTMX event handlers
    setupHtmxHandlers();

    const elapsed = performance.now() - startTime;
    console.log(`[Tachyon] Application initialized successfully in ${elapsed.toFixed(2)}ms`);
  } catch (error) {
    console.error('[Tachyon] Critical initialization error:', error);
    showErrorNotification('Failed to initialize application. Please refresh the page.');
    
    // Log error details for debugging
    if (error instanceof Error) {
      console.error('[Tachyon] Error stack:', error.stack);
    }
  }
}

/**
 * Set up HTMX event handlers for error handling and CSRF protection
 */
function setupHtmxHandlers(): void {
  document.body.addEventListener('htmx:beforeRequest', (event: any) => {
    // Add CSRF token to requests
    const csrfToken = getCsrfToken();
    if (csrfToken) {
      event.detail.headers['X-CSRF-Token'] = csrfToken;
    }
    console.log('[HTMX] Request started:', event.detail.pathInfo?.requestPath);
  });

  document.body.addEventListener('htmx:afterRequest', (event: any) => {
    const status = event.detail.xhr?.status;
    console.log('[HTMX] Request completed:', event.detail.pathInfo?.requestPath, `(${status})`);
  });

  document.body.addEventListener('htmx:responseError', (event: any) => {
    const status = event.detail.xhr.status;
    const path = event.detail.pathInfo?.requestPath;
    
    console.error('[HTMX] Response error:', {
      path,
      status,
      response: event.detail.xhr.responseText,
    });
    
    if (status === 401) {
      // Redirect to login
      console.warn('[HTMX] Unauthorized, redirecting to login');
      window.location.href = '/login';
    } else if (status === 403) {
      showErrorNotification('You do not have permission to perform this action.');
    } else if (status >= 500) {
      showErrorNotification('A server error occurred. Please try again later.');
    }
  });

  document.body.addEventListener('htmx:sendError', (event: any) => {
    console.error('[HTMX] Send error:', event.detail);
    showErrorNotification('Network error. Please check your connection.');
  });

  console.log('[Tachyon] HTMX handlers registered');
}

/**
 * Get CSRF token from meta tag
 */
function getCsrfToken(): string | null {
  const meta = document.querySelector('meta[name="csrf-token"]');
  return meta ? meta.getAttribute('content') : null;
}

/**
 * Show error notification to user
 */
function showErrorNotification(message: string): void {
  console.warn('[Tachyon] Showing error notification:', message);
  
  const notification = document.createElement('div');
  notification.className = 'fixed top-4 right-4 bg-red-500 text-white px-4 py-2 rounded shadow-lg z-50 animate-fade-in';
  notification.textContent = message;
  notification.setAttribute('role', 'alert');
  document.body.appendChild(notification);

  setTimeout(() => {
    notification.classList.add('animate-fade-out');
    setTimeout(() => notification.remove(), 300);
  }, 5000);
}

/**
 * Global error handler for uncaught errors
 */
function setupGlobalErrorHandler(): void {
  window.onerror = (message, source, lineno, colno, error) => {
    console.error('[Tachyon] Uncaught error:', {
      message,
      source,
      lineno,
      colno,
      error,
    });
    return false;
  };

  window.addEventListener('unhandledrejection', (event) => {
    console.error('[Tachyon] Unhandled promise rejection:', event.reason);
    event.preventDefault();
  });
}

// Set up global error handlers before initialization
setupGlobalErrorHandler();

// Initialize when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initializeApp);
} else {
  initializeApp();
}

// Export for module usage
export { initializeApp, handleApiError, showErrorNotification };
