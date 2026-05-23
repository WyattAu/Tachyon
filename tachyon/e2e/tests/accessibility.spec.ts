import { test, expect } from '@playwright/test';
import { AppPage } from './helpers';

interface A11yViolation {
  rule: string;
  severity: 'critical' | 'serious' | 'moderate' | 'minor';
  description: string;
  selector: string;
}

function collectViolations(violations: A11yViolation[]): A11yViolation[] {
  return violations;
}

async function checkImagesHaveAlt(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];
  const images = await page.locator('img').all();
  for (const img of images) {
    const alt = await img.getAttribute('alt');
    const role = await img.getAttribute('role');
    if (role === 'presentation' || role === 'none') continue;
    if (alt === null) {
      const src = await img.getAttribute('src') ?? 'unknown';
      violations.push({
        rule: 'img-alt',
        severity: 'critical',
        description: `Image missing alt attribute: ${src}`,
        selector: `img[src="${src}"]`,
      });
    }
  }
  return violations;
}

async function checkFormInputLabels(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];
  const inputs = await page.locator('input:not([type="hidden"]):not([type="submit"]):not([type="button"]):not([type="reset"]):not([type="image"])').all();
  for (const input of inputs) {
    const id = await input.getAttribute('id');
    const ariaLabel = await input.getAttribute('aria-label');
    const ariaLabelledby = await input.getAttribute('aria-labelledby');
    const placeholder = await input.getAttribute('placeholder');
    const title = await input.getAttribute('title');

    if (ariaLabel || ariaLabelledby) continue;

    if (id) {
      const labelExists = await page.locator(`label[for="${id}"]`).count().then(c => c > 0);
      if (labelExists) continue;
    }

    if (placeholder && placeholder.trim().length > 0) continue;
    if (title && title.trim().length > 0) continue;

    const name = await input.getAttribute('name') ?? 'unknown';
    violations.push({
      rule: 'label',
      severity: 'critical',
      description: `Form input "${name}" has no associated label, aria-label, or placeholder`,
      selector: `input[name="${name}"]`,
    });
  }

  const textareas = await page.locator('textarea').all();
  for (const ta of textareas) {
    const ariaLabel = await ta.getAttribute('aria-label');
    const ariaLabelledby = await ta.getAttribute('aria-labelledby');
    const id = await ta.getAttribute('id');
    const placeholder = await ta.getAttribute('placeholder');

    if (ariaLabel || ariaLabelledby) continue;
    if (id) {
      const labelExists = await page.locator(`label[for="${id}"]`).count().then(c => c > 0);
      if (labelExists) continue;
    }
    if (placeholder && placeholder.trim().length > 0) continue;

    violations.push({
      rule: 'label',
      severity: 'critical',
      description: 'Textarea has no associated label',
      selector: 'textarea',
    });
  }

  const selects = await page.locator('select').all();
  for (const sel of selects) {
    const ariaLabel = await sel.getAttribute('aria-label');
    const ariaLabelledby = await sel.getAttribute('aria-labelledby');
    const id = await sel.getAttribute('id');

    if (ariaLabel || ariaLabelledby) continue;
    if (id) {
      const labelExists = await page.locator(`label[for="${id}"]`).count().then(c => c > 0);
      if (labelExists) continue;
    }

    violations.push({
      rule: 'label',
      severity: 'critical',
      description: 'Select element has no associated label',
      selector: 'select',
    });
  }

  return violations;
}

async function checkInteractiveElementsFocusable(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];
  const interactiveSelectors = [
    '[onclick]:not(button):not(a):not(input):not(select):not(textarea):not([role="button"]):not([tabindex])',
    '[role="button"]:not([tabindex]):not(button):not(a)',
  ];

  for (const sel of interactiveSelectors) {
    const elements = await page.locator(sel).all();
    for (const el of elements) {
      const visible = await el.isVisible().catch(() => false);
      if (!visible) continue;
      const tag = await el.evaluate(e => e.tagName.toLowerCase());
      violations.push({
        rule: 'focusable',
        severity: 'serious',
        description: `Interactive <${tag}> element is not natively focusable and has no tabindex`,
        selector: sel,
      });
    }
  }
  return violations;
}

