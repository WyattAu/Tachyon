/**
 * Theme Component - Dark/Light mode toggle
 */

import { Events } from '../utils/events';

type Theme = 'light' | 'dark' | 'system';

const THEME_KEY = 'tachyon_theme';

/**
 * Initialize theme handling
 */
export async function initializeTheme(): Promise<void> {
  const savedTheme = localStorage.getItem(THEME_KEY) as Theme | null;
  const systemPrefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  
  // Determine initial theme
  let theme: Theme;
  if (savedTheme && savedTheme !== 'system') {
    theme = savedTheme;
  } else {
    theme = systemPrefersDark ? 'dark' : 'light';
  }

  // Apply theme
  applyTheme(theme);

  // Setup theme toggle
  setupThemeToggle();

  // Listen for system preference changes
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
    const currentTheme = localStorage.getItem(THEME_KEY) as Theme | null;
    if (currentTheme === 'system' || !currentTheme) {
      applyTheme(e.matches ? 'dark' : 'light');
    }
  });

  console.log('Theme component initialized');
}

/**
 * Apply theme to document
 */
function applyTheme(theme: Theme): void {
  const isDark = theme === 'dark' || 
    (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);

  if (isDark) {
    document.documentElement.classList.add('dark');
  } else {
    document.documentElement.classList.remove('dark');
  }

  // Update meta theme color
  const metaTheme = document.querySelector('meta[name="theme-color"]');
  if (metaTheme) {
    metaTheme.setAttribute('content', isDark ? '#1f2937' : '#ffffff');
  }

  // Emit theme change event
  window.Tachyon.events.emit(Events.THEME_CHANGED, { 
    theme, 
    isDark 
  });
}

/**
 * Setup theme toggle button
 */
function setupThemeToggle(): void {
  const toggleButton = document.getElementById('theme-toggle');
  const toggleIcon = document.getElementById('theme-toggle-icon');

  if (toggleButton) {
    toggleButton.addEventListener('click', () => {
      const currentTheme = localStorage.getItem(THEME_KEY) as Theme | null;
      const isDark = document.documentElement.classList.contains('dark');
      
      const newTheme: Theme = isDark ? 'light' : 'dark';
      localStorage.setItem(THEME_KEY, newTheme);
      applyTheme(newTheme);
      
      updateToggleIcon(newTheme);
    });
  }

  // Set initial icon state
  const savedTheme = localStorage.getItem(THEME_KEY) as Theme | null;
  if (savedTheme) {
    updateToggleIcon(savedTheme);
  }
}

/**
 * Update the theme toggle icon
 */
function updateToggleIcon(theme: Theme): void {
  const toggleIcon = document.getElementById('theme-toggle-icon');
  if (!toggleIcon) return;

  const isDark = theme === 'dark' || 
    (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);

  // Use SVG icons for sun/moon
  if (isDark) {
    toggleIcon.innerHTML = `
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"></path>
      </svg>
    `;
  } else {
    toggleIcon.innerHTML = `
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"></path>
      </svg>
    `;
  }
}

/**
 * Get current theme
 */
export function getCurrentTheme(): Theme {
  return (localStorage.getItem(THEME_KEY) as Theme) || 'system';
}

/**
 * Set theme
 */
export function setTheme(theme: Theme): void {
  localStorage.setItem(THEME_KEY, theme);
  applyTheme(theme);
  updateToggleIcon(theme);
}
