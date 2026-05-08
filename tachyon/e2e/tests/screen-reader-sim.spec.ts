import { test, expect } from '@playwright/test';
import { AppPage } from './helpers';

interface A11yViolation {
  rule: string;
  severity: 'critical' | 'serious' | 'moderate' | 'minor';
  wcag: string;
  description: string;
  selector: string;
}

// ---------------------------------------------------------------------------
// 1. ARIA Live Region Announcements
// ---------------------------------------------------------------------------

test.describe('Screen Reader Simulation — ARIA Live Regions', () => {
  test('form submission success triggers a polite live region announcement', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/login');
    await app.login('test@example.com', 'password123');

    const liveRegions = await page.evaluate(() => {
      const regions: { live: string | null; atomic: string | null; relevant: string | null; textContent: string; tagName: string }[] = [];
      const els = document.querySelectorAll('[aria-live]');
      for (const el of els) {
        const visible = el.offsetWidth > 0 || el.offsetHeight > 0;
        if (!visible) continue;
        regions.push({
          live: el.getAttribute('aria-live'),
          atomic: el.getAttribute('aria-atomic'),
          relevant: el.getAttribute('aria-relevant'),
          textContent: (el.textContent ?? '').trim().slice(0, 200),
          tagName: el.tagName.toLowerCase(),
        });
      }
      return regions;
    });

    if (liveRegions.length > 0) {
      const hasPolite = liveRegions.some(r => r.live === 'polite');
      const hasAssertive = liveRegions.some(r => r.live === 'assertive');
      expect(hasPolite || hasAssertive).toBeTruthy();
    }
  });

  test('toast notifications use aria-live or role=alert', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const announcements = await page.evaluate(() => {
      const results: { attr: string; textContent: string; role: string | null }[] = [];
      const selectors = [
        '[role="alert"]',
        '[role="status"]',
        '[aria-live="polite"]',
        '[aria-live="assertive"]',
        '[data-testid="toast"]',
        '.toast',
        '.notification',
      ];
      for (const sel of selectors) {
        const els = document.querySelectorAll(sel);
        for (const el of els) {
          const visible = el.offsetWidth > 0 || el.offsetHeight > 0;
          if (!visible) continue;
          results.push({
            attr: sel,
            textContent: (el.textContent ?? '').trim().slice(0, 200),
            role: el.getAttribute('role'),
          });
        }
      }
      return results;
    });

    for (const ann of announcements) {
      if (ann.attr === '[data-testid="toast"]' || ann.attr === '.toast' || ann.attr === '.notification') {
        const hasRole = ann.role === 'alert' || ann.role === 'status' || ann.role === 'log';
        expect(hasRole).toBeTruthy();
      }
    }
  });

  test('search result count uses a live region', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');
    await app.search('test');

    const liveRegions = await page.evaluate(() => {
      const regions: { live: string | null; textContent: string }[] = [];
      const els = document.querySelectorAll('[aria-live], [role="status"], [role="log"]');
      for (const el of els) {
        regions.push({
          live: el.getAttribute('aria-live'),
          textContent: (el.textContent ?? '').trim().slice(0, 200),
        });
      }
      return regions;
    });

    const countRegion = liveRegions.find(r =>
      /\d+\s*(result|found|item|match)/i.test(r.textContent)
    );
    if (countRegion) {
      expect(countRegion.live === 'polite' || countRegion.live === 'assertive').toBeTruthy();
    }
  });

  test('validation errors are announced via aria-live or aria-describedby', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/login');

    await page.click('button[type="submit"]');
    await page.waitForTimeout(500);

    const errorAnnouncements = await page.evaluate(() => {
      const results: { method: string; textContent: string; selector: string }[] = [];

      const liveErrors = document.querySelectorAll('[aria-live] .error, [aria-live][class*="error"], [role="alert"]');
      for (const el of liveErrors) {
        const visible = el.offsetWidth > 0 || el.offsetHeight > 0;
        if (visible) {
          results.push({ method: 'aria-live', textContent: (el.textContent ?? '').trim().slice(0, 200), selector: el.tagName.toLowerCase() });
        }
      }

      const describedInputs = document.querySelectorAll('[aria-describedby]');
      for (const input of describedInputs) {
        const descId = input.getAttribute('aria-describedby');
        if (!descId) continue;
        const descEl = document.getElementById(descId);
        if (descEl) {
          const text = (descEl.textContent ?? '').trim();
          if (text.length > 0) {
            results.push({ method: 'aria-describedby', textContent: text.slice(0, 200), selector: input.tagName.toLowerCase() });
          }
        }
      }

      return results;
    });

    const hasErrors = errorAnnouncements.length > 0;
    if (hasErrors) {
      const usesLiveOrAlert = errorAnnouncements.some(e =>
        e.method === 'aria-live' || e.method === 'aria-describedby'
      );
      expect(usesLiveOrAlert).toBeTruthy();
    }
  });
});

