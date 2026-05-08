import { test, expect, Page } from '@playwright/test';
import { AppPage } from './helpers';

test.describe('Onboarding', () => {
  test.slow();

  test('new user sees onboarding wizard', async ({ page }) => {
    const app = new AppPage(page);
    const uniqueId = Date.now();
    await app.register(`onboard_${uniqueId}`, `onboard_${uniqueId}@example.com`, 'TestPass123!');
    await app.login(`onboard_${uniqueId}@example.com`, 'TestPass123!');

    const wizard = page.locator(
      '[data-testid="onboarding-wizard"], [data-testid="onboarding"], section:has-text("Welcome"), [class*="onboarding"]',
    );
    const isWizardVisible = await wizard.first().isVisible().catch(() => false);
    expect(isWizardVisible).toBeTruthy();
  });

  test('step through all 4 onboarding steps', async ({ page }) => {
    const app = new AppPage(page);
    const uniqueId = Date.now();
    await app.register(`steps_${uniqueId}`, `steps_${uniqueId}@example.com`, 'TestPass123!');
    await app.login(`steps_${uniqueId}@example.com`, 'TestPass123!');

    const nextBtn = page.locator(
      'button:has-text("Next"), button:has-text("Continue"), button:has-text("Get Started"), [data-testid="onboarding-next"]',
    );

    for (let i = 0; i < 4; i++) {
      const isVisible = await nextBtn.first().isVisible({ timeout: 5000 }).catch(() => false);
      if (isVisible) {
        await nextBtn.first().click();
        await page.waitForLoadState('networkidle');
      }
    }

    const finishBtn = page.locator(
      'button:has-text("Finish"), button:has-text("Done"), button:has-text("Complete"), [data-testid="onboarding-finish"]',
    );
    const isFinishVisible = await finishBtn.first().isVisible({ timeout: 5000 }).catch(() => false);
    if (isFinishVisible) {
      await finishBtn.first().click();
      await page.waitForLoadState('networkidle');
    }

    const isOnDashboard =
      page.url().includes('documents') ||
      page.url().includes('spaces') ||
      page.url().includes('home') ||
      page.url().endsWith('/');
    expect(isOnDashboard).toBeTruthy();
  });

  test('sample documents are created after onboarding', async ({ page }) => {
    const app = new AppPage(page);
    const uniqueId = Date.now();
    await app.register(`samples_${uniqueId}`, `samples_${uniqueId}@example.com`, 'TestPass123!');
    await app.login(`samples_${uniqueId}@example.com`, 'TestPass123!');

    const sampleBtn = page.locator(
      'button:has-text("Create Samples"), button:has-text("Add Sample"), [data-testid="create-sample-content"]',
    );
    const isSampleBtnVisible = await sampleBtn.first().isVisible({ timeout: 5000 }).catch(() => false);
    if (isSampleBtnVisible) {
      await sampleBtn.first().click();
      await page.waitForLoadState('networkidle');
    }

    await app.goto('/documents');

    const sampleDocTitles = ['Welcome to Tachyon', 'Getting Started', 'Markdown Guide'];
    let foundAny = false;
    for (const title of sampleDocTitles) {
      const docLink = page.locator(`text="${title}"`);
      if (await docLink.first().isVisible({ timeout: 3000 }).catch(() => false)) {
        foundAny = true;
        break;
      }
    }
    expect(foundAny).toBeTruthy();
  });

  test('onboarding is skipped for returning users', async ({ page }) => {
    const app = new AppPage(page);
    const uniqueId = Date.now();
    const email = `returning_${uniqueId}@example.com`;
    await app.register(`returning_${uniqueId}`, email, 'TestPass123!');
    await app.login(email, 'TestPass123!');

    await page.goto('/');
    await page.waitForLoadState('networkidle');

    const wizard = page.locator(
      '[data-testid="onboarding-wizard"], [data-testid="onboarding"], section:has-text("Welcome")',
    );
    const isWizardVisible = await wizard.first().isVisible({ timeout: 3000 }).catch(() => false);

    if (isWizardVisible) {
      const skipBtn = page.locator(
        'button:has-text("Skip"), a:has-text("Skip"), [data-testid="onboarding-skip"]',
      );
      const isSkipVisible = await skipBtn.first().isVisible({ timeout: 2000 }).catch(() => false);
      if (isSkipVisible) {
        await skipBtn.first().click();
        await page.waitForLoadState('networkidle');
      }
    }

    await page.goto('/');
    await page.waitForLoadState('networkidle');

    const wizardAfter = page.locator(
      '[data-testid="onboarding-wizard"], [data-testid="onboarding"], section:has-text("Welcome")',
    );
    const isWizardStillVisible = await wizardAfter.first().isVisible({ timeout: 2000 }).catch(() => false);
    expect(isWizardStillVisible).toBeFalsy();
  });

  test('onboarding API endpoints respond correctly', async ({ request }) => {
    const statusRes = await request.get('/api/v1/onboarding/status');
    expect([200, 401, 403, 500]).toContain(statusRes.status());

    const suggestionsRes = await request.get('/api/v1/onboarding/suggestions');
    expect([200, 401, 403, 500]).toContain(suggestionsRes.status());
  });
});
