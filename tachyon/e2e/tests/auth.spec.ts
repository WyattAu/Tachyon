import { test, expect } from '@playwright/test';
import { AppPage } from './helpers';

test.describe('Authentication', () => {
  test('register page loads', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/register');
    await expect(page.locator('h1, h2').first()).toBeVisible();
  });

  test('login page loads', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/login');
    await expect(page.locator('h1, h2').first()).toBeVisible();
  });

  test('unauthenticated redirect to login', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/documents');
    await page.waitForURL('**/login**', { timeout: 5000 }).catch(() => {});
    const loginForm = page.locator('[name="email"], [name="password"]');
    const isLoginVisible = await loginForm.first().isVisible().catch(() => false);
    expect(isLoginVisible || page.url().includes('login')).toBeTruthy();
  });

  test('full auth flow: register → login → dashboard', async ({ page }) => {
    const app = new AppPage(page);
    const uniqueId = Date.now();
    await app.register(`testuser_${uniqueId}`, `test_${uniqueId}@example.com`, 'TestPass123!');
    await app.login(`test_${uniqueId}@example.com`, 'TestPass123!');
    await expect(page).toHaveURL(/\/(documents|spaces|home)?$/);
  });
});
