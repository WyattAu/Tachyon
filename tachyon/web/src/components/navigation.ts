/**
 * Navigation Component
 */

import { Events } from '../utils/events';

/**
 * Initialize the navigation component
 */
export function initializeNavigation(): void {
  setupMobileMenu();
  setupDropdowns();
  setupKeyboardShortcuts();
  setupAuthState();

  console.log('Navigation component initialized');
}

/**
 * Setup mobile menu toggle
 */
function setupMobileMenu(): void {
  const menuButton = document.getElementById('mobile-menu-button');
  const mobileMenu = document.getElementById('mobile-menu');

  if (menuButton && mobileMenu) {
    menuButton.addEventListener('click', () => {
      mobileMenu.classList.toggle('hidden');
    });

    // Close menu when clicking outside
    document.addEventListener('click', (event) => {
      if (!menuButton.contains(event.target as Node) && 
          !mobileMenu.contains(event.target as Node)) {
        mobileMenu.classList.add('hidden');
      }
    });
  }
}

/**
 * Setup dropdown menus
 */
function setupDropdowns(): void {
  const dropdowns = document.querySelectorAll('[data-dropdown]');

  dropdowns.forEach((dropdown) => {
    const toggle = dropdown.querySelector('[data-dropdown-toggle]');
    const content = dropdown.querySelector('[data-dropdown-content]');

    if (toggle && content) {
      toggle.addEventListener('click', (event) => {
        event.stopPropagation();
        content.classList.toggle('hidden');
      });
    }
  });

  // Close dropdowns when clicking outside
  document.addEventListener('click', () => {
    document.querySelectorAll('[data-dropdown-content]').forEach((content) => {
      content.classList.add('hidden');
    });
  });
}

/**
 * Setup keyboard shortcuts
 */
function setupKeyboardShortcuts(): void {
  document.addEventListener('keydown', (event) => {
    // Ctrl/Cmd + S to save
    if ((event.ctrlKey || event.metaKey) && event.key === 's') {
      event.preventDefault();
      window.Tachyon.events.emit('document:save');
    }

    // Ctrl/Cmd + N for new document
    if ((event.ctrlKey || event.metaKey) && event.key === 'n') {
      event.preventDefault();
      window.location.href = '/documents/new';
    }

    // Ctrl/Cmd + Shift + D for dashboard
    if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key === 'D') {
      event.preventDefault();
      window.location.href = '/';
    }
  });
}

/**
 * Setup authentication state handling
 */
function setupAuthState(): void {
  updateAuthUI();

  window.Tachyon.events.on(Events.AUTH_CHANGED, () => {
    updateAuthUI();
  });
}

/**
 * Update UI based on authentication state
 */
function updateAuthUI(): void {
  const isAuthenticated = window.Tachyon.isAuthenticated;
  
  // Toggle visibility of auth-dependent elements
  document.querySelectorAll('[data-auth-required]').forEach((el) => {
    if (isAuthenticated) {
      el.classList.remove('hidden');
    } else {
      el.classList.add('hidden');
    }
  });

  document.querySelectorAll('[data-guest-only]').forEach((el) => {
    if (isAuthenticated) {
      el.classList.add('hidden');
    } else {
      el.classList.remove('hidden');
    }
  });

  // Update user menu
  const userMenu = document.getElementById('user-menu');
  if (userMenu && window.Tachyon.user) {
    const userName = userMenu.querySelector('[data-user-name]');
    const userEmail = userMenu.querySelector('[data-user-email]');
    
    if (userName) userName.textContent = window.Tachyon.user.username;
    if (userEmail) userEmail.textContent = window.Tachyon.user.email || '';
  }
}
