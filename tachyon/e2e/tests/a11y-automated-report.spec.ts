import { test, expect } from '@playwright/test';
import { AppPage } from './helpers';

interface A11yViolation {
  rule: string;
  severity: 'critical' | 'serious' | 'moderate' | 'minor';
  wcag: string;
  description: string;
  selector: string;
}

interface PageAuditResult {
  pageName: string;
  url: string;
  violations: A11yViolation[];
  summary: {
    critical: number;
    serious: number;
    moderate: number;
    minor: number;
    total: number;
  };
}

// ---------------------------------------------------------------------------
// Audit helpers — each returns A11yViolation[] for a given page
// ---------------------------------------------------------------------------

async function auditImages(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];
  const results = await page.evaluate(() => {
    const issues: { src: string; hasAlt: boolean; alt: string | null; isDecorative: boolean }[] = [];
    const images = document.querySelectorAll('img');
    for (const img of images) {
      if (img.offsetWidth === 0 && img.offsetHeight === 0) continue;
      const role = img.getAttribute('role');
      const isDecorative = role === 'presentation' || role === 'none';
      const alt = img.getAttribute('alt');
      issues.push({
        src: (img.getAttribute('src') ?? '').slice(0, 100),
        hasAlt: alt !== null,
        alt,
        isDecorative,
      });
    }
    return issues;
  });

  for (const img of results) {
    if (img.isDecorative) continue;
    if (!img.hasAlt) {
      violations.push({
        rule: 'img-alt-mandatory',
        severity: 'critical',
        wcag: '1.1.1',
        description: `Image missing alt attribute: ${img.src}`,
        selector: `img[src="${img.src}"]`,
      });
    } else if (img.alt === '') {
      violations.push({
        rule: 'img-alt-empty',
        severity: 'minor',
        wcag: '1.1.1',
        description: `Image has empty alt="" but is not marked decorative (add role="presentation"): ${img.src}`,
        selector: `img[src="${img.src}"]`,
      });
    }
  }
  return violations;
}

async function auditSvgIcons(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];
  const results = await page.evaluate(() => {
    const issues: { hasAriaHidden: boolean; hasRole: boolean; roleValue: string | null; hasTitle: boolean; hasAriaLabel: boolean; parentAriaHidden: boolean }[] = [];
    const svgs = document.querySelectorAll('svg');
    for (const svg of svgs) {
      if (svg.offsetWidth === 0 && svg.offsetHeight === 0) continue;
      issues.push({
        hasAriaHidden: svg.getAttribute('aria-hidden') === 'true',
        hasRole: svg.hasAttribute('role'),
        roleValue: svg.getAttribute('role'),
        hasTitle: svg.querySelector('title') !== null,
        hasAriaLabel: svg.hasAttribute('aria-label'),
        parentAriaHidden: svg.parentElement?.getAttribute('aria-hidden') === 'true',
      });
    }
    return issues;
  });

  for (const svg of results) {
    if (svg.hasAriaHidden || svg.parentAriaHidden) continue;
    if (svg.roleValue === 'presentation' || svg.roleValue === 'none') continue;
    const hasAccessibleName = svg.hasAriaLabel || (svg.hasRole && svg.roleValue === 'img' && svg.hasTitle);
    if (!hasAccessibleName) {
      violations.push({
        rule: 'svg-accessible-name',
        severity: 'serious',
        wcag: '1.1.1',
        description: 'SVG icon lacks accessible name. Add aria-hidden="true" if decorative, or role="img" with <title>/aria-label if informative.',
        selector: 'svg',
      });
    }
  }
  return violations;
}