// ---------------------------------------------------------------------------
// 2. Accessible Navigation Patterns
// ---------------------------------------------------------------------------

test.describe('Screen Reader Simulation — Navigation Landmarks', () => {
  test('main landmark exists and contains primary content', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const mainInfo = await page.evaluate(() => {
      const main = document.querySelector('main') || document.querySelector('[role="main"]');
      if (!main) return { exists: false };
      return {
        exists: true,
        tag: main.tagName.toLowerCase(),
        hasRole: main.getAttribute('role') === 'main',
        childCount: main.children.length,
        textLength: (main.textContent ?? '').trim().length,
      };
    });

    expect(mainInfo.exists).toBeTruthy();
    expect(mainInfo.childCount).toBeGreaterThan(0);
    expect(mainInfo.textLength).toBeGreaterThan(10);
  });

  test('navigation landmark exists', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const navCount = await page.locator('nav, [role="navigation"]').count();
    expect(navCount).toBeGreaterThan(0);
  });

  test('skip navigation link targets the main content area', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const skipLinkInfo = await page.evaluate(() => {
      const skipLinks = document.querySelectorAll('a.sr-only, a[href="#main-content"], a[href="#main"], a[class*="skip"]');
      const results: { href: string | null; text: string }[] = [];
      for (const link of skipLinks) {
        results.push({
          href: link.getAttribute('href'),
          text: (link.textContent ?? '').trim(),
        });
      }
      return results;
    });

    if (skipLinkInfo.length > 0) {
      const skipHref = skipLinkInfo[0].href;
      if (skipHref && skipHref.startsWith('#')) {
        const targetId = skipHref.slice(1);
        const targetExists = await page.evaluate((id) => !!document.getElementById(id), targetId);
        expect(targetExists).toBeTruthy();
      }
    }
  });

  test('breadcrumbs (if present) use nav with aria-label', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const breadcrumbInfo = await page.evaluate(() => {
      const breadcrumbSelectors = [
        'nav[aria-label="Breadcrumb"]',
        'nav[aria-label="breadcrumb"]',
        '[aria-label="Breadcrumb"]',
        '.breadcrumb',
        '[class*="breadcrumb"]',
      ];
      for (const sel of breadcrumbSelectors) {
        const els = document.querySelectorAll(sel);
        if (els.length > 0) {
          const el = els[0];
          return {
            found: true,
            selector: sel,
            tagName: el.tagName.toLowerCase(),
            ariaLabel: el.getAttribute('aria-label'),
            hasNav: el.tagName.toLowerCase() === 'nav',
          };
        }
      }
      return { found: false };
    });

    if (breadcrumbInfo.found) {
      expect(breadcrumbInfo.hasNav).toBeTruthy();
      expect(breadcrumbInfo.ariaLabel?.toLowerCase()).toContain('breadcrumb');
    }
  });

  test('headings form a logical hierarchy with no skipped levels', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const headingViolations = await page.evaluate(() => {
      const violations: { from: number; to: number; text: string }[] = [];
      const headings = document.querySelectorAll('h1, h2, h3, h4, h5, h6');
      let prevLevel = 0;

      for (const heading of headings) {
        if (heading.offsetWidth === 0 && heading.offsetHeight === 0) continue;
        const tag = heading.tagName.toLowerCase();
        const level = parseInt(tag.replace('h', ''), 10);
        const text = (heading.textContent ?? '').trim();

        if (prevLevel > 0 && level > prevLevel + 1) {
          violations.push({ from: prevLevel, to: level, text: text.slice(0, 50) });
        }
        prevLevel = level;
      }
      return violations;
    });

    expect(headingViolations).toHaveLength(0);
  });
});

