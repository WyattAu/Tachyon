import { test, expect } from '@playwright/test';

test.describe('Documents', () => {
  test.beforeEach(async ({ page }) => {
    // Login first
    await page.goto('/');
    await page.fill('input[name="username"]', 'admin');
    await page.fill('input[name="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL(/.*dashboard/);
  });

  test('create new document', async ({ page }) => {
    await page.click('a[href="/documents"]');
    await page.click('button:has-text("New Document")');
    await page.fill('input[name="title"]', 'E2E Test Document');
    await page.fill('textarea[name="content"]', '# Test\n\nThis is an E2E test document.');
    await page.click('button:has-text("Save")');
    await expect(page.locator('.success')).toBeVisible();
  });

  test('list documents', async ({ page }) => {
    await page.click('a[href="/documents"]');
    await expect(page.locator('.document-list')).toBeVisible();
  });

  test('search documents', async ({ page }) => {
    await page.fill('input[name="search"]', 'test');
    await page.press('input[name="search"]', 'Enter');
    await expect(page.locator('.search-results')).toBeVisible();
  });
});