async function auditFormLabels(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];
  const results = await page.evaluate(() => {
    const issues: { tag: string; name: string; type: string; hasLabel: boolean; labelMethod: string }[] = [];
    const inputs = document.querySelectorAll(
      'input:not([type="hidden"]):not([type="submit"]):not([type="button"]):not([type="reset"]):not([type="image"]), textarea, select'
    );
    for (const input of inputs) {
      if (input.offsetWidth === 0 && input.offsetHeight === 0) continue;
      const ariaLabel = input.getAttribute('aria-label');
      const ariaLabelledby = input.getAttribute('aria-labelledby');
      const id = input.getAttribute('id');
      const placeholder = input.getAttribute('placeholder');
      const title = input.getAttribute('title');
      const wrappingLabel = input.closest('label');

      let hasLabel = false;
      let labelMethod = '';

      if (ariaLabel) { hasLabel = true; labelMethod = 'aria-label'; }
      else if (ariaLabelledby) { hasLabel = true; labelMethod = 'aria-labelledby'; }
      else if (id && document.querySelector(`label[for="${id}"]`)) { hasLabel = true; labelMethod = 'label[for]'; }
      else if (placeholder && placeholder.trim()) { hasLabel = true; labelMethod = 'placeholder'; }
      else if (title && title.trim()) { hasLabel = true; labelMethod = 'title'; }
      else if (wrappingLabel) { hasLabel = true; labelMethod = 'wrapping <label>'; }

      issues.push({
        tag: input.tagName.toLowerCase(),
        name: input.getAttribute('name') ?? 'unnamed',
        type: input.getAttribute('type') ?? '',
        hasLabel,
        labelMethod,
      });
    }
    return issues;
  });

  for (const input of results) {
    if (!input.hasLabel) {
      violations.push({
        rule: 'label',
        severity: 'critical',
        wcag: '1.3.1',
        description: `Form <${input.tag}> "${input.name}" has no associated label, aria-label, aria-labelledby, or placeholder`,
        selector: `${input.tag}[name="${input.name}"]`,
      });
    } else if (input.labelMethod === 'placeholder' || input.labelMethod === 'title') {
      violations.push({
        rule: 'label',
        severity: 'moderate',
        wcag: '1.3.1',
        description: `Form <${input.tag}> "${input.name}" uses ${input.labelMethod} instead of a proper <label> element`,
        selector: `${input.tag}[name="${input.name}"]`,
      });
    }
  }
  return violations;
}

async function auditRequiredFields(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];
  const results = await page.evaluate(() => {
    const issues: { name: string; required: boolean; ariaRequired: string | null; visualHint: boolean }[] = [];
    const inputs = document.querySelectorAll('input, textarea, select');
    for (const input of inputs) {
      if (input.offsetWidth === 0 && input.offsetHeight === 0) continue;
      const required = (input as HTMLInputElement).required;
      const ariaRequired = input.getAttribute('aria-required');
      const id = input.getAttribute('id');
      let labelText = '';
      if (id) {
        const label = document.querySelector(`label[for="${id}"]`);
        if (label) labelText = (label.textContent ?? '').trim();
      }
      const ariaLabel = input.getAttribute('aria-label') ?? '';
      const visualHint = /\*|required|mandatory/i.test(ariaLabel + ' ' + labelText);

      issues.push({
        name: input.getAttribute('name') ?? 'unnamed',
        required,
        ariaRequired,
        visualHint,
      });
    }
    return issues;
  });

  for (const input of results) {
    if (input.visualHint && !input.required && input.ariaRequired !== 'true') {
      violations.push({
        rule: 'aria-required',
        severity: 'serious',
        wcag: '3.3.2',
        description: `Input "${input.name}" visually indicates required but lacks required or aria-required="true"`,
        selector: `input[name="${input.name}"]`,
      });
    }
    if (input.required && input.ariaRequired === null) {
      violations.push({
        rule: 'aria-required',
        severity: 'minor',
        wcag: '3.3.2',
        description: `Input "${input.name}" uses native required but should also have aria-required="true" for screen reader consistency`,
        selector: `input[name="${input.name}"]`,
      });
    }
  }
  return violations;
}

