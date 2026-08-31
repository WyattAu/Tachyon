const puppeteer = require('puppeteer-core');

const CHROME = '/nix/store/mwmmlaalpfnaaknbrh333agk5a01cg68-chromium-146.0.7680.164/bin/chromium';
const BASE = 'http://localhost:3000';
const API = 'http://localhost:8080/api/v1';

const results = [];

function log(msg) {
  const ts = new Date().toISOString().slice(11, 23);
  console.log(`[${ts}] ${msg}`);
}

async function test(name, fn) {
  try {
    await fn();
    results.push({ name, status: 'PASS' });
    log(`PASS: ${name}`);
  } catch (e) {
    results.push({ name, status: 'FAIL', error: e.message });
    log(`FAIL: ${name}: ${e.message}`);
  }
}

function wait(ms) {
  return new Promise(r => setTimeout(r, ms));
}

const IGNORED_ERROR_PATTERNS = [
  /Manifest.*Syntax error/i,
  /unsupported MIME type/i,
  /401.*Unauthorized/i,
  /Failed to load resource.*401/i,
  /Failed to load resource.*404/i,
  /429.*Too Many Requests/i,
  /Failed to load resource.*429/i,
  /400.*Bad Request/i,
  /Failed to load resource.*400/i,
  /tailwind is not defined/i,
  /net::ERR_/i,
  /service worker/i,
  /sw\.js/i,
];

function isIgnoredError(msg) {
  if (!msg) return true;
  return IGNORED_ERROR_PATTERNS.some(p => p.test(msg));
}

function collectErrors(page) {
  const errors = [];
  page.on('pageerror', err => {
    if (!isIgnoredError(err.message)) errors.push(err.message);
  });
  page.on('console', msg => {
    if (msg.type() === 'error' && !isIgnoredError(msg.text())) {
      errors.push(msg.text());
    }
  });
  return errors;
}

async function navigateAndWait(page, url, waitMs = 3000) {
  const errors = collectErrors(page);
  await page.goto(url, { waitUntil: 'networkidle2', timeout: 15000 }).catch(() => {});
  await wait(waitMs);
  return errors;
}

async function dismissOnboarding(page) {
  await wait(1000);
  const skipped = await page.evaluate(() => {
    const buttons = Array.from(document.querySelectorAll('button'));
    const skipBtn = buttons.find(b => b.textContent.trim() === 'Skip');
    const continueBtn = buttons.find(b => b.textContent.trim() === 'Continue');
    const closeBtn = buttons.find(b =>
      b.textContent.trim() === 'Close' || b.textContent.trim() === 'Done' || b.textContent.trim() === 'Finish'
    );
    if (skipBtn) { skipBtn.click(); return 'skip'; }
    if (closeBtn) { closeBtn.click(); return 'close'; }
    if (continueBtn) { continueBtn.click(); return 'continue'; }
    return null;
  });
  if (skipped) {
    log(`  Dismissed onboarding via "${skipped}" button`);
    await wait(500);
  }
}