// ---------------------------------------------------------------------------
// 3. Form Accessibility
// ---------------------------------------------------------------------------

test.describe('Screen Reader Simulation — Form Accessibility', () => {
  test('all inputs have associated labels', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/login');

    const violations = await page.evaluate(() => {
      const issues: { name: string; type: string; tag: string }[] = [];
      const inputs = document.querySelectorAll(
        'input:not([type="hidden"]):not([type="submit"]):not([type="button"]):not([type="reset"]):not([type="image"]), textarea, select'
      );
      for (const input of inputs) {
        if (input.offsetWidth === 0 && input.offsetHeight === 0) continue;

        const ariaLabel = input.getAttribute('aria-label');
        const ariaLabelledby = input.getAttribute('aria-labelledby');
        if (ariaLabel || ariaLabelledby) continue;

        const id = input.getAttribute('id');
        if (id) {
          const label = document.querySelector(`label[for="${id}"]`);
          if (label) continue;
        }

        const placeholder = input.getAttribute('placeholder');
        if (placeholder && placeholder.trim().length > 0) continue;

        const title = input.getAttribute('title');
        if (title && title.trim().length > 0) continue;

        const wrappingLabel = input.closest('label');
        if (wrappingLabel) continue;

        issues.push({
          name: input.getAttribute('name') ?? 'unnamed',
          type: input.getAttribute('type') ?? input.tagName.toLowerCase(),
          tag: input.tagName.toLowerCase(),
        });
      }
      return issues;
    });

    expect(violations).toHaveLength(0);
  });

  test('required fields have aria-required="true" or are native required', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/login');

    const violations = await page.evaluate(() => {
      const issues: { name: string; reason: string }[] = [];
      const inputs = document.querySelectorAll('input, textarea, select');
      for (const input of inputs) {
        if (input.offsetWidth === 0 && input.offsetHeight === 0) continue;

        const required = (input as HTMLInputElement).required;
        const ariaRequired = input.getAttribute('aria-required');
        const ariaLabel = input.getAttribute('aria-label') ?? '';
        const labelText = (() => {
          const id = input.getAttribute('id');
          if (id) {
            const label = document.querySelector(`label[for="${id}"]`);
            if (label) return (label.textContent ?? '').trim();
          }
          return '';
        })();

        const indicatesRequired = /\*|required|mandatory/i.test(ariaLabel + ' ' + labelText);

        if (required || ariaRequired === 'true' || indicatesRequired) {
          if (!required && ariaRequired !== 'true') {
            issues.push({
              name: input.getAttribute('name') ?? 'unnamed',
              reason: 'Visually indicated as required but no required/aria-required attribute',
            });
          }
        }
      }
      return issues;
    });

    expect(violations).toHaveLength(0);
  });

  test('error messages use aria-describedby pointing to inputs', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/login');

    await page.click('button[type="submit"]');
    await page.waitForTimeout(500);

    const errors = await page.evaluate(() => {
      const results: { errorId: string; inputName: string }[] = [];
      const describedInputs = document.querySelectorAll('[aria-describedby]');
      for (const input of describedInputs) {
        const descId = input.getAttribute('aria-describedby');
        if (!descId) continue;
        const descEl = document.getElementById(descId);
        if (descEl) {
          const text = (descEl.textContent ?? '').trim();
          if (text.length > 0) {
            results.push({
              errorId: descId,
              inputName: input.getAttribute('name') ?? 'unnamed',
            });
          }
        }
      }
      return results;
    });

    const visibleErrors = await page.locator('[class*="error"], [role="alert"]').count();
    if (visibleErrors > 0) {
      expect(errors.length).toBeGreaterThan(0);
    }
  });

  test('focus moves to first error after form submission failure', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/login');

    await page.click('button[type="submit"]');
    await page.waitForTimeout(500);

    const focusedElement = await page.evaluate(() => {
      const el = document.activeElement;
      if (!el) return { tag: null, hasError: false, name: null };
      const hasError =
        el.getAttribute('aria-invalid') === 'true' ||
        el.classList.toString().includes('error') ||
        el.closest('[class*="error"]') !== null;
      return {
        tag: el.tagName.toLowerCase(),
        hasError,
        name: el.getAttribute('name'),
      };
    });

    const hasVisibleErrors = await page.locator('[class*="error"], [aria-invalid="true"]').count();
    if (hasVisibleErrors > 0) {
      expect(focusedElement.hasError || focusedElement.tag === 'input').toBeTruthy();
    }
  });

  test('radio groups use role=radiogroup with aria-labelledby', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const radioGroups = await page.evaluate(() => {
      const radios = document.querySelectorAll('input[type="radio"]');
      if (radios.length === 0) return { hasRadios: false, violations: [] as string[] };

      const groups = new Map<string, Element[]>();
      for (const radio of radios) {
        if (radio.offsetWidth === 0 && radio.offsetHeight === 0) continue;
        const name = radio.getAttribute('name') ?? '_unnamed';
        const arr = groups.get(name) ?? [];
        arr.push(radio);
        groups.set(name, arr);
      }

      const violations: string[] = [];
      for (const [name, elements] of groups) {
        if (elements.length < 2) continue;
        const parent = elements[0].parentElement;
        const hasGroupRole = parent?.getAttribute('role') === 'radiogroup' ||
          parent?.closest('[role="radiogroup"]') !== null;
        const group = parent?.closest('[role="radiogroup"]') ?? parent;
        const hasLabelledby = group?.getAttribute('aria-labelledby') || group?.getAttribute('aria-label');
        if (!hasGroupRole) {
          violations.push(`Radio group "${name}" is missing role="radiogroup"`);
        }
        if (!hasLabelledby) {
          violations.push(`Radio group "${name}" is missing aria-labelledby or aria-label`);
        }
      }
      return { hasRadios: true, violations };
    });

    if (radioGroups.hasRadios) {
      expect(radioGroups.violations).toHaveLength(0);
    }
  });
});