async function auditHeadingHierarchy(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];
  const results = await page.evaluate(() => {
    const issues: { prevLevel: number; level: number; text: string }[] = [];
    const headings = document.querySelectorAll('h1, h2, h3, h4, h5, h6');
    let prevLevel = 0;
    for (const heading of headings) {
      if (heading.offsetWidth === 0 && heading.offsetHeight === 0) continue;
      const tag = heading.tagName.toLowerCase();
      const level = parseInt(tag.replace('h', ''), 10);
      const text = (heading.textContent ?? '').trim();
      if (prevLevel > 0 && level > prevLevel + 1) {
        issues.push({ prevLevel, level, text: text.slice(0, 50) });
      }
      prevLevel = level;
    }
    return issues;
  });

  for (const issue of results) {
    violations.push({
      rule: 'heading-order',
      severity: 'moderate',
      wcag: '1.3.1',
      description: `Heading level skipped: h${issue.prevLevel} → h${issue.level} ("${issue.text}")`,
      selector: `h${issue.level}`,
    });
  }
  return violations;
}

async function auditLandmarks(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];
  const landmarkInfo = await page.evaluate(() => {
    const main = document.querySelector('main') || document.querySelector('[role="main"]');
    const nav = document.querySelector('nav') || document.querySelector('[role="navigation"]');
    return {
      hasMain: !!main,
      mainTag: main?.tagName.toLowerCase() ?? null,
      hasNav: !!nav,
      navCount: document.querySelectorAll('nav, [role="navigation"]').length,
    };
  });

  if (!landmarkInfo.hasMain) {
    violations.push({
      rule: 'main',
      severity: 'critical',
      wcag: '1.3.1',
      description: 'No <main> or [role="main"] landmark found. Screen readers need this to navigate to primary content.',
      selector: 'main',
    });
  }
  if (!landmarkInfo.hasNav) {
    violations.push({
      rule: 'landmark',
      severity: 'serious',
      wcag: '2.4.1',
      description: 'No <nav> or [role="navigation"] landmark found.',
      selector: 'nav',
    });
  }
  return violations;
}

async function auditSkipLink(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];
  const info = await page.evaluate(() => {
    const skipLinks = document.querySelectorAll('a.sr-only, a[href="#main-content"], a[href="#main"], a[href="#main-nav"], a[class*="skip"]');
    if (skipLinks.length === 0) return { found: false };
    const link = skipLinks[0];
    const href = link.getAttribute('href');
    let targetExists = false;
    if (href && href.startsWith('#')) {
      targetExists = !!document.getElementById(href.slice(1));
    }
    return { found: true, href, targetExists };
  });

  if (!info.found) {
    violations.push({
      rule: 'skip-link',
      severity: 'serious',
      wcag: '2.4.1',
      description: 'No skip navigation link found. Add a visually hidden link as the first focusable element that targets #main-content.',
      selector: 'a.sr-only',
    });
  } else if (info.href && info.href.startsWith('#') && !info.targetExists) {
    violations.push({
      rule: 'skip-link-target',
      severity: 'serious',
      wcag: '2.4.1',
      description: `Skip link targets ${info.href} but that element does not exist in the DOM.`,
      selector: `a[href="${info.href}"]`,
    });
  }
  return violations;
}

async function auditDocumentLang(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];
  const lang = await page.getAttribute('html', 'lang');
  if (!lang) {
    violations.push({
      rule: 'html-lang',
      severity: 'serious',
      wcag: '3.1.1',
      description: '<html> element has no lang attribute. Screen readers use this to select the correct voice/prounciation.',
      selector: 'html',
    });
  } else if (lang.length < 2) {
    violations.push({
      rule: 'html-lang-valid',
      severity: 'serious',
      wcag: '3.1.1',
      description: `lang="${lang}" is too short. Use a valid BCP 47 tag (e.g. "en", "en-US").`,
      selector: 'html',
    });
  }
  return violations;
}

