import { test, expect, Page } from '@playwright/test';

test.describe('Authentication Flow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should display login page', async ({ page }) => {
    await page.goto('/login');
    
    await expect(page.locator('h1, h2')).toContainText(/login|sign in/i);
    await expect(page.locator('input[type="email"]')).toBeVisible();
    await expect(page.locator('input[type="password"]')).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });

  test('should show error for invalid credentials', async ({ page }) => {
    await page.goto('/login');
    
    await page.fill('input[type="email"]', 'invalid@example.com');
    await page.fill('input[type="password"]', 'wrongpassword');
    await page.click('button[type="submit"]');
    
    await expect(page.locator('[role="alert"], .error-message')).toBeVisible({ timeout: 5000 });
  });

  test('should validate email format', async ({ page }) => {
    await page.goto('/login');
    
    await page.fill('input[type="email"]', 'not-an-email');
    await page.fill('input[type="password"]', 'somepassword');
    await page.click('button[type="submit"]');
    
    const emailInput = page.locator('input[type="email"]');
    const isValid = await emailInput.evaluate((el: HTMLInputElement) => el.validity.valid);
    expect(isValid).toBe(false);
  });

  test('should require password field', async ({ page }) => {
    await page.goto('/login');
    
    await page.fill('input[type="email"]', 'test@example.com');
    await page.click('button[type="submit"]');
    
    const passwordInput = page.locator('input[type="password"]');
    const isRequired = await passwordInput.evaluate((el: HTMLInputElement) => el.required);
    expect(isRequired).toBe(true);
  });

  test('should successfully login with valid credentials', async ({ page }) => {
    await page.goto('/login');
    
    await page.fill('input[type="email"]', 'test@example.com');
    await page.fill('input[type="password"]', 'ValidPassword123!');
    await page.click('button[type="submit"]');
    
    await page.waitForURL(/\/(documents|dashboard|home)/, { timeout: 10000 });
    
    await expect(page).toHaveURL(/\/(documents|dashboard|home)/);
  });

  test('should navigate to registration page', async ({ page }) => {
    await page.goto('/login');
    
    await page.click('a[href*="register"], a:has-text("Sign up")');
    
    await expect(page).toHaveURL(/\/register/);
  });

  test('should display registration form', async ({ page }) => {
    await page.goto('/register');
    
    await expect(page.locator('input[type="email"]')).toBeVisible();
    await expect(page.locator('input[type="password"]')).toBeVisible();
    await expect(page.locator('input[name="username"], input[id="username"]')).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });

  test('should register new user successfully', async ({ page }) => {
    const timestamp = Date.now();
    await page.goto('/register');
    
    await page.fill('input[name="username"], input[id="username"]', `testuser_${timestamp}`);
    await page.fill('input[type="email"]', `test_${timestamp}@example.com`);
    await page.fill('input[type="password"]', 'SecurePassword123!');
    
    const confirmPassword = page.locator('input[name="confirmPassword"], input[id="confirmPassword"]');
    if (await confirmPassword.isVisible()) {
      await confirmPassword.fill('SecurePassword123!');
    }
    
    await page.click('button[type="submit"]');
    
    await page.waitForURL(/\/(documents|dashboard|verify)/, { timeout: 10000 });
  });

  test('should logout successfully', async ({ page }) => {
    await page.goto('/login');
    await page.fill('input[type="email"]', 'test@example.com');
    await page.fill('input[type="password"]', 'ValidPassword123!');
    await page.click('button[type="submit"]');
    
    await page.waitForURL(/\/(documents|dashboard)/, { timeout: 10000 });
    
    await page.click('button[aria-label="User menu"], button:has-text("Logout")');
    await page.click('button:has-text("Logout"), a:has-text("Logout")');
    
    await expect(page).toHaveURL(/\/(login|\/)/);
  });

  test('should maintain session across page reloads', async ({ page }) => {
    await page.goto('/login');
    await page.fill('input[type="email"]', 'test@example.com');
    await page.fill('input[type="password"]', 'ValidPassword123!');
    await page.click('button[type="submit"]');
    
    await page.waitForURL(/\/(documents|dashboard)/, { timeout: 10000 });
    
    await page.reload();
    
    await expect(page).not.toHaveURL(/\/login/);
  });

  test('should show password reset link', async ({ page }) => {
    await page.goto('/login');
    
    await page.click('a:has-text("Forgot password"), a:has-text("Reset password")');
    
    await expect(page).toHaveURL(/\/(forgot-password|reset-password)/);
  });

  test('should request password reset', async ({ page }) => {
    await page.goto('/forgot-password');
    
    await page.fill('input[type="email"]', 'test@example.com');
    await page.click('button[type="submit"]');
    
    await expect(page.locator('.success-message, [role="status"]')).toBeVisible({ timeout: 5000 });
  });

  test('should protect authenticated routes', async ({ page }) => {
    await page.goto('/documents');
    
    await expect(page).toHaveURL(/\/login/);
  });

  test('should display user profile when authenticated', async ({ page }) => {
    await page.goto('/login');
    await page.fill('input[type="email"]', 'test@example.com');
    await page.fill('input[type="password"]', 'ValidPassword123!');
    await page.click('button[type="submit"]');
    
    await page.waitForURL(/\/(documents|dashboard)/, { timeout: 10000 });
    
    await page.goto('/settings');
    
    await expect(page.locator('input[type="email"], [data-testid="user-email"]')).toBeVisible();
  });
});

test.describe('OAuth Integration', () => {
  test('should display OAuth login options', async ({ page }) => {
    await page.goto('/login');
    
    const oauthButtons = page.locator('button:has-text("Google"), button:has-text("GitHub"), button:has-text("OAuth")');
    const count = await oauthButtons.count();
    
    expect(count).toBeGreaterThan(0);
  });

  test('should handle OAuth callback', async ({ page }) => {
    await page.goto('/auth/callback?code=test_code&state=test_state');
    
    await page.waitForURL(/\/(documents|dashboard|error)/, { timeout: 10000 });
  });
});
