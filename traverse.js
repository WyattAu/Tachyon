const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

const BASE_URL = 'http://192.168.1.191:8080';
const RESULTS_DIR = path.join(__dirname, 'traversal-results');
const DOM_DIR = path.join(RESULTS_DIR, 'dom');
const SCREENSHOT_DIR = path.join(RESULTS_DIR, 'screenshots');
const REPORT_FILE = path.join(RESULTS_DIR, 'traversal-report.json');

// All routes from the Leptos router
const ROUTES = [
  // Public routes
  { path: '/', name: 'home', auth: false },
  { path: '/login', name: 'login', auth: false },
  { path: '/register', name: 'register', auth: false },
  { path: '/local', name: 'local', auth: false },
  { path: '/servers', name: 'servers', auth: false },
  // Protected routes (need auth)
  { path: '/dashboard', name: 'dashboard', auth: true },
  { path: '/documents', name: 'documents', auth: true },
  { path: '/documents/test-doc/edit', name: 'document-edit', auth: true },
  { path: '/documents/test-doc', name: 'document-view', auth: true },
  { path: '/graph', name: 'graph', auth: true },
  { path: '/teams', name: 'teams', auth: true },
  { path: '/teams/test-team', name: 'team-view', auth: true },
  { path: '/search', name: 'search', auth: true },
  { path: '/catalog', name: 'catalog', auth: true },
  { path: '/tags', name: 'tags', auth: true },
  { path: '/settings', name: 'settings', auth: true },
  { path: '/admin/roles', name: 'admin-roles', auth: true },
  { path: '/templates', name: 'templates', auth: true },
  { path: '/plugins', name: 'plugins', auth: true },
  { path: '/spaces', name: 'spaces', auth: true },
  { path: '/ssg', name: 'ssg', auth: true },
  { path: '/billing', name: 'billing', auth: true },
  { path: '/audit', name: 'audit', auth: true },
  { path: '/profile', name: 'profile', auth: true },
  { path: '/onboarding', name: 'onboarding', auth: true },
];

async function login(page) {
  console.log('[AUTH] Logging in...');
  await page.goto(`${BASE_URL}/login`, { waitUntil: 'networkidle', timeout: 30000 });
  await page.waitForTimeout(2000);

  // Try to find and fill login form
  try {
    // Fill server URL if present
    const serverUrl = await page.$('#server-url');
    if (serverUrl) {
      await serverUrl.fill(BASE_URL);
    }

    // Fill username by ID (not the server URL field)
    const usernameInput = await page.$('#username');
    const passwordInput = await page.$('#password');

    if (usernameInput && passwordInput) {
      await usernameInput.fill('admin');
      await passwordInput.fill('admin123');

      // Find and click submit button
      const submitBtn = await page.$('button[type="submit"], button:has-text("Sign"), button:has-text("Login"), button:has-text("Log")');
      if (submitBtn) {
        await submitBtn.click();
        await page.waitForTimeout(3000);
        console.log('[AUTH] Login submitted');
      } else {
        // Try pressing Enter
        await passwordInput.press('Enter');
        await page.waitForTimeout(3000);
        console.log('[AUTH] Login via Enter key');
      }
    } else {
      console.log('[AUTH] Login form not found, trying API login');
      // Try API login
      const response = await page.evaluate(async () => {
        const res = await fetch('/api/v1/auth/login', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ username: 'admin', password: 'admin123' }),
        });
        return res.json();
      });
      if (response.access_token) {
        // Store token in localStorage
        await page.evaluate((token) => {
          localStorage.setItem('auth_token', token);
          localStorage.setItem('access_token', token);
        }, response.access_token);
        console.log('[AUTH] API login successful, token stored');
      }
    }
  } catch (e) {
    console.log('[AUTH] Login error:', e.message);
  }

  // Check if we're logged in by navigating to dashboard
  await page.goto(`${BASE_URL}/dashboard`, { waitUntil: 'networkidle', timeout: 15000 });
  await page.waitForTimeout(2000);
  const url = page.url();
  console.log('[AUTH] After login, URL:', url);
}