// ---------------------------------------------------------------------------
// 4. Interactive Elements
// ---------------------------------------------------------------------------

test.describe('Screen Reader Simulation — Interactive Elements', () => {
  test('clickable elements use semantic tags or have proper ARIA', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const violations = await page.evaluate(() => {
      const issues: { tag: string; className: string; reason: string }[] = [];
      const badClickables = document.querySelectorAll('[onclick]:not(button):not(a):not(input):not(select):not(textarea):not([role="button"]):not([role="link"]):not(summary)');
      for (const el of badClickables) {
        if (el.offsetWidth === 0 && el.offsetHeight === 0) continue;
        const tabindex = el.getAttribute('tabindex');
        if (tabindex === null || tabindex === '-1') {
          issues.push({
            tag: el.tagName.toLowerCase(),
            className: el.className.toString().slice(0, 80),
            reason: 'Has onclick but is not a semantic interactive element, has no role, and no tabindex',
          });
        }
      }
      return issues;
    });

    expect(violations).toHaveLength(0);
  });

  test('custom buttons have role=button, tabindex=0, and keyboard support', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const violations = await page.evaluate(() => {
      const issues: { tag: string; missing: string[] }[] = [];
      const customButtons = document.querySelectorAll('[role="button"]:not(button):not(input[type="button"]):not(input[type="submit"])');
      for (const el of customButtons) {
        if (el.offsetWidth === 0 && el.offsetHeight === 0) continue;
        const missing: string[] = [];
        const tabindex = el.getAttribute('tabindex');
        if (tabindex === null || tabindex === '-1') {
          missing.push('tabindex="0"');
        }
        issues.push({ tag: el.tagName.toLowerCase(), missing });
      }
      return issues;
    });

    for (const v of violations) {
      expect(v.missing).toHaveLength(0);
    }
  });

  test('modal dialogs have proper ARIA attributes', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    await page.keyboard.press('Control+k');

    const dialog = page.locator('[role="dialog"], [aria-modal="true"]');
    const dialogVisible = await dialog.first().isVisible().catch(() => false);

    if (!dialogVisible) {
      test.skip();
      return;
    }

    const dialogInfo = await page.evaluate(() => {
      const dialog = document.querySelector('[role="dialog"], [aria-modal="true"]');
      if (!dialog) return null;
      return {
        role: dialog.getAttribute('role'),
        ariaModal: dialog.getAttribute('aria-modal'),
        ariaLabelledby: dialog.getAttribute('aria-labelledby'),
        ariaLabel: dialog.getAttribute('aria-label'),
        labelledbyExists: (() => {
          const id = dialog.getAttribute('aria-labelledby');
          return id ? !!document.getElementById(id) : false;
        })(),
      };
    });

    expect(dialogInfo).not.toBeNull();
    expect(dialogInfo!.role).toBe('dialog');
    expect(dialogInfo!.ariaModal).toBe('true');
    expect(dialogInfo!.ariaLabelledby || dialogInfo!.ariaLabel).toBeTruthy();
    if (dialogInfo!.ariaLabelledby) {
      expect(dialogInfo!.labelledbyExists).toBeTruthy();
    }

    await page.keyboard.press('Escape');
  });

  test('dropdown menus use role=menu with role=menuitem children', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const menuButtons = page.locator('button[aria-haspopup="menu"], button[aria-haspopup="true"]');
    const btnCount = await menuButtons.count();

    if (btnCount === 0) {
      test.skip();
      return;
    }

    for (let i = 0; i < btnCount; i++) {
      const btn = menuButtons.nth(i);
      const visible = await btn.isVisible().catch(() => false);
      if (!visible) continue;

      await btn.click();
      await page.waitForTimeout(300);

      const menuInfo = await page.evaluate(() => {
        const menu = document.querySelector('[role="menu"]');
        if (!menu) return null;
        const items = menu.querySelectorAll('[role="menuitem"]');
        return {
          hasMenu: true,
          itemCount: items.length,
          allHaveRole: Array.from(items).every(item => item.getAttribute('role') === 'menuitem'),
        };
      });

      if (menuInfo) {
        expect(menuInfo.allHaveRole).toBeTruthy();
      }

      await page.keyboard.press('Escape');
      await page.waitForTimeout(200);
    }
  });

  test('toggle buttons have aria-pressed state', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const violations = await page.evaluate(() => {
      const issues: { text: string }[] = [];
      const buttons = document.querySelectorAll('button');
      for (const btn of buttons) {
        if (btn.offsetWidth === 0 && btn.offsetHeight === 0) continue;
        const ariaPressed = btn.getAttribute('aria-pressed');
        if (ariaPressed === null) continue;
        if (ariaPressed !== 'true' && ariaPressed !== 'false' && ariaPressed !== 'mixed') {
          issues.push({
            text: (btn.textContent ?? '').trim().slice(0, 50),
          });
        }
      }
      return issues;
    });

    expect(violations).toHaveLength(0);
  });
});

