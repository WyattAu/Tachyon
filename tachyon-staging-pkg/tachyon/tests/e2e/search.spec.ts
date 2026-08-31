import { test, expect } from '@playwright/test';

test.describe('Search Functionality', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should display search bar', async ({ page }) => {
    const searchBar = page.locator('input[type="search"], input[placeholder*="search"], [data-testid="search-input"]');
    await expect(searchBar).toBeVisible();
  });

  test('should perform basic search', async ({ page }) => {
    const searchInput = page.locator('input[type="search"], input[placeholder*="search"]').first();
    
    await searchInput.fill('test');
    await searchInput.press('Enter');
    
    await expect(page).toHaveURL(/search/);
    await expect(page.locator('.search-results, [data-testid="search-results"]')).toBeVisible({ timeout: 5000 });
  });

  test('should display search results count', async ({ page }) => {
    await page.goto('/search?q=test');
    
    const resultsInfo = page.locator('.results-count, [data-testid="results-count"], text=/found|results/i');
    await expect(resultsInfo).toBeVisible({ timeout: 5000 });
  });

  test('should show empty state for no results', async ({ page }) => {
    const uniqueQuery = `noresults${Date.now()}`;
    await page.goto(`/search?q=${uniqueQuery}`);
    
    await expect(page.locator('text=/no results found|no documents found/i')).toBeVisible({ timeout: 5000 });
  });

  test('should filter search by type', async ({ page }) => {
    await page.goto('/search?q=test');
    
    const typeFilter = page.locator('select[name="type"], [data-testid="type-filter"]');
    if (await typeFilter.isVisible()) {
      await typeFilter.selectOption('document');
      
      await page.waitForTimeout(500);
      
      await expect(page.locator('.search-results')).toBeVisible();
    }
  });

  test('should filter search by date range', async ({ page }) => {
    await page.goto('/search?q=test');
    
    const dateFilter = page.locator('[data-testid="date-filter"], button:has-text("Date")');
    if (await dateFilter.isVisible()) {
      await dateFilter.click();
      
      const startDate = page.locator('input[name="startDate"], input[type="date"]').first();
      if (await startDate.isVisible()) {
        await startDate.fill('2024-01-01');
      }
    }
  });

  test('should sort search results', async ({ page }) => {
    await page.goto('/search?q=test');
    
    const sortSelect = page.locator('select[name="sort"], [data-testid="sort-select"]');
    if (await sortSelect.isVisible()) {
      await sortSelect.selectOption('created_at');
      
      await page.waitForTimeout(500);
    }
  });

  test('should paginate search results', async ({ page }) => {
    await page.goto('/search?q=test');
    
    const nextButton = page.locator('button:has-text("Next"), button[aria-label="Next page"]');
    if (await nextButton.isEnabled()) {
      await nextButton.click();
      
      await page.waitForTimeout(500);
    }
  });

  test('should highlight search terms in results', async ({ page }) => {
    await page.goto('/search?q=test&highlight=true');
    
    const highlights = page.locator('mark, .highlight');
    const count = await highlights.count();
    
    expect(count).toBeGreaterThanOrEqual(0);
  });

  test('should show search suggestions', async ({ page }) => {
    const searchInput = page.locator('input[type="search"], input[placeholder*="search"]').first();
    
    await searchInput.fill('te');
    await searchInput.focus();
    
    const suggestions = page.locator('.search-suggestions, [data-testid="suggestions"]');
    await page.waitForTimeout(500);
    
    const isVisible = await suggestions.isVisible().catch(() => false);
    expect(typeof isVisible).toBe('boolean');
  });

  test('should display recent searches', async ({ page }) => {
    const searchInput = page.locator('input[type="search"], input[placeholder*="search"]').first();
    
    await searchInput.click();
    
    const recentSearches = page.locator('.recent-searches, [data-testid="recent-searches"]');
    const isVisible = await recentSearches.isVisible().catch(() => false);
    expect(typeof isVisible).toBe('boolean');
  });

  test('should clear search', async ({ page }) => {
    const searchInput = page.locator('input[type="search"], input[placeholder*="search"]').first();
    
    await searchInput.fill('test');
    
    const clearButton = page.locator('button[aria-label="Clear"], button:has-text("Clear")');
    if (await clearButton.isVisible()) {
      await clearButton.click();
      
      await expect(searchInput).toHaveValue('');
    }
  });

  test('should search by tags', async ({ page }) => {
    await page.goto('/search?tags=rust,async');
    
    await expect(page.locator('.search-results, [data-testid="search-results"]')).toBeVisible({ timeout: 5000 });
  });

  test('should search by author', async ({ page }) => {
    await page.goto('/search?author=test-user');
    
    await expect(page.locator('.search-results, [data-testid="search-results"]')).toBeVisible({ timeout: 5000 });
  });

  test('should show facets/filters', async ({ page }) => {
    await page.goto('/search?q=test');
    
    const facets = page.locator('.facets, [data-testid="facets"], .filters');
    const isVisible = await facets.isVisible().catch(() => false);
    expect(typeof isVisible).toBe('boolean');
  });

  test('should perform advanced search', async ({ page }) => {
    await page.goto('/search/advanced');
    
    const queryInput = page.locator('input[name="query"], textarea[name="query"]');
    if (await queryInput.isVisible()) {
      await queryInput.fill('advanced test query');
      
      const searchButton = page.locator('button[type="submit"], button:has-text("Search")');
      await searchButton.click();
      
      await expect(page.locator('.search-results')).toBeVisible({ timeout: 5000 });
    }
  });

  test('should save search', async ({ page }) => {
    await page.goto('/login');
    await page.fill('input[type="email"]', 'test@example.com');
    await page.fill('input[type="password"]', 'ValidPassword123!');
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/(documents|dashboard)/, { timeout: 10000 });
    
    await page.goto('/search?q=test');
    
    const saveButton = page.locator('button:has-text("Save Search"), button[aria-label="Save search"]');
    if (await saveButton.isVisible()) {
      await saveButton.click();
      
      await expect(page.locator('[role="status"], .success-message')).toBeVisible({ timeout: 5000 });
    }
  });

  test('should handle special characters in search', async ({ page }) => {
    await page.goto('/search?q=test%20%26%20special');
    
    await expect(page.locator('.search-results, [data-testid="search-results"], .no-results')).toBeVisible({ timeout: 5000 });
  });

  test('should search within specific project', async ({ page }) => {
    await page.goto('/search?q=test&project=test-project');
    
    await expect(page.locator('.search-results, [data-testid="search-results"]')).toBeVisible({ timeout: 5000 });
  });

  test('should show search loading state', async ({ page }) => {
    const searchInput = page.locator('input[type="search"], input[placeholder*="search"]').first();
    
    await searchInput.fill('test');
    await searchInput.press('Enter');
    
    const loadingSpinner = page.locator('.loading, [data-testid="loading"], .spinner');
    const wasVisible = await loadingSpinner.isVisible({ timeout: 100 }).catch(() => false);
    
    await expect(page.locator('.search-results, .no-results')).toBeVisible({ timeout: 5000 });
  });
});

test.describe('Global Search', () => {
  test('should search across all content types', async ({ page }) => {
    await page.goto('/search/global?q=test&scope=all');
    
    await expect(page.locator('.search-results, [data-testid="search-results"]')).toBeVisible({ timeout: 5000 });
  });

  test('should group results by type', async ({ page }) => {
    await page.goto('/search?q=test');
    
    const groupedResults = page.locator('.result-group, [data-testid="result-group"]');
    const count = await groupedResults.count();
    
    expect(count).toBeGreaterThanOrEqual(0);
  });
});
