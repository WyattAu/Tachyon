/**
 * Tachyon Web Frontend
 * 
 * A modern knowledge management web interface built with:
 * - CodeMirror 6 for markdown editing
 * - HTMX for dynamic interactions
 * - TailwindCSS for styling
 */

// Import styles
import './styles/main.css';

// Import components
import { initializeEditor } from './components/editor';
import { initializeSearch } from './components/search';
import { initializeNavigation } from './components/navigation';
import { initializeTheme } from './components/theme';

// Import utilities
import { ApiClient } from './utils/api';
import { EventBus } from './utils/events';

// Global application state
interface TachyonAppState {
  api: ApiClient;
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
}

// Declare global window extension
declare global {
  interface Window {
    Tachyon: TachyonAppState;
  }
}

/**
 * Initialize the Tachyon web application
 */
async function initializeApp(): Promise<void> {
  console.log('Initializing Tachyon Web Application...');

  // Create API client
  const apiBaseUrl = (window as any).TACHYON_API_URL || '/api/v1';
  const api = new ApiClient(apiBaseUrl);

  // Create event bus for component communication
  const events = new EventBus();

  // Initialize global state
  window.Tachyon = {
    api,
    events,
    isAuthenticated: false,
    user: null,
    currentDocument: null,
  };

  try {
    // Initialize theme (dark/light mode)
    await initializeTheme();

    // Check authentication status
    const authStatus = await api.checkAuth();
    if (authStatus.authenticated) {
      window.Tachyon.isAuthenticated = true;
      window.Tachyon.user = authStatus.user || null;
    }

    // Initialize components
    initializeNavigation();
    
    if (document.getElementById('editor')) {
      await initializeEditor();
    }

    if (document.getElementById('search')) {
      initializeSearch();
    }

    // Set up HTMX event handlers
    setupHtmxHandlers();

    console.log('Tachyon Web Application initialized successfully');
  } catch (error) {
    console.error('Failed to initialize Tachyon:', error);
    showErrorNotification('Failed to initialize application. Please refresh the page.');
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
  });

  document.body.addEventListener('htmx:responseError', (event: any) => {
    const status = event.detail.xhr.status;
    if (status === 401) {
      // Redirect to login
      window.location.href = '/login';
    } else if (status === 403) {
      showErrorNotification('You do not have permission to perform this action.');
    } else if (status >= 500) {
      showErrorNotification('A server error occurred. Please try again later.');
    }
  });

  document.body.addEventListener('htmx:sendError', () => {
    showErrorNotification('Network error. Please check your connection.');
  });
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
  const notification = document.createElement('div');
  notification.className = 'fixed top-4 right-4 bg-red-500 text-white px-4 py-2 rounded shadow-lg z-50 animate-fade-in';
  notification.textContent = message;
  document.body.appendChild(notification);

  setTimeout(() => {
    notification.classList.add('animate-fade-out');
    setTimeout(() => notification.remove(), 300);
  }, 5000);
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initializeApp);
} else {
  initializeApp();
}

// Export for module usage
export { initializeApp };