async function auditPageTitle(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];
  const title = await page.title();
  if (!title || title.trim().length === 0) {
    violations.push({
      rule: 'document-title',
      severity: 'serious',
      wcag: '2.4.2',
      description: 'Page has no <title> or title is empty.',
      selector: 'title',
    });
  } else if (title.trim().length < 3) {
    violations.push({
      rule: 'document-title-descriptive',
      severity: 'moderate',
      wcag: '2.4.2',
      description: `Page title "${title}" is too short to be descriptive. Include page-specific context.`,
      selector: 'title',
    });
  }
  return violations;
}

async function auditAriaReferenceIntegrity(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];
  const results = await page.evaluate(() => {
    const issues: { attr: string; missingId: string }[] = [];
    const attrs = ['aria-labelledby', 'aria-describedby', 'aria-controls', 'aria-flowto', 'aria-owns'];
    for (const attr of attrs) {
      const els = document.querySelectorAll(`[${attr}]`);
      for (const el of els) {
        const value = el.getAttribute(attr) ?? '';
        const ids = value.split(/\s+/);
        for (const id of ids) {
          if (id && !document.getElementById(id)) {
            issues.push({ attr, missingId: id });
          }
        }
      }
    }
    return issues;
  });

  for (const issue of results) {
    const severity = (issue.attr === 'aria-labelledby' || issue.attr === 'aria-describedby') ? 'critical' : 'moderate';
    const wcag = issue.attr === 'aria-labelledby' ? '1.3.1' : '4.1.2';
    violations.push({
      rule: `aria-valid-attr-value-${issue.attr}`,
      severity,
      wcag,
      description: `${issue.attr}="${issue.missingId}" references element #${issue.missingId} which does not exist in the DOM.`,
      selector: `[${issue.attr}]`,
    });
  }
  return violations;
}

async function auditInteractiveSemantics(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];
  const results = await page.evaluate(() => {
    const issues: { tag: string; reason: string; className: string }[] = [];
    const badClickables = document.querySelectorAll(
      '[onclick]:not(button):not(a):not(input):not(select):not(textarea):not([role="button"]):not([role="link"]):not(summary)'
    );
    for (const el of badClickables) {
      if (el.offsetWidth === 0 && el.offsetHeight === 0) continue;
      const tabindex = el.getAttribute('tabindex');
      if (!tabindex || tabindex === '-1') {
        issues.push({
          tag: el.tagName.toLowerCase(),
          reason: 'Has onclick but is not a semantic element, has no role, and no tabindex',
          className: el.className.toString().slice(0, 80),
        });
      }
    }
    return issues;
  });

  for (const issue of results) {
    violations.push({
      rule: 'interactive-semantics',
      severity: 'serious',
      wcag: '2.1.1',
      description: `<${issue.tag}.${issue.className}> has an onclick handler but is not keyboard accessible. Use <button>, <a>, or add role="button" with tabindex="0".`,
      selector: `${issue.tag}[onclick]`,
    });
  }
  return violations;
}

