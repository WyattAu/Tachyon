import { test, expect } from '@playwright/test';
import { AppPage } from './helpers';

test.describe('Navigation', () => {
  test('home page loads', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');
    await expect(page).toHaveTitle(/Tachyon|Wiki/);
  });

  test('404 page for invalid route', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/this-does-not-exist');
    const has404 = await page.locator('text=/404|Not Found|Page not found/i').isVisible().catch(() => false);
    const isHome = page.url() === '/' || page.url().endsWith('/');
    expect(has404 || isHome).toBeTruthy();
  });

  test('navigation links are present', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');
    const nav = page.locator('nav, header, [role="navigation"]');
    const hasNav = await nav.isVisible().catch(() => false);
    expect(hasNav).toBeTruthy();
  });
});
