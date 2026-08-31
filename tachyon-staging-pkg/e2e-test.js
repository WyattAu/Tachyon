const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

const BASE_URL = process.env.TACHYON_URL || 'http://192.168.1.191:8080';
const RESULTS_DIR = path.join(__dirname, 'e2e-results');
const SCREENSHOT_DIR = path.join(RESULTS_DIR, 'screenshots');
const REPORT_FILE = path.join(RESULTS_DIR, 'e2e-report.json');
const TIMEOUT = 30000;

const TEST_USER = {
  username: `testuser_${Date.now()}`,
  email: `test_${Date.now()}@example.com`,
  password: 'TestPass123!',
};

const TEST_DOC = {
  title: `Test Document ${Date.now()}`,
  content: `# Test Document\n\nThis is a test document with unique content: ${Date.now()}.\n\n## Features\n- Markdown support\n- Wiki-links: [[${Date.now()}_linked]]\n- CRUD operations`,
};

let results = [];
let page;
let context;
let browser;

function log(test, status, msg = '') {
  const icon = status === 'pass' ? '✓' : status === 'fail' ? '✗' : '○';
  console.log(`  ${icon} ${test}${msg ? ' - ' + msg : ''}`);
}

async function screenshot(name) {
  const filePath = path.join(SCREENSHOT_DIR, `${name}.png`);
  await page.screenshot({ path: filePath, fullPage: true });
  return filePath;
}

async function record(test, fn) {
  const start = Date.now();
  try {
    const result = await fn();
    const duration = Date.now() - start;
    results.push({ test, status: 'pass', duration, details: result });
    log(test, 'pass', `${duration}ms`);
    return result;
  } catch (e) {
    const duration = Date.now() - start;
    results.push({ test, status: 'fail', duration, error: e.message });
    log(test, 'fail', e.message.substring(0, 100));
    await screenshot(`fail-${test.replace(/\s+/g, '-')}`).catch(() => {});
    return null;
  }
}

async function waitForNav(urlPart, timeout = TIMEOUT) {
  try {
    await page.waitForURL(`**${urlPart}**`, { timeout });
    return true;
  } catch {
    return false;
  }
}

async function testRegistration() {
  log('Registration', 'running');

  await page.goto(`${BASE_URL}/register`, { waitUntil: 'load', timeout: TIMEOUT });
  await page.waitForTimeout(2000);

  // Find and fill registration form
  const usernameInput = await page.$('#username, input[name="username"]');
  const emailInput = await page.$('#email, input[name="email"]');
  const passwordInput = await page.$('#password, input[name="password"]');
  const confirmInput = await page.$('#confirm-password, input[name="confirm_password"], input[name="confirmPassword"]');

  if (!usernameInput) {
    throw new Error('Registration form not found');
  }

  await usernameInput.fill(TEST_USER.username);
  if (emailInput) await emailInput.fill(TEST_USER.email);
  if (passwordInput) await passwordInput.fill(TEST_USER.password);
  if (confirmInput) await confirmInput.fill(TEST_USER.password);

  // Accept terms if present
  const termsCheckbox = await page.$('#terms, input[name="terms"], input[type="checkbox"]');
  if (termsCheckbox) await termsCheckbox.check();

  await screenshot('registration-filled');

  // Submit
  const submitBtn = await page.$('button[type="submit"], button:has-text("Register"), button:has-text("Sign"), button:has-text("Create")');
  if (submitBtn) {
    await submitBtn.click();
  } else {
    await (confirmInput || passwordInput).press('Enter');
  }

  await page.waitForTimeout(3000);
  await screenshot('registration-submitted');

  const url = page.url();
  const redirected = url.includes('/dashboard') || url.includes('/onboarding') || url.includes('/login');
  return { redirected, url };
}

