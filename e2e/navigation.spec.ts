import { test, expect } from '@playwright/test';

test.describe('Navigation', () => {
  test.beforeEach(async ({ page }) => {
    // Login first
    await page.goto('/');
    await page.fill('input[name="username"]', 'admin');
    await page.fill('input[name="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL(/.*dashboard/);
  });

  test('navigate to dashboard', async ({ page }) => {
    await expect(page).toHaveURL(/.*dashboard/);
    await expect(page.locator('h1')).toContainText('Dashboard');
  });

  test('navigate to documents', async ({ page }) => {
    await page.click('a[href="/documents"]');
    await expect(page).toHaveURL(/.*documents/);
  });

  test('navigate to graph', async ({ page }) => {
    await page.click('a[href="/graph"]');
    await expect(page).toHaveURL(/.*graph/);
  });

  test('navigate to settings', async ({ page }) => {
    await page.click('a[href="/settings"]');
    await expect(page).toHaveURL(/.*settings/);
  });
});
