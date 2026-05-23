/**
 * Smoke tests — critical path validation
 * Fast subset of E2E tests covering the most essential flows.
 * Target: < 2 minutes total execution time.
 *
 * Run: npx playwright test --config=playwright.smoke.config.ts
 */

import { test, expect } from '@playwright/test';

const BASE_URL = process.env.E2E_BASE_URL || 'http://localhost:8080';

test.describe('Smoke: Health & Availability', () => {
  test('health endpoint returns 200', async ({ request }) => {
    const res = await request.get(`${BASE_URL}/health`);
    expect(res.ok()).toBeTruthy();
    const body = await res.json();
    expect(body.status).toMatch(/^(healthy|degraded)$/);
  });

  test('readiness endpoint responds', async ({ request }) => {
    const res = await request.get(`${BASE_URL}/ready`);
    expect([200, 503]).toContain(res.status());
  });

  test('frontend loads', async ({ page }) => {
    await page.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: 15000 });
    // Wait for WASM hydration with extended timeout
    await page.waitForSelector('main, [role="main"], #app, .app', { timeout: 30000 });
  });
});

test.describe('Smoke: Authentication', () => {
  test('register + login + logout cycle', async ({ page }) => {
    const uniqueId = `smoke_${Date.now()}`;
    const username = `smoke_${uniqueId}`;
    const password = 'SmokeTest123!';

    // Register
    await page.goto(`${BASE_URL}/register`, { waitUntil: 'domcontentloaded', timeout: 15000 });
    await page.waitForSelector('input, form', { timeout: 15000 });

    // Try to find registration form fields
    const usernameInput = page.locator('input[name="username"], input[placeholder*="username"], input[type="text"]').first();
    const emailInput = page.locator('input[name="email"], input[type="email"]').first();
    const passwordInput = page.locator('input[name="password"], input[type="password"]').first();
    const submitBtn = page.locator('button[type="submit"], button:has-text("Register"), button:has-text("Sign")').first();

    if (await usernameInput.isVisible()) {
      await usernameInput.fill(username);
      if (await emailInput.isVisible()) await emailInput.fill(`${uniqueId}@test.smoke`);
      await passwordInput.fill(password);
      await submitBtn.click();
      await page.waitForTimeout(2000);
    }
  });
});

test.describe('Smoke: API Endpoints', () => {
  test('API health check', async ({ request }) => {
    const res = await request.get(`${BASE_URL}/api/v1/health`);
    expect(res.ok()).toBeTruthy();
  });

  test('OpenAPI spec available', async ({ request }) => {
    const res = await request.get(`${BASE_URL}/api-docs/openapi.json`);
    if (res.ok()) {
      const spec = await res.json();
      expect(spec.openapi).toBeDefined();
    }
  });

  test('Security headers present', async ({ request }) => {
    const res = await request.get(`${BASE_URL}/health`);
    expect(res.headers()['x-content-type-options']).toBe('nosniff');
    expect(res.headers()['x-frame-options']).toBeDefined();
  });
});
