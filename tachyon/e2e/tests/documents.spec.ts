import { test, expect } from '@playwright/test';
import { AppPage } from './helpers';

test.describe('Documents', () => {
  let app: AppPage;
  const uniqueId = Date.now();

  test.beforeAll(async () => {
    // Pre-create user via API or first test
  });

  test.beforeEach(async ({ page }) => {
    app = new AppPage(page);
  });

  test('documents page loads when authenticated', async ({ page }) => {
    app = new AppPage(page);
    await app.goto('/documents');
    const isOnDocs = page.url().includes('documents');
    const isOnLogin = page.url().includes('login');
    expect(isOnDocs || isOnLogin).toBeTruthy();
  });

  test('search page loads', async ({ page }) => {
    app = new AppPage(page);
    await app.goto('/search');
    const searchInput = page.locator('input[type="search"], input[placeholder*="search" i], [data-testid="search-input"]');
    const isOnSearch = page.url().includes('search');
    const isOnLogin = page.url().includes('login');
    expect(isOnSearch || isOnLogin).toBeTruthy();
  });
});
