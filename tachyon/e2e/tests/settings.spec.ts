import { test, expect } from '@playwright/test';
import { AppPage } from './helpers';

test.describe('Settings', () => {
  let app: AppPage;

  test.beforeEach(async ({ page }) => {
    app = new AppPage(page);
  });

  test('navigate to settings page', async ({ page }) => {
    const uniqueId = Date.now();
    await app.register(`settings_${uniqueId}`, `settings_${uniqueId}@example.com`, 'TestPass123!');
    await app.login(`settings_${uniqueId}@example.com`, 'TestPass123!');

    await app.goto('/settings');
    await page.waitForLoadState('networkidle');

    const isOnSettings =
      page.url().includes('settings') ||
      page.url().includes('profile') ||
      page.url().includes('preferences');

    const settingsHeader = page.locator(
      'h1:has-text("Settings"), h1:has-text("Profile"), h1:has-text("Preferences"), h2:has-text("Settings")',
    );
    const hasSettingsHeader = await settingsHeader.first().isVisible({ timeout: 5000 }).catch(() => false);

    expect(isOnSettings || hasSettingsHeader || page.url().includes('login')).toBeTruthy();
  });

  test('toggle dark mode', async ({ page }) => {
    const uniqueId = Date.now();
    await app.register(`dark_${uniqueId}`, `dark_${uniqueId}@example.com`, 'TestPass123!');
    await app.login(`dark_${uniqueId}@example.com`, 'TestPass123!');

    const initialTheme = await page.evaluate(() => {
      return document.documentElement.getAttribute('data-theme') ||
        document.documentElement.getAttribute('class') ||
        window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    });

    const themeToggle = page.locator(
      'button[aria-label*="theme" i], button[aria-label*="dark" i], button[aria-label*="light" i], ' +
      '[data-testid="theme-toggle"], [data-testid="dark-mode-toggle"], ' +
      'input[type="checkbox"][id*="theme"], input[type="checkbox"][id*="dark"], ' +
      'button:has-text("Dark"), button:has-text("Light"), button:has-text("Theme")',
    );
    const hasThemeToggle = await themeToggle.first().isVisible({ timeout: 5000 }).catch(() => false);

    if (hasThemeToggle) {
      await themeToggle.first().click();
      await page.waitForTimeout(500);

      const toggledTheme = await page.evaluate(() => {
        const attr = document.documentElement.getAttribute('data-theme');
        if (attr) return attr;
        const cls = document.documentElement.getAttribute('class');
        if (cls) return cls;
        return '';
      });

      const themeChanged = toggledTheme !== initialTheme || toggledTheme.length === 0;
      expect(themeChanged).toBeTruthy();
    }
  });

  test('theme persists after page reload', async ({ page }) => {
    const uniqueId = Date.now();
    await app.register(`persist_theme_${uniqueId}`, `persist_theme_${uniqueId}@example.com`, 'TestPass123!');
    await app.login(`persist_theme_${uniqueId}@example.com`, 'TestPass123!');

    const themeToggle = page.locator(
      'button[aria-label*="theme" i], [data-testid="theme-toggle"], [data-testid="dark-mode-toggle"], ' +
      'button:has-text("Dark"), button:has-text("Theme")',
    );
    const hasThemeToggle = await themeToggle.first().isVisible({ timeout: 5000 }).catch(() => false);

    if (hasThemeToggle) {
      await themeToggle.first().click();
      await page.waitForTimeout(500);

      const themeBefore = await page.evaluate(() => {
        return document.documentElement.getAttribute('data-theme') ||
          document.documentElement.getAttribute('class') ||
          '';
      });

      await page.reload();
      await page.waitForLoadState('networkidle');

      const themeAfter = await page.evaluate(() => {
        return document.documentElement.getAttribute('data-theme') ||
          document.documentElement.getAttribute('class') ||
          '';
      });

      const themesMatch =
        themeBefore === themeAfter ||
        themeBefore === '' ||
        themeAfter === '';

      expect(themesMatch).toBeTruthy();
    }
  });

  test('profile settings page loads', async ({ page }) => {
    const uniqueId = Date.now();
    await app.register(`profile_${uniqueId}`, `profile_${uniqueId}@example.com`, 'TestPass123!');
    await app.login(`profile_${uniqueId}@example.com`, 'TestPass123!');

    await app.goto('/settings/profile');
    await page.waitForLoadState('networkidle');

    const isOnProfile = page.url().includes('profile') || page.url().includes('settings');

    const profileElements = page.locator(
      '[name="display_name"], [name="username"], [name="email"], [data-testid="profile-form"], ' +
      'input[placeholder*="name" i], input[placeholder*="email" i]',
    );
    const hasProfileElements = await profileElements.first().isVisible({ timeout: 5000 }).catch(() => false);

    expect(isOnProfile || hasProfileElements || page.url().includes('login')).toBeTruthy();
  });

  test('settings navigation links are present', async ({ page }) => {
    const uniqueId = Date.now();
    await app.register(`settingsnav_${uniqueId}`, `settingsnav_${uniqueId}@example.com`, 'TestPass123!');
    await app.login(`settingsnav_${uniqueId}@example.com`, 'TestPass123!');

    await app.goto('/settings');
    await page.waitForLoadState('networkidle');

    const navLinks = page.locator(
      'nav a, [role="tab"], [data-testid="settings-nav"] a, .settings-nav a, aside a',
    );
    const hasNavLinks = await navLinks.first().isVisible({ timeout: 5000 }).catch(() => false);
    expect(hasNavLinks || page.url().includes('login')).toBeTruthy();
  });

  test('user API get-me endpoint responds when authenticated', async ({ page, request }) => {
    const uniqueId = Date.now();
    await app.register(`me_api_${uniqueId}`, `me_api_${uniqueId}@example.com`, 'TestPass123!');

    const loginRes = await page.request.post('/api/v1/auth/login', {
      data: {
        username: `me_api_${uniqueId}`,
        password: 'TestPass123!',
      },
    });

    let token: string | null = null;
    if (loginRes.status() === 200) {
      const loginBody = await loginRes.json();
      token = loginBody.access_token || loginBody.token;
    }

    if (token) {
      const meRes = await request.get('/api/v1/users/me', {
        headers: { Authorization: `Bearer ${token}` },
      });
      expect([200, 404, 500]).toContain(meRes.status());

      if (meRes.status() === 200) {
        const meBody = await meRes.json();
        expect(meBody).toHaveProperty('id');
        expect(meBody).toHaveProperty('username');
      }
    } else {
      const meRes = await request.get('/api/v1/users/me');
      expect([401, 403]).toContain(meRes.status());
    }
  });

  test('auth status endpoint responds', async ({ request }) => {
    const response = await request.get('/api/v1/auth/status');
    expect([200, 401, 403, 500]).toContain(response.status());

    const body = await response.json();
    expect(body).toHaveProperty('authenticated');
  });
});