(async () => {
  const browser = await puppeteer.launch({
    executablePath: CHROME,
    headless: 'new',
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-web-security'],
  });

  try {
    const page = await browser.newPage();
    await page.setViewport({ width: 1280, height: 900 });

    const client = await page.createCDPSession();
    await client.send('Network.clearBrowserCache');
    await client.send('Network.clearBrowserCookies');

    const timestamp = Date.now();
    const testEmail = `test-${timestamp}@tachyon.test`;
    const testPassword = 'TestPass123!';
    const testUsername = `testuser-${timestamp}`;
    let createdDocId = null;

    // =========================================================================
    // Test 1: All routes render (smoke test)
    // =========================================================================
    await test('All 12 routes render without errors', async () => {
      const routes = [
        '/', '/login', '/register', '/dashboard', '/documents',
        '/search', '/teams', '/billing', '/audit', '/ssg', '/tags', '/spaces',
      ];

      for (const route of routes) {
        const errors = await navigateAndWait(page, `${BASE}${route}`);

        if (errors.length > 0) {
          throw new Error(`JS errors on ${route}: ${errors.slice(0, 3).join('; ')}`);
        }

        const bodyText = await page.evaluate(() => document.body?.innerText?.trim() || '');
        if (bodyText.length === 0) {
          throw new Error(`Empty body on ${route}`);
        }
      }
    });

    // =========================================================================
    // Test 2: Registration
    // =========================================================================
    await test('Registration creates account and navigates', async () => {
      const errors = await navigateAndWait(page, `${BASE}/register`);
      if (errors.length > 0) {
        throw new Error(`JS errors on /register: ${errors.slice(0, 3).join('; ')}`);
      }

      await dismissOnboarding(page);

      await page.waitForSelector('#reg-username', { timeout: 10000 });

      await page.type('#reg-username', testUsername, { delay: 30 });
      await page.type('#reg-email', testEmail, { delay: 30 });
      await page.type('#reg-password', testPassword, { delay: 30 });
      await page.type('#reg-confirm', testPassword, { delay: 30 });

      await Promise.all([
        page.waitForFunction(
          () => window.location.pathname.includes('/dashboard') || window.location.pathname.includes('/login'),
          { timeout: 15000 }
        ).catch(() => {}),
        page.evaluate(() => document.querySelector('button[type="submit"]')?.click()),
      ]);

      await wait(3000);

      const currentUrl = page.url();
      const isDashboard = currentUrl.includes('/dashboard');
      const isLogin = currentUrl.includes('/login');

      if (!isDashboard && !isLogin) {
        const bodyText = await page.evaluate(() => document.body?.innerText || '');
        const hasError = bodyText.toLowerCase().includes('registration failed') ||
          bodyText.toLowerCase().includes('already exists');
        if (hasError) {
          throw new Error(`Registration showed error. URL: ${currentUrl}`);
        }
        throw new Error(`Registration did not navigate to dashboard or login. URL: ${currentUrl}`);
      }

      if (isLogin) {
        log('  Registration navigated to login (auto-login may be disabled)');
      }
    });

    // =========================================================================
    // Test 3: Login + Auth persistence
    // =========================================================================
    await test('Login stores token and navigates to dashboard', async () => {
      const errors = await navigateAndWait(page, `${BASE}/login`);
      if (errors.length > 0) {
        throw new Error(`JS errors on /login: ${errors.slice(0, 3).join('; ')}`);
      }

      await dismissOnboarding(page);

      await page.waitForSelector('#username', { timeout: 10000 });

      await page.type('#username', testUsername, { delay: 30 });
      await page.type('#password', testPassword, { delay: 30 });

      await Promise.all([
        page.waitForFunction(
          () => window.location.pathname.includes('/dashboard') || window.location.pathname.includes('/documents'),
          { timeout: 15000 }
        ).catch(() => {}),
        page.evaluate(() => document.querySelector('button[type="submit"]')?.click()),
      ]);

      await wait(3000);

      const token = await page.evaluate(() => {
        try { return localStorage.getItem('tachyon_token'); } catch { return null; }
      });

      if (!token) {
        throw new Error('tachyon_token not found in localStorage after login');
      }

      const currentUrl = page.url();
      if (!currentUrl.includes('/dashboard') && !currentUrl.includes('/documents')) {
        throw new Error(`Not on dashboard after login. URL: ${currentUrl}`);
      }

      const bodyText = await page.evaluate(() => document.body?.innerText || '');
      if (bodyText.trim().length === 0) {
        throw new Error('Dashboard body is empty');
      }
    });

    // =========================================================================
    // Test 4: Document creation
    // =========================================================================
    await test('Document creation opens editor', async () => {
      await wait(5000);

      const errors = await navigateAndWait(page, `${BASE}/documents`);
      if (errors.length > 0) {
        throw new Error(`JS errors on /documents: ${errors.slice(0, 3).join('; ')}`);
      }

      await dismissOnboarding(page);

      const tokenBefore = await page.evaluate(() => {
        try { return localStorage.getItem('tachyon_token'); } catch { return null; }
      });
      if (!tokenBefore) {
        throw new Error('No auth token found before document creation');
      }

      await page.waitForSelector('button', { timeout: 10000 });
      await wait(5000);

      const newDocBtn = await page.evaluateHandle(() => {
        const buttons = Array.from(document.querySelectorAll('button'));
        return buttons.find(b => b.textContent.includes('New Document')) || null;
      });

      if (!newDocBtn || !(await newDocBtn.asElement())) {
        throw new Error('"+ New Document" button not found');
      }

      await newDocBtn.asElement().click();
      await wait(1000);

      const titleInput = await page.$('input[placeholder="Enter document title"]');
      if (!titleInput) {
        throw new Error('Title input in modal not found');
      }

      const docTitle = `Smoke Test Doc ${timestamp}`;
      await titleInput.type(docTitle, { delay: 30 });
      await wait(500);

      let createResponse = null;
      const responseHandler = async (resp) => {
        if (resp.url().includes('/documents') && !resp.url().includes('search') && resp.request().method() === 'POST') {
          try {
            createResponse = { status: resp.status(), body: (await resp.text()).slice(0, 300) };
          } catch(e) {}
        }
      };
      page.on('response', responseHandler);

      const createBtn = await page.evaluateHandle(() => {
        const buttons = Array.from(document.querySelectorAll('button'));
        return buttons.find(b => b.textContent.trim() === 'Create') || null;
      });

      if (!createBtn || !(await createBtn.asElement())) {
        throw new Error('"Create" button not found in modal');
      }

      await page.evaluate(() => {
        const btn = Array.from(document.querySelectorAll('button')).find(b => b.textContent.trim() === 'Create');
        if (btn) btn.click();
      });

      for (let i = 0; i < 20; i++) {
        await wait(1000);
        const currentUrl = page.url();
        if (currentUrl.includes('/edit')) break;
      }

      await wait(5000);

      const currentUrl = page.url();
      const editMatch = currentUrl.match(/\/documents\/([^/]+)\/edit/);
      if (!editMatch) {
        const bodyText = await page.evaluate(() => document.body?.innerText?.slice(0, 300) || '');
        log(`  Create API response: ${JSON.stringify(createResponse)}`);

        if (createResponse && createResponse.status === 429) {
          log('  Rate limited, clicking existing document card instead');

          await page.evaluate(() => {
            const card = document.querySelector('.bg-white.dark\\:bg-gray-800.rounded-lg.shadow');
            if (card) card.click();
          });
          await wait(5000);

          const retryUrl = page.url();
          const retryMatch = retryUrl.match(/\/documents\/([^/]+)\/edit/);
          if (retryMatch && retryMatch[1]) {
            createdDocId = retryMatch[1];
          } else {
            throw new Error(`Document creation and fallback both failed. URL: ${retryUrl}`);
          }
        } else {
          throw new Error(`Did not navigate to edit page. URL: ${currentUrl}. Body: ${bodyText}`);
        }
      }

      createdDocId = editMatch[1];

      page.off('response', responseHandler);

      const editor = await page.$('.native-editor');
      if (!editor) {
        throw new Error('Native editor (.native-editor) not found on edit page');
      }
    });

    // =========================================================================
    // Test 5: Native editor structure and rendering
    // =========================================================================
    await test('Native editor renders with correct structure', async () => {
      if (!createdDocId) {
        throw new Error('No document was created, skipping editor test');
      }

      await wait(1000);

      const editor = await page.$('.native-editor');
      if (!editor) {
        throw new Error('Native editor (.native-editor) not found on edit page');
      }

      // Verify editor has tabindex for keyboard focus
      const tabindex = await editor.evaluate(el => el.getAttribute('tabindex'));
      if (tabindex !== '0') {
        throw new Error(`Editor should have tabindex="0", got "${tabindex}"`);
      }

      // Verify editor does NOT have contenteditable="true" (custom key handling, not native)
      const ce = await editor.evaluate(el => el.getAttribute('contenteditable'));
      if (ce === 'true') {
        throw new Error(`Editor should NOT have contenteditable="true" (uses custom key handling)`);
      }
      // Leptos prop:contenteditable may not render as a DOM attribute — null is acceptable

      // Verify editor renders line content areas OR placeholder (empty docs show placeholder)
      const editorState = await page.evaluate(() => {
        const lineContents = document.querySelectorAll('.native-editor .line-content').length;
        const placeholder = document.querySelector('.editor-placeholder');
        return { lineContents, hasPlaceholder: !!placeholder };
      });

      if (editorState.lineContents === 0 && !editorState.hasPlaceholder) {
        throw new Error('Editor has no .line-content elements and no .editor-placeholder — content not rendered');
      }

      log(`  Editor structure OK: ${editorState.lineContents} lines, placeholder=${editorState.hasPlaceholder}`);

      // Verify cursor element exists
      const cursor = await page.$('.editor-cursor');
      if (!cursor) {
        throw new Error('Editor cursor (.editor-cursor) not found');
      }

      // Verify editor has scroll spacer
      const spacer = await page.$('.editor-scroll-spacer');
      if (!spacer) {
        throw new Error('Editor scroll spacer (.editor-scroll-spacer) not found');
      }

      log(`  Editor structure OK: cursor present, spacer present, lines=${editorState.lineContents}, placeholder=${editorState.hasPlaceholder}`);
    });

    // =========================================================================
    // Test 6: Markdown preview toggle
    // =========================================================================
    await test('Markdown preview toggle shows split view', async () => {
      if (!createdDocId) {
        throw new Error('No document was created, skipping preview test');
      }

      await wait(1000);

      const previewBtn = await page.evaluateHandle(() => {
        const buttons = Array.from(document.querySelectorAll('.editor-toolbar-btn, button'));
        return buttons.find(b => {
          const title = b.getAttribute('title') || '';
          const text = b.textContent || '';
          return title.toLowerCase().includes('preview') || text.includes('\u{1F441}');
        }) || null;
      });

      if (!previewBtn || !(await previewBtn.asElement())) {
        throw new Error('Preview button not found in toolbar');
      }

      await previewBtn.asElement().click();
      await wait(1500);

      const previewExists = await page.evaluate(() => {
        return !!document.querySelector('.markdown-preview, .prose, [class*="preview"]');
      });

      if (!previewExists) {
        const pageHtml = await page.evaluate(() => document.body.innerHTML.slice(0, 1000));
        throw new Error(`Markdown preview not found after toggle. Page snippet: ${pageHtml}`);
      }
    });

    // =========================================================================
    // Test 7: Document list
    // =========================================================================
    await test('Document list shows document cards', async () => {
      const errors = await navigateAndWait(page, `${BASE}/documents`);
      if (errors.length > 0) {
        throw new Error(`JS errors on /documents: ${errors.slice(0, 3).join('; ')}`);
      }

      await dismissOnboarding(page);
      await wait(2000);

      const cardCount = await page.evaluate(() => {
        const cards = document.querySelectorAll('.bg-white.dark\\:bg-gray-800.rounded-lg.shadow');
        return cards.length;
      });

      if (cardCount === 0) {
        const bodyText = await page.evaluate(() => document.body?.innerText?.slice(0, 500) || '');
        if (!bodyText.toLowerCase().includes('no documents') && !bodyText.toLowerCase().includes('empty')) {
          throw new Error(`No document cards found and no empty state shown. Body: ${bodyText}`);
        }
        log('  Document list shows empty state (expected if no documents)');
      }
    });

    // =========================================================================
    // Test 8: Client-side search (Ctrl+K)
    // =========================================================================
    await test('Client-side search opens with Ctrl+K', async () => {
      const errors = await navigateAndWait(page, `${BASE}/documents`);
      if (errors.length > 0) {
        throw new Error(`JS errors on /documents: ${errors.slice(0, 3).join('; ')}`);
      }

      await dismissOnboarding(page);
      await wait(1000);

      await page.keyboard.down('Control');
      await page.keyboard.press('k');
      await page.keyboard.up('Control');
      await wait(1000);

      const searchPanel = await page.evaluate(() => {
        const searchEl = document.querySelector(
          '[class*="search-overlay"], [class*="client-search"], [class*="command-palette"], ' +
          '[data-testid="search"], input[placeholder*="Search"], input[placeholder*="search"]'
        );
        return !!searchEl;
      });

      if (!searchPanel) {
        const bodyHtml = await page.evaluate(() => document.body.innerHTML.slice(0, 500));
        throw new Error(`Search panel not found after Ctrl+K. Snippet: ${bodyHtml}`);
      }
    });

    // =========================================================================
    // Test 9: Dashboard loads
    // =========================================================================
    await test('Dashboard loads with content', async () => {
      const errors = await navigateAndWait(page, `${BASE}/dashboard`);
      if (errors.length > 0) {
        throw new Error(`JS errors on /dashboard: ${errors.slice(0, 3).join('; ')}`);
      }

      await dismissOnboarding(page);
      await wait(2000);

      const bodyText = await page.evaluate(() => document.body?.innerText?.trim() || '');
      if (bodyText.length === 0) {
        throw new Error('Dashboard body is empty');
      }
    });

    // =========================================================================
    // Test 10: Graph page loads
    // =========================================================================
    await test('Graph page loads with visualization', async () => {
      const errors = await navigateAndWait(page, `${BASE}/graph`);
      if (errors.length > 0) {
        throw new Error(`JS errors on /graph: ${errors.slice(0, 3).join('; ')}`);
      }

      await dismissOnboarding(page);
      await wait(3000);

      const bodyText = await page.evaluate(() => document.body?.innerText?.trim() || '');
      if (bodyText.length === 0) {
        throw new Error('Graph page body is empty');
      }

      const hasGraphElement = await page.evaluate(() => {
        return !!(
          document.querySelector('svg') ||
          document.querySelector('canvas') ||
          document.querySelector('[class*="graph"]') ||
          document.querySelector('[class*="visualization"]')
        );
      });

      if (!hasGraphElement) {
        const hasGraphText = bodyText.toLowerCase().includes('graph') ||
          bodyText.toLowerCase().includes('knowledge') ||
          bodyText.toLowerCase().includes('node');
        if (!hasGraphText) {
          throw new Error('No graph visualization elements found');
        }
      }
    });

  } finally {
    await browser.close();

    console.log('\n=== RESULTS ===');
    for (const r of results) {
      console.log(`${r.status === 'PASS' ? 'PASS' : 'FAIL'} ${r.name}`);
      if (r.error) console.log(`       ${r.error}`);
    }
    const passed = results.filter(r => r.status === 'PASS').length;
    console.log(`\n${passed}/${results.length} tests passed`);

    if (passed < results.length) {
      process.exit(1);
    }
  }
})();