async function testLogin() {
  log('Login', 'running');

  await page.goto(`${BASE_URL}/login`, { waitUntil: 'load', timeout: TIMEOUT });
  await page.waitForTimeout(2000);

  const usernameInput = await page.$('#username, input[name="username"]');
  const passwordInput = await page.$('#password, input[name="password"]');

  if (!usernameInput || !passwordInput) {
    // Try API login fallback
    const response = await page.evaluate(async (baseUrl) => {
      const res = await fetch(`${baseUrl}/api/v1/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username: 'admin', password: 'admin123' }),
      });
      return res.json();
    }, BASE_URL);

    if (response.access_token) {
      await page.evaluate((token) => {
        localStorage.setItem('auth_token', token);
        localStorage.setItem('access_token', token);
      }, response.access_token);
      await page.goto(`${BASE_URL}/dashboard`, { waitUntil: 'load', timeout: TIMEOUT });
      await page.waitForTimeout(2000);
    } else {
      throw new Error('Login form not found and API login failed');
    }
  } else {
    await usernameInput.fill('admin');
    await passwordInput.fill('admin123');

    const submitBtn = await page.$('button[type="submit"], button:has-text("Sign"), button:has-text("Login"), button:has-text("Log")');
    if (submitBtn) {
      await submitBtn.click();
    } else {
      await passwordInput.press('Enter');
    }

    await page.waitForTimeout(3000);
  }

  await screenshot('login-complete');

  const token = await page.evaluate(() => localStorage.getItem('auth_token') || localStorage.getItem('access_token'));
  const url = page.url();
  const hasToken = !!token;
  const onDashboard = url.includes('/dashboard') || url.includes('/onboarding');

  return { hasToken, onDashboard, url };
}

async function testDocumentCRUD() {
  log('Document CRUD', 'running');

  await page.goto(`${BASE_URL}/documents`, { waitUntil: 'load', timeout: TIMEOUT });
  await page.waitForTimeout(2000);
  await screenshot('documents-list');

  // Click "New Document"
  const newBtn = await page.$('button:has-text("New"), button:has-text("Create"), a:has-text("New"), a:has-text("Create")');
  if (newBtn) {
    await newBtn.click();
    await page.waitForTimeout(2000);
  } else {
    // Navigate directly
    await page.goto(`${BASE_URL}/documents/new`, { waitUntil: 'load', timeout: TIMEOUT });
    await page.waitForTimeout(2000);
  }

  await screenshot('document-new');

  // Fill title
  const titleInput = await page.$('#title, input[name="title"], input[placeholder*="title" i], input[placeholder*="Title"]');
  if (titleInput) {
    await titleInput.fill(TEST_DOC.title);
  }

  // Fill content
  const contentInput = await page.$('#content, textarea[name="content"], textarea, .ProseMirror, [contenteditable="true"]');
  if (contentInput) {
    await contentInput.fill(TEST_DOC.content);
  }

  await screenshot('document-filled');

  // Save
  const saveBtn = await page.$('button:has-text("Save"), button:has-text("Create"), button[type="submit"]');
  if (saveBtn) {
    await saveBtn.click();
    await page.waitForTimeout(3000);
  }

  await screenshot('document-saved');

  // Go back to list
  await page.goto(`${BASE_URL}/documents`, { waitUntil: 'load', timeout: TIMEOUT });
  await page.waitForTimeout(2000);

  // Verify document appears
  const docVisible = await page.evaluate((title) => {
    return document.body.innerText.includes(title);
  }, TEST_DOC.title);

  // Click to view
  const docLink = await page.$(`a:has-text("${TEST_DOC.title}"), tr:has-text("${TEST_DOC.title}") a`);
  if (docLink) {
    await docLink.click();
    await page.waitForTimeout(2000);
    await screenshot('document-view');
  }

  // Edit
  const editBtn = await page.$('button:has-text("Edit"), a:has-text("Edit")');
  if (editBtn) {
    await editBtn.click();
    await page.waitForTimeout(2000);

    const editContent = await page.$('#content, textarea[name="content"], textarea, .ProseMirror, [contenteditable="true"]');
    if (editContent) {
      await editContent.fill(TEST_DOC.content + '\n\n## Edited Section\nUpdated content.');
    }

    const saveBtn2 = await page.$('button:has-text("Save"), button[type="submit"]');
    if (saveBtn2) {
      await saveBtn2.click();
      await page.waitForTimeout(2000);
      await screenshot('document-edited');
    }
  }

  // Delete
  const deleteBtn = await page.$('button:has-text("Delete"), button:has-text("Remove")');
  if (deleteBtn) {
    await deleteBtn.click();
    await page.waitForTimeout(1000);

    // Confirm deletion
    const confirmBtn = await page.$('button:has-text("Confirm"), button:has-text("Yes"), button:has-text("OK")');
    if (confirmBtn) {
      await confirmBtn.click();
      await page.waitForTimeout(2000);
    }
  }

  await screenshot('document-deleted');

  // Verify removed from list
  await page.goto(`${BASE_URL}/documents`, { waitUntil: 'load', timeout: TIMEOUT });
  await page.waitForTimeout(2000);
  const docRemoved = !(await page.evaluate((title) => {
    return document.body.innerText.includes(title);
  }, TEST_DOC.title));

  return { docVisible, docRemoved };
}

async function testSearch() {
  log('Search', 'running');

  const uniqueContent = `Searchable_${Date.now()}`;

  // Create a document with unique content
  await page.goto(`${BASE_URL}/documents/new`, { waitUntil: 'load', timeout: TIMEOUT });
  await page.waitForTimeout(2000);

  const titleInput = await page.$('#title, input[name="title"]');
  const contentInput = await page.$('#content, textarea[name="content"], textarea, .ProseMirror, [contenteditable="true"]');

  if (titleInput) await titleInput.fill(`Search Test ${Date.now()}`);
  if (contentInput) await contentInput.fill(`# Search Test\n\nUnique content: ${uniqueContent}`);

  const saveBtn = await page.$('button:has-text("Save"), button:has-text("Create"), button[type="submit"]');
  if (saveBtn) {
    await saveBtn.click();
    await page.waitForTimeout(3000);
  }

  // Navigate to search
  await page.goto(`${BASE_URL}/search`, { waitUntil: 'load', timeout: TIMEOUT });
  await page.waitForTimeout(2000);

  // Search
  const searchInput = await page.$('#search, input[name="search"], input[type="search"], input[placeholder*="search" i]');
  if (searchInput) {
    await searchInput.fill(uniqueContent);
    await page.waitForTimeout(2000);
    await screenshot('search-results');
  }

  const hasResults = await page.evaluate((content) => {
    return document.body.innerText.includes(content);
  }, uniqueContent);

  return { hasResults, uniqueContent };
}

async function testWikiLinks() {
  log('Wiki-links', 'running');

  const docA = `WikiDocA_${Date.now()}`;
  const docB = `WikiDocB_${Date.now()}`;

  // Create doc B first (the target)
  await page.goto(`${BASE_URL}/documents/new`, { waitUntil: 'load', timeout: TIMEOUT });
  await page.waitForTimeout(2000);
  let titleInput = await page.$('#title, input[name="title"]');
  let contentInput = await page.$('#content, textarea[name="content"], textarea, .ProseMirror, [contenteditable="true"]');
  if (titleInput) await titleInput.fill(docB);
  if (contentInput) await contentInput.fill(`# ${docB}\n\nThis is the target document.`);
  let saveBtn = await page.$('button:has-text("Save"), button:has-text("Create"), button[type="submit"]');
  if (saveBtn) { await saveBtn.click(); await page.waitForTimeout(3000); }

  // Create doc A with wiki-link to B
  await page.goto(`${BASE_URL}/documents/new`, { waitUntil: 'load', timeout: TIMEOUT });
  await page.waitForTimeout(2000);
  titleInput = await page.$('#title, input[name="title"]');
  contentInput = await page.$('#content, textarea[name="content"], textarea, .ProseMirror, [contenteditable="true"]');
  if (titleInput) await titleInput.fill(docA);
  if (contentInput) await contentInput.fill(`# ${docA}\n\nLink to [[${docB}]] here.`);
  saveBtn = await page.$('button:has-text("Save"), button:has-text("Create"), button[type="submit"]');
  if (saveBtn) { await saveBtn.click(); await page.waitForTimeout(3000); }

  await screenshot('wikilink-doc');

  // Find and click the wiki-link
  const wikiLink = await page.$(`a:has-text("${docB}"), [data-wiki-link]:has-text("${docB}")`);
  if (wikiLink) {
    await wikiLink.click();
    await page.waitForTimeout(3000);
    await screenshot('wikilink-navigated');
  }

  const url = page.url();
  const navigated = url.includes(encodeURIComponent(docB)) || url.includes(docB.toLowerCase());

  return { navigated, url, docA, docB };
}

async function testDailyNotes() {
  log('Daily Notes', 'running');

  await page.goto(`${BASE_URL}/daily`, { waitUntil: 'load', timeout: TIMEOUT });
  await page.waitForTimeout(3000);

  await screenshot('daily-notes');

  // Check if today's note is present
  const hasTodayNote = await page.evaluate(() => {
    const body = document.body.innerText;
    const today = new Date().toISOString().split('T')[0];
    return body.includes(today) || body.includes('Today') || body.includes('today');
  });

  // Add content
  const contentInput = await page.$('#content, textarea[name="content"], textarea, .ProseMirror, [contenteditable="true"]');
  if (contentInput) {
    await contentInput.fill(`# Daily Note\n\nTest entry for ${new Date().toISOString()}`);
    const saveBtn = await page.$('button:has-text("Save"), button[type="submit"]');
    if (saveBtn) {
      await saveBtn.click();
      await page.waitForTimeout(2000);
    }
  }

  // Navigate to previous day
  const prevBtn = await page.$('button:has-text("Prev"), button:has-text("←"), button:has-text("<"), a:has-text("Prev"), [aria-label*="prev" i]');
  if (prevBtn) {
    await prevBtn.click();
    await page.waitForTimeout(2000);
    await screenshot('daily-notes-prev');
  }

  // Navigate back to today
  const todayBtn = await page.$('button:has-text("Today"), button:has-text("Current"), a:has-text("Today")');
  if (todayBtn) {
    await todayBtn.click();
    await page.waitForTimeout(2000);
    await screenshot('daily-notes-today');
  }

  return { hasTodayNote };
}

async function testGraphView() {
  log('Graph View', 'running');

  await page.goto(`${BASE_URL}/graph`, { waitUntil: 'load', timeout: TIMEOUT });
  await page.waitForTimeout(3000);

  await screenshot('graph-view');

  // Check for graph elements
  const graphInfo = await page.evaluate(() => {
    const canvas = document.querySelector('canvas');
    const svg = document.querySelector('svg');
    const nodes = document.querySelectorAll('[data-node], .node, circle, rect');
    return {
      hasCanvas: !!canvas,
      hasSvg: !!svg,
      nodeCount: nodes.length,
      hasGraph: !!(canvas || svg || nodes.length > 0),
    };
  });

  // Click a node if available
  if (graphInfo.nodeCount > 0) {
    const node = await page.$('[data-node], .node, circle, rect');
    if (node) {
      await node.click();
      await page.waitForTimeout(2000);
      await screenshot('graph-node-clicked');
    }
  }

  return graphInfo;
}

async function testImportWizard() {
  log('Import Wizard', 'running');

  await page.goto(`${BASE_URL}/import`, { waitUntil: 'load', timeout: TIMEOUT });
  await page.waitForTimeout(2000);

  await screenshot('import-page');

  // Look for source selection
  const obsidianOption = await page.$('button:has-text("Obsidian"), label:has-text("Obsidian"), div:has-text("Obsidian"), [data-source="obsidian"]');
  if (obsidianOption) {
    await obsidianOption.click();
    await page.waitForTimeout(2000);
    await screenshot('import-obsidian-selected');
  }

  // Check for file upload zone
  const hasUploadZone = await page.evaluate(() => {
    return !!(document.querySelector('input[type="file"]') ||
      document.querySelector('[data-upload]') ||
      document.querySelector('.upload-zone') ||
      document.querySelector('div:has-text("Drop")'));
  });

  return { hasUploadZone };
}

async function main() {
  fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });

  console.log('=== Tachyon E2E Test Suite ===');
  console.log(`Target: ${BASE_URL}`);
  console.log('');

  browser = await chromium.launch({
    headless: true,
    args: ['--no-sandbox', '--disable-setuid-sandbox'],
  });

  context = await browser.newContext({
    viewport: { width: 1440, height: 900 },
    userAgent: 'Tachyon-E2E/1.0',
  });

  page = await context.newPage();

  // Run all tests
  const tests = [
    testRegistration,
    testLogin,
    testDocumentCRUD,
    testSearch,
    testWikiLinks,
    testDailyNotes,
    testGraphView,
    testImportWizard,
  ];

  for (const test of tests) {
    try {
      await test();
    } catch (e) {
      console.log(`  ✗ ${test.name} - Fatal: ${e.message.substring(0, 100)}`);
      results.push({ test: test.name, status: 'fail', error: e.message });
    }
    console.log('');
  }

  // Generate report
  const passed = results.filter(r => r.status === 'pass').length;
  const failed = results.filter(r => r.status === 'fail').length;

  const report = {
    timestamp: new Date().toISOString(),
    baseUrl: BASE_URL,
    summary: { total: results.length, passed, failed },
    results,
  };

  fs.writeFileSync(REPORT_FILE, JSON.stringify(report, null, 2));

  console.log('=== E2E SUMMARY ===');
  console.log(`Passed: ${passed}/${results.length}`);
  console.log(`Failed: ${failed}/${results.length}`);
  console.log(`Report: ${REPORT_FILE}`);
  console.log(`Screenshots: ${SCREENSHOT_DIR}`);

  await browser.close();

  process.exit(failed > 0 ? 1 : 0);
}

main().catch(e => {
  console.error('Fatal error:', e);
  process.exit(1);
});
