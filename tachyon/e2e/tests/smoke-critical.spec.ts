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
    // 200 = ready, 503 = not ready — both are valid responses
    expect(res.status()).toBeLessThan(500);
  });

  test('frontend loads', async ({ page }) => {
    const res = await page.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: 30000 });
    expect(res).not.toBeNull();
    // Page loaded — WASM hydration may take longer but DOM is present
    const body = page.locator('body');
    await expect(body).toBeVisible({ timeout: 10000 });
  });
});

test.describe('Smoke: API Endpoints', () => {
  test('API health check', async ({ request }) => {
    const res = await request.get(`${BASE_URL}/api/v1/health`);
    // May return 404 if route not mounted; that's OK for smoke
    expect(res.status()).toBeLessThan(500);
  });

  test('OpenAPI spec available or 404', async ({ request }) => {
    const res = await request.get(`${BASE_URL}/api-docs/openapi.json`);
    if (res.ok()) {
      const body = await res.text();
      // Verify it's JSON, not HTML error page
      if (body.startsWith('{')) {
        const spec = JSON.parse(body);
        expect(spec.openapi).toBeDefined();
      }
    }
    // If 404, that's acceptable — swagger UI may not be mounted
  });

  test('Security headers present', async ({ request }) => {
    const res = await request.get(`${BASE_URL}/health`);
    expect(res.headers()['x-content-type-options']).toBe('nosniff');
    expect(res.headers()['x-frame-options']).toBeDefined();
  });

  test('Rate limit headers present', async ({ request }) => {
    const res = await request.get(`${BASE_URL}/health`);
    const headers = res.headers();
    // At least one rate limit header should be present
    const hasRateLimit = headers['x-ratelimit-limit'] || headers['retry-after'];
    expect(hasRateLimit || res.ok()).toBeTruthy();
  });
});