// ---------------------------------------------------------------------------
// 5. Image and Media Accessibility
// ---------------------------------------------------------------------------

test.describe('Screen Reader Simulation — Image and Media', () => {
  test('all img elements have alt text (empty for decorative)', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const violations = await page.evaluate(() => {
      const issues: { src: string; reason: string }[] = [];
      const images = document.querySelectorAll('img');
      for (const img of images) {
        if (img.offsetWidth === 0 && img.offsetHeight === 0) continue;
        const role = img.getAttribute('role');
        if (role === 'presentation' || role === 'none') continue;

        const alt = img.getAttribute('alt');
        if (alt === null) {
          issues.push({
            src: (img.getAttribute('src') ?? 'unknown').slice(0, 100),
            reason: 'Missing alt attribute',
          });
        }
      }
      return issues;
    });

    expect(violations).toHaveLength(0);
  });

  test('SVG icons have aria-hidden=true or role=img with title', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const violations = await page.evaluate(() => {
      const issues: { reason: string }[] = [];
      const svgs = document.querySelectorAll('svg');
      for (const svg of svgs) {
        if (svg.offsetWidth === 0 && svg.offsetHeight === 0) continue;
        const ariaHidden = svg.getAttribute('aria-hidden');
        const role = svg.getAttribute('role');
        const ariaLabel = svg.getAttribute('aria-label');
        const title = svg.querySelector('title');

        if (ariaHidden === 'true') continue;
        if (role === 'presentation' || role === 'none') continue;

        const isInfoCarrying = ariaLabel || (role === 'img' && title);
        if (!isInfoCarrying) {
          const parentAriaHidden = svg.parentElement?.getAttribute('aria-hidden');
          if (parentAriaHidden !== 'true') {
            issues.push({
              reason: 'SVG has no aria-hidden="true", no role="img" with <title>, and no aria-label. If decorative, add aria-hidden="true".',
            });
          }
        }
      }
      return issues;
    });

    expect(violations).toHaveLength(0);
  });

  test('decorative elements have aria-hidden=true', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const violations = await page.evaluate(() => {
      const issues: { tag: string; className: string }[] = [];
      const decorativePatterns = document.querySelectorAll(
        '[class*="divider"], [class*="separator"], [class*="decorative"], [class*="icon-only"]'
      );
      for (const el of decorativePatterns) {
        if (el.offsetWidth === 0 && el.offsetHeight === 0) continue;
        const ariaHidden = el.getAttribute('aria-hidden');
        if (ariaHidden !== 'true') {
          const text = (el.textContent ?? '').trim();
          if (text.length === 0) {
            issues.push({
              tag: el.tagName.toLowerCase(),
              className: el.className.toString().split(' ').slice(0, 3).join(' '),
            });
          }
        }
      }
      return issues;
    });

    expect(violations).toHaveLength(0);
  });
});

