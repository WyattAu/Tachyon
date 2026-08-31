import { defineConfig } from '@playwright/test';

// Smoke tests: fast subset covering critical paths only
// Run with: npx playwright test --config=playwright.smoke.config.ts
export default defineConfig({
  testDir: './tests',
  testMatch: 'smoke-*.spec.ts',
  timeout: 30000,
  retries: 0,
  use: {
    baseURL: process.env.E2E_BASE_URL || 'http://localhost:8080',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    actionTimeout: 10000,
    navigationTimeout: 15000,
    expect: { timeout: 5000 },
  },
  projects: [
    { name: 'chromium', use: { browserName: 'chromium' } },
  ],
});