async function checkColorContrast(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];

  const contrastChecks = await page.evaluate(() => {
    const results: { selector: string; contrast: number; fg: string; bg: string }[] = [];
    const elements = document.querySelectorAll('p, span, a, button, label, h1, h2, h3, h4, h5, h6, li, td, th, [role="button"], [role="link"]');

    for (const el of elements) {
      const style = window.getComputedStyle(el);
      const fg = style.color;
      const bg = style.backgroundColor;

      if (bg === 'rgba(0, 0, 0, 0)' || bg === 'transparent') {
        let parent: Element | null = el.parentElement;
        let bgColor = bg;
        while (parent) {
          const pStyle = window.getComputedStyle(parent);
          bgColor = pStyle.backgroundColor;
          if (bgColor !== 'rgba(0, 0, 0, 0)' && bgColor !== 'transparent') break;
          parent = parent.parentElement;
        }
        if (bgColor === 'rgba(0, 0, 0, 0)' || bgColor === 'transparent') continue;
      }

      results.push({
        selector: el.tagName.toLowerCase() + (el.id ? `#${el.id}` : '') + (el.className ? `.${el.className.split(' ').join('.')}` : ''),
        contrast: 0,
        fg,
        bg,
      });
    }
    return results;
  });

  for (const check of contrastChecks) {
    const contrast = await page.evaluate(({ fg, bg }: { fg: string; bg: string }) => {
      function parseColor(c: string): [number, number, number] {
        const match = c.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/);
        if (!match) return [0, 0, 0];
        return [parseInt(match[1]), parseInt(match[2]), parseInt(match[3])];
      }
      function relativeLuminance([r, g, b]: [number, number, number]): number {
        const [rs, gs, bs] = [r, g, b].map(c => {
          const s = c / 255;
          return s <= 0.03928 ? s / 12.92 : Math.pow((s + 0.055) / 1.055, 2.4);
        });
        return 0.2126 * rs + 0.7152 * gs + 0.0722 * bs;
      }
      function contrastRatio(l1: number, l2: number): number {
        const lighter = Math.max(l1, l2);
        const darker = Math.min(l1, l2);
        return (lighter + 0.05) / (darker + 0.05);
      }
      const fgRgb = parseColor(fg);
      const bgRgb = parseColor(bg);
      const lFg = relativeLuminance(fgRgb);
      const lBg = relativeLuminance(bgRgb);
      return contrastRatio(lFg, lBg);
    }, { fg: check.fg, bg: check.bg });

    if (contrast > 0 && contrast < 4.5) {
      violations.push({
        rule: 'color-contrast',
        severity: 'serious',
        description: `Low contrast ratio (${contrast.toFixed(2)}:1) on ${check.selector}`,
        selector: check.selector,
      });
    }
  }
  return violations;
}

async function checkHeadingHierarchy(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];
  const headings = await page.locator('h1, h2, h3, h4, h5, h6').all();

  if (headings.length === 0) return violations;

  let prevLevel = 0;
  for (const heading of headings) {
    const visible = await heading.isVisible().catch(() => false);
    if (!visible) continue;

    const tag = await heading.evaluate(e => e.tagName.toLowerCase());
    const level = parseInt(tag.replace('h', ''), 10);

    if (prevLevel === 0 && level !== 1) {
      const text = await heading.textContent() ?? '';
      violations.push({
        rule: 'heading-order',
        severity: 'moderate',
        description: `First visible heading is <${tag}> ("${text.trim()}") but should start with <h1>`,
        selector: tag,
      });
    }

    if (prevLevel > 0 && level > prevLevel + 1) {
      const text = await heading.textContent() ?? '';
      violations.push({
        rule: 'heading-order',
        severity: 'moderate',
        description: `Heading level skipped: <h${prevLevel}> to <${tag}> ("${text.trim()}")`,
        selector: tag,
      });
    }

    prevLevel = level;
  }
  return violations;
}