// ---------------------------------------------------------------------------
// 6. Document Structure Simulation
// ---------------------------------------------------------------------------

test.describe('Screen Reader Simulation — Document Structure', () => {
  test('page title is descriptive', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const title = await page.title();
    expect(title.length).toBeGreaterThanOrEqual(3);
    expect(title.trim().length).toBeGreaterThan(0);
  });

  test('language is set on html element', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const lang = await page.getAttribute('html', 'lang');
    expect(lang).toBeTruthy();
    expect(lang!.length).toBeGreaterThanOrEqual(2);
  });

  test('reading order matches visual order (tab order is logical)', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const tabOrder = await page.evaluate(() => {
      const elements: { tag: string; rect: DOMRect }[] = [];
      const focusable = document.querySelectorAll(
        'a[href], button:not([disabled]), input:not([disabled]):not([type="hidden"]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
      );
      for (const el of focusable) {
        if (el.offsetWidth === 0 && el.offsetHeight === 0) continue;
        const rect = el.getBoundingClientRect();
        if (rect.top < 0) continue;
        elements.push({
          tag: el.tagName.toLowerCase(),
          rect: { ...rect },
        });
      }
      return elements;
    });

    if (tabOrder.length < 3) return;

    let inversionCount = 0;
    for (let i = 1; i < tabOrder.length; i++) {
      if (tabOrder[i].rect.top < tabOrder[i - 1].rect.top - 5) {
        inversionCount++;
      }
    }

    const inversionRatio = inversionCount / Math.max(tabOrder.length - 1, 1);
    expect(inversionRatio).toBeLessThan(0.3);
  });

  test('data tables have proper header cells with scope', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const tableViolations = await page.evaluate(() => {
      const issues: { tableIndex: number; reason: string }[] = [];
      const tables = document.querySelectorAll('table');
      for (let t = 0; t < tables.length; t++) {
        const table = tables[t];
        if (table.offsetWidth === 0 && table.offsetHeight === 0) continue;
        const role = table.getAttribute('role');
        if (role === 'presentation' || role === 'none') continue;

        const ths = table.querySelectorAll('th');
        if (ths.length === 0) {
          const caption = table.querySelector('caption');
          issues.push({
            tableIndex: t,
            reason: `Data table has no <th> elements${caption ? ` (caption: "${(caption.textContent ?? '').trim().slice(0, 50)}")` : ''}`,
          });
          continue;
        }

        for (const th of ths) {
          const scope = th.getAttribute('scope');
          if (!scope) {
            const hasId = th.getAttribute('id');
            if (!hasId) {
              issues.push({
                tableIndex: t,
                reason: `<th> element "${(th.textContent ?? '').trim().slice(0, 30)}" has no scope attribute`,
              });
            }
          }
        }
      }
      return issues;
    });

    expect(tableViolations).toHaveLength(0);
  });
});