async function auditEmptyLinksAndButtons(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
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

    if (!text && !ariaLabel && !ariaLabelledby && !title && !imgAlt) {
      violations.push({
        rule: 'link-name',
        severity: 'critical',
        wcag: '2.4.4',
        description: 'Link has no accessible name (no text, aria-label, aria-labelledby, title, or img alt).',
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

    if (!text && !ariaLabel && !ariaLabelledby && !title && !imgAlt) {
      violations.push({
        rule: 'button-name',
        severity: 'critical',
        wcag: '4.1.2',
        description: 'Button has no accessible name (no text, aria-label, aria-labelledby, title, or img alt).',
        selector: 'button',
      });
    }
  }
  return violations;
}

async function auditFocusVisible(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];
  const hasFocusVisibleCSS = await page.evaluate(() => {
    const styleSheets = document.styleSheets;
    for (let i = 0; i < styleSheets.length; i++) {
      try {
        const rules = (styleSheets[i] as CSSStyleSheet).cssRules;
        for (let j = 0; j < rules.length; j++) {
          const text = rules[j].cssText;
          if (text.includes('focus-visible') || text.includes(':focus')) {
            const hasOutline = text.includes('outline') && !text.includes('outline: none') && !text.includes('outline:none');
            if (hasOutline) return true;
          }
        }
      } catch {
        // cross-origin
      }
    }
    return false;
  });

  if (!hasFocusVisibleCSS) {
    violations.push({
      rule: 'focus-visible',
      severity: 'serious',
      wcag: '2.4.7',
      description: 'No CSS rule found that provides a visible focus indicator (:focus-visible or :focus with outline).',
      selector: ':focus',
    });
  }
  return violations;
}

async function auditTables(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];
  const results = await page.evaluate(() => {
    const issues: { reason: string; index: number }[] = [];
    const tables = document.querySelectorAll('table');
    for (let i = 0; i < tables.length; i++) {
      const table = tables[i];
      if (table.offsetWidth === 0 && table.offsetHeight === 0) continue;
      const role = table.getAttribute('role');
      if (role === 'presentation' || role === 'none') continue;

      const ths = table.querySelectorAll('th');
      if (ths.length === 0) {
        issues.push({ reason: 'Data table has no <th> elements', index: i });
        continue;
      }

      let headerWithoutScope = 0;
      for (const th of ths) {
        const scope = th.getAttribute('scope');
        const id = th.getAttribute('id');
        if (!scope && !id) {
          headerWithoutScope++;
        }
      }
      if (headerWithoutScope > 0) {
        issues.push({ reason: `${headerWithoutScope} <th> element(s) have no scope attribute`, index: i });
      }

      const caption = table.querySelector('caption');
      if (!caption) {
        const ariaLabel = table.getAttribute('aria-label');
        const ariaLabelledby = table.getAttribute('aria-labelledby');
        if (!ariaLabel && !ariaLabelledby) {
          issues.push({ reason: 'Table has no <caption>, aria-label, or aria-labelledby', index: i });
        }
      }
    }
    return issues;
  });

  for (const issue of results) {
    violations.push({
      rule: 'table-headers',
      severity: 'serious',
      wcag: '1.3.1',
      description: `Table #${issue.index}: ${issue.reason}`,
      selector: 'table',
    });
  }
  return violations;
}

async function auditLiveRegions(page: import('@playwright/test').Page): Promise<A11yViolation[]> {
  const violations: A11yViolation[] = [];
  const results = await page.evaluate(() => {
    const issues: { live: string | null; hasContent: boolean; selector: string }[] = [];
    const liveEls = document.querySelectorAll('[aria-live]');
    for (const el of liveEls) {
      if (el.offsetWidth === 0 && el.offsetHeight === 0) continue;
      const live = el.getAttribute('aria-live');
      if (live !== 'polite' && live !== 'assertive') {
        issues.push({
          live,
          hasContent: (el.textContent ?? '').trim().length > 0,
          selector: el.tagName.toLowerCase(),
        });
      }
    }
    return issues;
  });

  for (const issue of results) {
    violations.push({
      rule: 'aria-live-value',
      severity: 'moderate',
      wcag: '4.1.3',
      description: `Element with aria-live="${issue.live}" should use "polite" or "assertive".`,
      selector: issue.selector,
    });
  }
  return violations;
}

// ---------------------------------------------------------------------------
// Full audit runner
// ---------------------------------------------------------------------------