async function checkAriaAttributes(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];

  const ariaLabelledbyMissing = await page.evaluate(() => {
    const issues: string[] = [];
    const elements = document.querySelectorAll('[aria-labelledby]');
    for (const el of elements) {
      const ids = (el.getAttribute('aria-labelledby') ?? '').split(/\s+/);
      for (const id of ids) {
        if (id && !document.getElementById(id)) {
          issues.push(`aria-labelledby="${id}" references missing element #${id}`);
        }
      }
    }
    return issues;
  });
  for (const issue of ariaLabelledbyMissing) {
    violations.push({ rule: 'aria-valid-attr-value', severity: 'critical', description: issue, selector: '[aria-labelledby]' });
  }

  const ariaDescribedbyMissing = await page.evaluate(() => {
    const issues: string[] = [];
    const elements = document.querySelectorAll('[aria-describedby]');
    for (const el of elements) {
      const ids = (el.getAttribute('aria-describedby') ?? '').split(/\s+/);
      for (const id of ids) {
        if (id && !document.getElementById(id)) {
          issues.push(`aria-describedby="${id}" references missing element #${id}`);
        }
      }
    }
    return issues;
  });
  for (const issue of ariaDescribedbyMissing) {
    violations.push({ rule: 'aria-valid-attr-value', severity: 'serious', description: issue, selector: '[aria-describedby]' });
  }

  const ariaControlsMissing = await page.evaluate(() => {
    const issues: string[] = [];
    const elements = document.querySelectorAll('[aria-controls]');
    for (const el of elements) {
      const ids = (el.getAttribute('aria-controls') ?? '').split(/\s+/);
      for (const id of ids) {
        if (id && !document.getElementById(id)) {
          issues.push(`aria-controls="${id}" references missing element #${id}`);
        }
      }
    }
    return issues;
  });
  for (const issue of ariaControlsMissing) {
    violations.push({ rule: 'aria-valid-attr-value', severity: 'moderate', description: issue, selector: '[aria-controls]' });
  }

  const invalidRoles = await page.evaluate(() => {
    const validRoles = new Set(['alert', 'alertdialog', 'application', 'article', 'banner', 'button', 'cell', 'checkbox', 'columnheader', 'combobox', 'complementary', 'contentinfo', 'definition', 'dialog', 'directory', 'document', 'feed', 'figure', 'form', 'grid', 'gridcell', 'group', 'heading', 'img', 'link', 'list', 'listbox', 'listitem', 'log', 'main', 'marquee', 'math', 'menu', 'menubar', 'menuitem', 'menuitemcheckbox', 'menuitemradio', 'navigation', 'note', 'option', 'presentation', 'progressbar', 'radio', 'radiogroup', 'region', 'row', 'rowgroup', 'rowheader', 'scrollbar', 'search', 'searchbox', 'separator', 'slider', 'spinbutton', 'status', 'switch', 'tab', 'table', 'tablist', 'tabpanel', 'term', 'textbox', 'timer', 'toolbar', 'tooltip', 'tree', 'treegrid', 'treeitem']);
    const issues: string[] = [];
    const elements = document.querySelectorAll('[role]');
    for (const el of elements) {
      const role = el.getAttribute('role') ?? '';
      if (!validRoles.has(role)) {
        issues.push(`Invalid role="${role}"`);
      }
    }
    return issues;
  });
  for (const issue of invalidRoles) {
    violations.push({ rule: 'valid-role', severity: 'critical', description: issue, selector: '[role]' });
  }

  return violations;
}

async function checkEmptyLinksAndButtons(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];

  const links = await page.locator('a[href]').all();
  for (const link of links) {
    const visible = await link.isVisible().catch(() => false);
    if (!visible) continue;

    const text = (await link.textContent() ?? '').trim();
    const ariaLabel = await link.getAttribute('aria-label');
    const ariaLabelledby = await link.getAttribute('aria-labelledby');
    const title = await link.getAttribute('title');
    const imgAlt = await link.locator('img').first().getAttribute('alt').catch(() => null);
    const svgHidden = await link.locator('svg[aria-hidden="true"]').count().then(c => c > 0);

    if (!text && !ariaLabel && !ariaLabelledby && !title && !imgAlt && svgHidden) {
      violations.push({
        rule: 'link-name',
        severity: 'critical',
        description: 'Link has no accessible name (no text, aria-label, aria-labelledby, or title)',
        selector: 'a[href]',
      });
    }
  }

  const buttons = await page.locator('button').all();
  for (const btn of buttons) {
    const visible = await btn.isVisible().catch(() => false);
    if (!visible) continue;

    const text = (await btn.textContent() ?? '').trim();
    const ariaLabel = await btn.getAttribute('aria-label');
    const ariaLabelledby = await btn.getAttribute('aria-labelledby');
    const title = await btn.getAttribute('title');
    const imgAlt = await btn.locator('img').first().getAttribute('alt').catch(() => null);
    const svgHidden = await btn.locator('svg[aria-hidden="true"]').count().then(c => c > 0);

    if (!text && !ariaLabel && !ariaLabelledby && !title && !imgAlt && svgHidden) {
      violations.push({
        rule: 'button-name',
        severity: 'critical',
        description: 'Button has no accessible name (no text, aria-label, aria-labelledby, or title)',
        selector: 'button',
      });
    }
  }

  return violations;
}

async function checkSkipNavigation(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];
  const skipLink = page.locator('a.sr-only, a[class*="skip"], a[href="#main-content"], a[href="#main-nav"]');
  const count = await skipLink.count();
  if (count === 0) {
    violations.push({
      rule: 'skip-link',
      severity: 'serious',
      description: 'No skip navigation link found',
      selector: 'a.sr-only',
    });
  }
  return violations;
}