async function captureRoute(page, route, index) {
  const result = {
    index,
    path: route.path,
    name: route.name,
    auth: route.auth,
    url: '',
    status: 'unknown',
    httpStatus: null,
    domSnapshot: null,
    screenshotPath: null,
    errors: [],
    warnings: [],
    consoleMessages: [],
    networkErrors: [],
    loadTime: 0,
    hasContent: false,
    title: '',
    elementCount: 0,
    linkCount: 0,
    buttonCount: 0,
    inputCount: 0,
    imageCount: 0,
    a11yIssues: [],
  };

  const consoleMsgs = [];
  const networkErrors = [];

  // Listen for console messages
  page.on('console', msg => {
    if (msg.type() === 'error' || msg.type() === 'warning') {
      consoleMsgs.push({ type: msg.type(), text: msg.text() });
    }
  });

  // Listen for network errors
  page.on('requestfailed', req => {
    networkErrors.push({ url: req.url(), failure: req.failure()?.errorText });
  });

  const startTime = Date.now();

  try {
    const fullUrl = `${BASE_URL}${route.path}`;
    console.log(`[${String(index + 1).padStart(2, ' ')}/${ROUTES.length}] ${route.name.padEnd(20)} ${route.path}`);

    const response = await page.goto(fullUrl, {
      waitUntil: 'networkidle',
      timeout: 20000,
    });

    result.httpStatus = response?.status() || null;
    result.url = page.url();

    // Wait for content to render (Leptos WASM needs time)
    await page.waitForTimeout(3000);

    result.loadTime = Date.now() - startTime;
    result.title = await page.title();

    // Check if page has meaningful content
    const bodyText = await page.evaluate(() => document.body?.innerText || '');
    result.hasContent = bodyText.trim().length > 50;

    // Count elements
    const counts = await page.evaluate(() => ({
      elements: document.querySelectorAll('*').length,
      links: document.querySelectorAll('a').length,
      buttons: document.querySelectorAll('button').length,
      inputs: document.querySelectorAll('input, textarea, select').length,
      images: document.querySelectorAll('img').length,
    }));
    result.elementCount = counts.elements;
    result.linkCount = counts.links;
    result.buttonCount = counts.buttons;
    result.inputCount = counts.inputs;
    result.imageCount = counts.images;

    // Check for accessibility issues
    const a11y = await page.evaluate(() => {
      const issues = [];
      // Check for missing alt text on images
      document.querySelectorAll('img:not([alt])').forEach((img, i) => {
        if (i < 5) issues.push(`Image missing alt: ${img.src?.substring(0, 80)}`);
      });
      // Check for missing labels on inputs
      document.querySelectorAll('input:not([aria-label]):not([id])').forEach((input, i) => {
        if (i < 5) issues.push(`Input missing label: type=${input.type}`);
      });
      // Check for empty buttons
      document.querySelectorAll('button').forEach((btn, i) => {
        if (!btn.textContent?.trim() && !btn.getAttribute('aria-label') && i < 5) {
          issues.push('Button missing text/aria-label');
        }
      });
      return issues;
    });
    result.a11yIssues = a11y;

    // Determine status
    if (result.httpStatus === 401 || result.httpStatus === 403) {
      result.status = 'auth_required';
    } else if (result.httpStatus >= 500) {
      result.status = 'server_error';
    } else if (result.httpStatus === 404) {
      result.status = 'not_found';
    } else if (!result.hasContent) {
      result.status = 'empty';
    } else {
      result.status = 'ok';
    }

    // Capture DOM snapshot
    const domSnapshot = await page.evaluate(() => {
      return document.documentElement.outerHTML;
    });
    const domPath = path.join(DOM_DIR, `${route.name}.html`);
    fs.writeFileSync(domPath, domSnapshot);
    result.domSnapshot = domPath;

    // Capture screenshot
    const screenshotPath = path.join(SCREENSHOT_DIR, `${route.name}.png`);
    await page.screenshot({ path: screenshotPath, fullPage: true });
    result.screenshotPath = screenshotPath;

    result.consoleMessages = consoleMsgs;
    result.networkErrors = networkErrors;

    const statusIcon = result.status === 'ok' ? '✓' : result.status === 'auth_required' ? '🔒' : result.status === 'empty' ? '⚠' : '✗';
    console.log(`         ${statusIcon} ${result.status} | ${result.httpStatus} | ${result.loadTime}ms | ${result.elementCount} elem | ${result.a11yIssues.length} a11y`);

  } catch (e) {
    result.status = 'error';
    result.errors.push(e.message);
    result.loadTime = Date.now() - startTime;
    console.log(`         ✗ ERROR: ${e.message.substring(0, 100)}`);
  }

  return result;
}

