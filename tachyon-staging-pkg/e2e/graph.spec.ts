import { test, expect } from '@playwright/test';

test.describe('Knowledge Graph', () => {
  test.beforeEach(async ({ page }) => {
    // Login first
    await page.goto('/');
    await page.fill('input[name="username"]', 'admin');
    await page.fill('input[name="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL(/.*dashboard/);
  });

  test('graph page loads', async ({ page }) => {
    await page.click('a[href="/graph"]');
    await expect(page.locator('.graph-container')).toBeVisible();
  });

  test('graph shows nodes', async ({ page }) => {
    await page.click('a[href="/graph"]');
    await expect(page.locator('.graph-node')).toHaveCount({ minimum: 0 });
  });
});