async function runFullAudit(page: import('@playwright/test').Page, pageName: string, url: string): Promise<PageAuditResult> {
  const violations = [
    ...(await auditImages(page)),
    ...(await auditSvgIcons(page)),
    ...(await auditFormLabels(page)),
    ...(await auditRequiredFields(page)),
    ...(await auditHeadingHierarchy(page)),
    ...(await auditLandmarks(page)),
    ...(await auditSkipLink(page)),
    ...(await auditDocumentLang(page)),
    ...(await auditPageTitle(page)),
    ...(await auditAriaReferenceIntegrity(page)),
    ...(await auditInteractiveSemantics(page)),
    ...(await auditEmptyLinksAndButtons(page)),
    ...(await auditFocusVisible(page)),
    ...(await auditTables(page)),
    ...(await auditLiveRegions(page)),
  ];

  return {
    pageName,
    url,
    violations,
    summary: {
      critical: violations.filter(v => v.severity === 'critical').length,
      serious: violations.filter(v => v.severity === 'serious').length,
      moderate: violations.filter(v => v.severity === 'moderate').length,
      minor: violations.filter(v => v.severity === 'minor').length,
      total: violations.length,
    },
  };
}

function formatReport(results: PageAuditResult[]): string {
  const lines: string[] = [];
  lines.push('='.repeat(80));
  lines.push('  ACCESSIBILITY AUTOMATED REPORT — WCAG 2.1');
  lines.push('='.repeat(80));
  lines.push('');

  let totalCritical = 0;
  let totalSerious = 0;
  let totalModerate = 0;
  let totalMinor = 0;

  for (const result of results) {
    const { summary, pageName, url, violations } = result;
    totalCritical += summary.critical;
    totalSerious += summary.serious;
    totalModerate += summary.moderate;
    totalMinor += summary.minor;

    const status = summary.critical === 0 && summary.serious === 0 ? 'PASS' : 'FAIL';
    lines.push(`[${status}] ${pageName} — ${url}`);
    lines.push(`     ${summary.critical} critical | ${summary.serious} serious | ${summary.moderate} moderate | ${summary.minor} minor`);
    lines.push('');

    const actionable = violations.filter(v => v.severity === 'critical' || v.severity === 'serious');
    for (const v of actionable) {
      lines.push(`     [${v.severity.toUpperCase()}] ${v.rule} (WCAG ${v.wcag})`);
      lines.push(`       ${v.description}`);
      lines.push(`       Selector: ${v.selector}`);
      lines.push('');
    }
  }

  lines.push('-'.repeat(80));
  lines.push('TOTALS');
  lines.push('-'.repeat(80));
  lines.push(`  Critical:  ${totalCritical}`);
  lines.push(`  Serious:   ${totalSerious}`);
  lines.push(`  Moderate:  ${totalModerate}`);
  lines.push(`  Minor:     ${totalMinor}`);
  lines.push(`  ─────────────────`);
  lines.push(`  Grand total: ${totalCritical + totalSerious + totalModerate + totalMinor}`);

  const overallStatus = totalCritical === 0 && totalSerious === 0 ? 'PASS' : 'FAIL';
  lines.push('');
  lines.push(`Overall: ${overallStatus}`);
  lines.push('='.repeat(80));

  return lines.join('\n');
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

test.describe('Accessibility Automated Report', () => {
  const pagesToAudit = [
    { name: 'Home', path: '/' },
    { name: 'Login', path: '/login' },
    { name: 'Register', path: '/register' },
    { name: '404', path: '/this-page-does-not-exist' },
  ];

  for (const { name, path } of pagesToAudit) {
    test(`${name} page audit`, async ({ page }) => {
      const app = new AppPage(page);
      await app.goto(path);

      const result = await runFullAudit(page, name, path);

      const report = formatReport([result]);
      console.log('\n' + report);

      expect(result.summary.critical).toBe(0);
    });
  }

  test('full cross-page audit summary', async ({ page }) => {
    const app = new AppPage(page);
    const allResults: PageAuditResult[] = [];

    for (const { name, path } of pagesToAudit) {
      const result = await runFullAudit(page, name, path);
      allResults.push(result);
    }

    const report = formatReport(allResults);
    console.log('\n' + report);

    const totalCritical = allResults.reduce((sum, r) => sum + r.summary.critical, 0);
    const totalSerious = allResults.reduce((sum, r) => sum + r.summary.serious, 0);

    expect(totalCritical).toBe(0);
  });
});
