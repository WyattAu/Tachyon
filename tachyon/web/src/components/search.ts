/**
 * Search Component
 * Uses neverthrow for functional error handling
 */

import { Events } from '../utils/events';
import { getApiClient } from '../utils/api';

interface SearchOptions {
  debounceMs?: number;
  minQueryLength?: number;
}

interface SearchResult {
  document_id: string;
  title: string;
  snippet?: string;
  tags?: string[];
}

const DEFAULT_OPTIONS: SearchOptions = {
  debounceMs: 300,
  minQueryLength: 2,
};

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

/**
 * Initialize the search component
 */
export function initializeSearch(options: SearchOptions = {}): void {
  const opts = { ...DEFAULT_OPTIONS, ...options };
  const searchInput = document.getElementById('search-input') as HTMLInputElement;
  const searchResults = document.getElementById('search-results');
  const searchForm = document.getElementById('search-form');

  if (!searchInput) {
    console.warn('[Search] Search input element not found');
    return;
  }

  console.log('[Search] Initializing search component');

  // Handle input changes with debouncing
  searchInput.addEventListener('input', (event) => {
    const target = event.target as HTMLInputElement;
    const query = target.value.trim();

    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }

    if (query.length < opts.minQueryLength!) {
      hideSearchResults();
      return;
    }

    debounceTimer = setTimeout(() => {
      performSearch(query);
    }, opts.debounceMs);
  });

  // Handle form submission
  if (searchForm) {
    searchForm.addEventListener('submit', (event) => {
      event.preventDefault();
      const query = searchInput.value.trim();
      if (query.length >= opts.minQueryLength!) {
        performSearch(query);
      }
    });
  }

  // Handle keyboard shortcuts
  document.addEventListener('keydown', (event) => {
    // Ctrl/Cmd + K to focus search
    if ((event.ctrlKey || event.metaKey) && event.key === 'k') {
      event.preventDefault();
      searchInput.focus();
      searchInput.select();
      console.log('[Search] Keyboard shortcut triggered: focus search');
    }

    // Escape to clear search
    if (event.key === 'Escape' && document.activeElement === searchInput) {
      searchInput.value = '';
      hideSearchResults();
      searchInput.blur();
    }
  });

  // Close search results when clicking outside
  document.addEventListener('click', (event) => {
    if (searchResults && !searchResults.contains(event.target as Node) && 
        !searchInput.contains(event.target as Node)) {
      hideSearchResults();
    }
  });

  console.log('[Search] Component initialized successfully');
}

/**
 * Perform search with neverthrow error handling
 */
async function performSearch(query: string): Promise<void> {
  console.log('[Search] Performing search:', query);
  window.Tachyon.events.emit(Events.SEARCH_TRIGGERED, { query });

  const searchResults = document.getElementById('search-results');
  if (!searchResults) {
    console.error('[Search] Search results container not found');
    return;
  }

  // Show loading state
  searchResults.innerHTML = '<div class="p-4 text-gray-500">Searching...</div>';
  searchResults.classList.remove('hidden');

  try {
    const api = getApiClient();
    const result = await api.search(query);

    result.match(
      (response) => {
        const results = response.data?.results || [] as SearchResult[];
        console.log('[Search] Found', results.length, 'results');

        if (results.length === 0) {
          searchResults.innerHTML = '<div class="p-4 text-gray-500">No results found</div>';
          return;
        }

        // Render results
        searchResults.innerHTML = results.map((result: SearchResult) => `
          <a href="/documents/${result.document_id}" 
             class="block p-4 hover:bg-gray-100 dark:hover:bg-gray-800 border-b border-gray-200 dark:border-gray-700 last:border-b-0">
            <div class="font-medium text-gray-900 dark:text-white">${escapeHtml(result.title)}</div>
            <div class="text-sm text-gray-500 dark:text-gray-400 mt-1">${escapeHtml(result.snippet || '')}</div>
            <div class="flex items-center gap-2 mt-2">
              ${result.tags?.map((tag: string) => `
                <span class="px-2 py-0.5 text-xs bg-gray-100 dark:bg-gray-700 rounded">${escapeHtml(tag)}</span>
              `).join('') || ''}
            </div>
          </a>
        `).join('');
      },
      (error) => {
        console.error('[Search] Search failed:', {
          type: error.type,
          status: error.status,
          message: error.message,
          details: error.details,
        });
        
        window.Tachyon.errors.push(error);
        searchResults.innerHTML = `
          <div class="p-4 text-red-500">
            <div class="font-medium">Search failed</div>
            <div class="text-sm mt-1">${error.message}</div>
          </div>
        `;
      }
    );
  } catch (error) {
    console.error('[Search] Unexpected error:', error);
    searchResults.innerHTML = '<div class="p-4 text-red-500">An unexpected error occurred. Please try again.</div>';
  }
}

/**
 * Hide search results
 */
function hideSearchResults(): void {
  const searchResults = document.getElementById('search-results');
  if (searchResults) {
    searchResults.classList.add('hidden');
  }
}

/**
 * Escape HTML to prevent XSS
 */
function escapeHtml(text: string): string {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}