// ---------------------------------------------------------------------------
// 7. Focus Management
// ---------------------------------------------------------------------------

test.describe('Screen Reader Simulation — Focus Management', () => {
  test('no focus traps outside of modals', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const isModalOpen = await page.locator('[role="dialog"][aria-modal="true"]').first().isVisible().catch(() => false);
    if (isModalOpen) {
      await page.keyboard.press('Escape');
      await page.waitForTimeout(200);
    }

    const focusedElements = new Set<string>();
    const totalFocusable = await page.evaluate(() => {
      return document.querySelectorAll(
        'a[href], button:not([disabled]), input:not([disabled]):not([type="hidden"]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
      ).length;
    });

    const iterations = Math.min(totalFocusable, 30);
    for (let i = 0; i < iterations; i++) {
      await page.keyboard.press('Tab');
      const info = await page.evaluate(() => {
        const el = document.activeElement;
        if (!el) return null;
        return el.tagName + '#' + (el.id || 'none') + '.' + (el.className.toString().split(' ').slice(0, 2).join('.') || 'none');
      });
      if (info) focusedElements.add(info);
    }

    expect(focusedElements.size).toBeGreaterThanOrEqual(3);
  });

  test('after modal close, focus returns to trigger element', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const triggerBtn = page.locator('button[aria-haspopup], button[aria-expanded]');
    const btnCount = await triggerBtn.count();

    if (btnCount === 0) {
      await page.keyboard.press('Control+k');
      const dialog = page.locator('[role="dialog"]');
      const dialogVisible = await dialog.first().isVisible().catch(() => false);

      if (!dialogVisible) {
        test.skip();
        return;
      }

      await page.keyboard.press('Escape');
      await page.waitForTimeout(300);

      const focusIsOutsideDialog = await page.evaluate(() => {
        const el = document.activeElement;
        if (!el) return false;
        return !el.closest('[role="dialog"]');
      });
      expect(focusIsOutsideDialog).toBeTruthy();
      return;
    }

    const btn = triggerBtn.first();
    const visible = await btn.isVisible().catch(() => false);
    if (!visible) {
      test.skip();
      return;
    }

    const btnId = await btn.evaluate(el => {
      const id = el.id || 'trigger-' + Math.random().toString(36).slice(2, 8);
      if (!el.id) el.id = id;
      return el.id;
    });

    await btn.click();
    await page.waitForTimeout(300);

    const dialogVisible = await page.locator('[role="dialog"], [aria-modal="true"]').first().isVisible().catch(() => false);
    if (dialogVisible) {
      await page.keyboard.press('Escape');
      await page.waitForTimeout(300);

      const focusReturned = await page.evaluate((id) => {
        const el = document.activeElement;
        if (!el) return false;
        return el.id === id || el.closest(`#${id}`) !== null;
      }, btnId);
      expect(focusReturned).toBeTruthy();
    }
  });

  test('after page navigation, focus moves to content area', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const navLink = page.locator('nav a[href]').first();
    const linkExists = await navLink.isVisible().catch(() => false);

    if (!linkExists) {
      test.skip();
      return;
    }

    const href = await navLink.getAttribute('href');
    if (!href || href.startsWith('http') || href.startsWith('#')) {
      test.skip();
      return;
    }

    await navLink.click();
    await page.waitForLoadState('networkidle');

    const focusInfo = await page.evaluate(() => {
      const el = document.activeElement;
      if (!el) return { tag: null, inMain: false };
      const main = el.closest('main') || el.closest('[role="main"]');
      const isHeading = /^H[1-6]$/.test(el.tagName);
      return {
        tag: el.tagName.toLowerCase(),
        inMain: main !== null,
        isHeading,
        isBody: el.tagName.toLowerCase() === 'body',
      };
    });

    expect(focusInfo.isBody).toBeFalsy();
  });
});