async function main() {
  // Create directories
  fs.mkdirSync(DOM_DIR, { recursive: true });
  fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });

  console.log('=== Tachyon Full App Traversal ===');
  console.log(`Target: ${BASE_URL}`);
  console.log(`Routes: ${ROUTES.length}`);
  console.log(`Results: ${RESULTS_DIR}`);
  console.log('');

  const browser = await chromium.launch({
    headless: true,
    args: ['--no-sandbox', '--disable-setuid-sandbox'],
  });

  const context = await browser.newContext({
    viewport: { width: 1440, height: 900 },
    userAgent: 'Tachyon-Traversal/1.0',
  });

  const page = await context.newPage();

  // Login first
  await login(page);

  // Traverse all routes
  const results = [];
  for (let i = 0; i < ROUTES.length; i++) {
    let result;
    try {
      result = await captureRoute(page, ROUTES[i], i);
    } catch (e) {
      console.log(`         ✗ FATAL: ${e.message.substring(0, 80)}`);
      // Browser/page crashed - recreate
      try { await page.close(); } catch (_) {}
      try {
        page = await context.newPage();
        result = {
          index: i, path: ROUTES[i].path, name: ROUTES[i].name,
          auth: ROUTES[i].auth, status: 'error', errors: [e.message],
          httpStatus: null, domSnapshot: null, screenshotPath: null,
          consoleMessages: [], networkErrors: [], loadTime: 0,
          hasContent: false, title: '', elementCount: 0, linkCount: 0,
          buttonCount: 0, inputCount: 0, imageCount: 0, a11yIssues: [],
          url: '',
        };
      } catch (_) {
        result = { index: i, path: ROUTES[i].path, name: ROUTES[i].name, status: 'error', errors: ['browser crashed'], httpStatus: null, domSnapshot: null, screenshotPath: null, consoleMessages: [], networkErrors: [], loadTime: 0, hasContent: false, title: '', elementCount: 0, linkCount: 0, buttonCount: 0, inputCount: 0, imageCount: 0, a11yIssues: [], url: '', auth: ROUTES[i].auth };
      }
    }
    results.push(result);

    // Clear listeners between routes
    try {
      page.removeAllListeners('console');
      page.removeAllListeners('requestfailed');
    } catch (_) {}

    // Small delay between routes
    try { await page.waitForTimeout(500); } catch (_) {}
  }

  // Generate report
  const report = {
    timestamp: new Date().toISOString(),
    baseUrl: BASE_URL,
    totalRoutes: ROUTES.length,
    summary: {
      ok: results.filter(r => r.status === 'ok').length,
      auth_required: results.filter(r => r.status === 'auth_required').length,
      empty: results.filter(r => r.status === 'empty').length,
      not_found: results.filter(r => r.status === 'not_found').length,
      server_error: results.filter(r => r.status === 'server_error').length,
      error: results.filter(r => r.status === 'error').length,
    },
    totalA11yIssues: results.reduce((sum, r) => sum + r.a11yIssues.length, 0),
    results,
  };

  fs.writeFileSync(REPORT_FILE, JSON.stringify(report, null, 2));

  // Print summary
  console.log('');
  console.log('=== TRAVERSAL SUMMARY ===');
  console.log(`OK: ${report.summary.ok}`);
  console.log(`Auth Required: ${report.summary.auth_required}`);
  console.log(`Empty: ${report.summary.empty}`);
  console.log(`Not Found: ${report.summary.not_found}`);
  console.log(`Server Error: ${report.summary.server_error}`);
  console.log(`Error: ${report.summary.error}`);
  console.log(`Total A11y Issues: ${report.totalA11yIssues}`);
  console.log(`Report: ${REPORT_FILE}`);

  // Print issues to debug
  console.log('');
  console.log('=== ISSUES REQUIRING DEBUG ===');
  for (const r of results) {
    if (r.status !== 'ok') {
      console.log(`[${r.status}] ${r.path} - ${r.errors.join(', ') || 'no content'}`);
    }
    if (r.a11yIssues.length > 0) {
      for (const issue of r.a11yIssues) {
        console.log(`  [a11y] ${r.path}: ${issue}`);
      }
    }
    if (r.networkErrors.length > 0) {
      for (const ne of r.networkErrors) {
        console.log(`  [net] ${r.path}: ${ne.url} - ${ne.failure}`);
      }
    }
  }

  await browser.close();
}

main().catch(e => {
  console.error('Fatal error:', e);
  process.exit(1);
});
