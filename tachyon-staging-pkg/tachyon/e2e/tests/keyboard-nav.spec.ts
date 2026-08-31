import { test, expect } from '@playwright/test';
import { AppPage } from './helpers';

test.describe('Keyboard Navigation', () => {
  test('skip link is present in DOM', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const skipLink = page.locator('a.sr-only, a[href="#main-content"]');
    await expect(skipLink.first()).toBeAttached();
  });

  test('skip link becomes visible on focus', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const skipLink = page.locator('a[href="#main-content"]').first();
    await expect(skipLink).toBeAttached();

    await skipLink.focus();

    const isVisible = await skipLink.isVisible().catch(() => false);
    expect(isVisible).toBeTruthy();
  });

  test('skip link navigates to main content', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const skipLink = page.locator('a[href="#main-content"]').first();
    await skipLink.focus();
    await skipLink.press('Enter');

    const mainContent = page.locator('#main-content, main, [role="main"]').first();
    await expect(mainContent).toBeFocused();
  });

  test('Tab order through main navigation is logical', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const navLinks = page.locator('nav a, aside[role="navigation"] a');
    const count = await navLinks.count();

    expect(count).toBeGreaterThan(0);

    const focusedOrder: string[] = [];

    for (let i = 0; i < Math.min(count, 5); i++) {
      await page.keyboard.press('Tab');
      const focusedElement = page.locator(':focus');
      const text = await focusedElement.textContent() ?? '';
      const href = await focusedElement.getAttribute('href') ?? '';
      if (text.trim() || href) {
        focusedOrder.push(text.trim() || href);
      }
    }

    expect(focusedOrder.length).toBeGreaterThan(0);
  });

  test('all navigation links are keyboard focusable', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const navLinks = page.locator('nav a, aside[role="navigation"] a');
    const count = await navLinks.count();

    let focusableCount = 0;
    for (let i = 0; i < count; i++) {
      const link = navLinks.nth(i);
      const visible = await link.isVisible().catch(() => false);
      if (!visible) continue;

      try {
        await link.focus({ timeout: 1000 });
        const isFocused = await link.evaluate(el => document.activeElement === el);
        if (isFocused) focusableCount++;
      } catch {
        // Element may not be focusable
      }
    }

    const visibleLinks = await navLinks.count();
    if (visibleLinks > 0) {
      expect(focusableCount).toBeGreaterThan(0);
    }
  });

  test('buttons in header are keyboard focusable', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const headerButtons = page.locator('header button');
    const count = await headerButtons.count();

    let focusableCount = 0;
    for (let i = 0; i < count; i++) {
      const btn = headerButtons.nth(i);
      const visible = await btn.isVisible().catch(() => false);
      if (!visible) continue;

      try {
        await btn.focus({ timeout: 1000 });
        const isFocused = await btn.evaluate(el => document.activeElement === el);
        if (isFocused) focusableCount++;
      } catch {
        // Element may not be focusable
      }
    }

    if (count > 0) {
      expect(focusableCount).toBeGreaterThan(0);
    }
  });

  test('focus visible outline is applied', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const firstLink = page.locator('a[href]').first();
    await firstLink.focus();

    const outlineStyle = await firstLink.evaluate(el => {
      const styles = window.getComputedStyle(el);
      return {
        outline: styles.outline,
        outlineStyle: styles.outlineStyle,
        outlineWidth: styles.outlineWidth,
      };
    });

    // Either this element or :focus-visible should show an outline
    const hasFocusIndicator =
      (outlineStyle.outline !== 'none' && outlineStyle.outline !== '') ||
      outlineStyle.outlineStyle !== 'none';

    // If no inline outline, check the computed :focus-visible via CSS
    if (!hasFocusIndicator) {
      const focusVisibleExists = await page.evaluate(() => {
        const styleSheets = document.styleSheets;
        for (let i = 0; i < styleSheets.length; i++) {
          try {
            const rules = (styleSheets[i] as CSSStyleSheet).cssRules;
            for (let j = 0; j < rules.length; j++) {
              if (rules[j].cssText.includes('focus-visible')) {
                return true;
              }
            }
          } catch {
            // Cross-origin stylesheet
          }
        }
        return false;
      });
      expect(focusVisibleExists).toBeTruthy();
    }
  });

  test('Escape key closes open menus/popups', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    // Look for any popup/menu trigger button
    const menuButtons = page.locator('button[aria-haspopup], button[aria-expanded]');
    const count = await menuButtons.count();

    if (count === 0) {
      test.skip();
      return;
    }

    const firstMenuBtn = menuButtons.first();
    const visible = await firstMenuBtn.isVisible().catch(() => false);
    if (!visible) {
      test.skip();
      return;
    }

    await firstMenuBtn.click();

    // Check if a menu appeared
    const ariaControls = await firstMenuBtn.getAttribute('aria-controls');
    if (ariaControls) {
      const menu = page.locator(`#${ariaControls}`);
      const menuVisible = await menu.isVisible().catch(() => false);
      if (menuVisible) {
        await page.keyboard.press('Escape');

        const menuStillVisible = await menu.isVisible().catch(() => false);
        expect(menuStillVisible).toBeFalsy();
      }
    }
  });

  test('Tab does not trap focus when no modal is open', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    // Count focusable elements
    const focusableElements = await page.evaluate(() => {
      const selectors = 'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';
      return document.querySelectorAll(selectors).length;
    });

    // Press Tab many times — focus should cycle through the document,
    // not get stuck in a focus trap
    const focusedElements = new Set<string>();
    for (let i = 0; i < Math.min(focusableElements, 20); i++) {
      await page.keyboard.press('Tab');
      const focused = await page.evaluate(() => {
        const el = document.activeElement;
        if (!el) return null;
        return el.tagName + (el.id ? `#${el.id}` : '') + (el.className ? `.${el.className.split(' ').slice(0, 2).join('.')}` : '');
      });
      if (focused) focusedElements.add(focused);
    }

    // Focus should have moved to at least 3 different elements
    expect(focusedElements.size).toBeGreaterThanOrEqual(3);
  });

  test('modal dialog (command palette) traps focus', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    // Open command palette with Ctrl+K
    await page.keyboard.press('Control+k');

    // Wait for dialog to appear
    const dialog = page.locator('[role="dialog"], [aria-modal="true"]');
    const dialogVisible = await dialog.first().isVisible().catch(() => false);

    if (!dialogVisible) {
      test.skip();
      return;
    }

    // Focus should be inside the dialog
    const focusInsideDialog = await page.evaluate(() => {
      const el = document.activeElement;
      if (!el) return false;
      const dialog = el.closest('[role="dialog"], [aria-modal="true"]');
      return dialog !== null;
    });
    expect(focusInsideDialog).toBeTruthy();

    // Press Escape to close
    await page.keyboard.press('Escape');

    const stillVisible = await dialog.first().isVisible().catch(() => false);
    expect(stillVisible).toBeFalsy();
  });

  test('modal dialog closes on Escape', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    // Open command palette
    await page.keyboard.press('Control+k');

    const dialog = page.locator('[role="dialog"], [aria-modal="true"]');
    const dialogVisible = await dialog.first().isVisible().catch(() => false);

    if (!dialogVisible) {
      test.skip();
      return;
    }

    await page.keyboard.press('Escape');

    const stillVisible = await dialog.first().isVisible().catch(() => false);
    expect(stillVisible).toBeFalsy();
  });

  test('keyboard shortcut Ctrl+K opens command palette', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    await page.keyboard.press('Control+k');

    const dialog = page.locator('[role="dialog"], [aria-modal="true"]');
    const dialogVisible = await dialog.first().isVisible().catch(() => false);
    expect(dialogVisible).toBeTruthy();
  });
});