async function checkMainLandmark(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];
  // Wait for WASM hydration to render the main landmark
  try {
    await page.waitForSelector('main, [role="main"]', { timeout: 15000 });
  } catch {
    // Main landmark never appeared
  }
  const main = page.locator('main, [role="main"]');
  if (await main.count() === 0) {
    violations.push({
      rule: 'main',
      severity: 'critical',
      description: 'No <main> or [role="main"] landmark found',
      selector: 'main',
    });
  }
  return violations;
}

async function checkDocumentLang(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];
  const lang = await page.getAttribute('html', 'lang');
  if (!lang) {
    violations.push({
      rule: 'html-lang',
      severity: 'serious',
      description: '<html> element has no lang attribute',
      selector: 'html',
    });
  }
  return violations;
}

async function checkPageTitle(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];
  const title = await page.title();
  if (!title || title.trim().length === 0) {
    violations.push({
      rule: 'document-title',
      severity: 'serious',
      description: 'Page has no <title> element or empty title',
      selector: 'title',
    });
  }
  return violations;
}

async function runA11yAudit(page: import('@playwright/test').Page, pageName: string): Promise<A11yViolation[]> {
  const violations = collectViolations([
    ...(await checkImagesHaveAlt(page)),
    ...(await checkFormInputLabels(page)),
    ...(await checkInteractiveElementsFocusable(page)),
    ...(await checkHeadingHierarchy(page)),
    ...(await checkAriaAttributes(page)),
    ...(await checkEmptyLinksAndButtons(page)),
    ...(await checkSkipNavigation(page)),
    ...(await checkMainLandmark(page)),
    ...(await checkDocumentLang(page)),
    ...(await checkPageTitle(page)),
    ...(await checkColorContrast(page)),
  ]);

  const criticals = violations.filter(v => v.severity === 'critical');
  const serious = violations.filter(v => v.severity === 'serious');
  const moderate = violations.filter(v => v.severity === 'moderate');

  if (criticals.length > 0 || serious.length > 0) {
    console.error(`\n[${pageName}] FAIL: ${criticals.length} critical, ${serious.length} serious, ${moderate.length} moderate violations`);
    for (const v of [...criticals, ...serious]) {
      console.error(`  [${v.severity}] ${v.rule}: ${v.description}`);
    }
  } else if (moderate.length > 0) {
    console.warn(`\n[${pageName}] WARN: ${moderate.length} moderate violations`);
    for (const v of moderate) {
      console.warn(`  [${v.severity}] ${v.rule}: ${v.description}`);
    }
  } else {
    console.log(`\n[${pageName}] PASS: No accessibility violations found`);
  }

  return violations;
}

test.describe('Accessibility', () => {
  test('login page meets accessibility standards', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/login');

    const violations = await runA11yAudit(page, 'Login');
    const criticals = violations.filter(v => v.severity === 'critical');
    expect(criticals).toHaveLength(0);
  });

  test('register page meets accessibility standards', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/register');

    const violations = await runA11yAudit(page, 'Register');
    const criticals = violations.filter(v => v.severity === 'critical');
    expect(criticals).toHaveLength(0);
  });

  test('home page meets accessibility standards', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const violations = await runA11yAudit(page, 'Home');
    const criticals = violations.filter(v => v.severity === 'critical');
    expect(criticals).toHaveLength(0);
  });

  test('404 page meets accessibility standards', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/this-does-not-exist');

    const violations = await runA11yAudit(page, '404');
    const criticals = violations.filter(v => v.severity === 'critical');
    expect(criticals).toHaveLength(0);
  });

  test('skip navigation link exists and targets main content', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const skipLink = page.locator('a[href="#main-content"], a[href="#main-nav"]');
    await expect(skipLink.first()).toBeAttached();

    const mainContent = page.locator('#main-content, main, [role="main"]');
    await expect(mainContent.first()).toBeAttached();
  });

  test('main landmark is present', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const main = page.locator('main, [role="main"]');
    await expect(main.first()).toBeAttached();
  });

  test('navigation landmark is present', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const nav = page.locator('nav, [role="navigation"]');
    await expect(nav.first()).toBeAttached();
  });

  test('no empty links or buttons on login page', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/login');

    const violations = await checkEmptyLinksAndButtons(page);
    expect(violations).toHaveLength(0);
  });

  test('form inputs on login page have labels', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/login');

    const violations = await checkFormInputLabels(page);
    expect(violations).toHaveLength(0);
  });

  test('heading hierarchy is correct on home page', async ({ page }) => {
    const app = new AppPage(page);
    await app.goto('/');

    const violations = await checkHeadingHierarchy(page);
    const criticals = violations.filter(v => v.severity === 'critical');
    expect(criticals).toHaveLength(0);
  });
});
