/**
 * Minimal Playwright test: verify token storage + navigation after login.
 *
 * Run:  npx playwright install chromium && node test_auth_flow.mjs
 */
import { chromium } from 'playwright';

const SERVER = process.env.TACHYON_SERVER || 'http://192.168.1.191:8080';

(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();

  page.on('console', msg => console.log(`  [browser] ${msg.type()}: ${msg.text()}`));

  console.log('1. Navigate to /login');
  await page.goto(`${SERVER}/login`, { waitUntil: 'networkidle' });
  console.log(`   URL: ${page.url()}`);

  // Verify form exists
  await page.waitForSelector('#username', { timeout: 10000 });
  console.log('   Form rendered ✓');

  console.log('2. Fill credentials');
  await page.fill('#username', 'admin');
  await page.fill('#password', 'admin123');

  // Before clicking, snapshot localStorage
  const preKeys = await page.evaluate(() => {
    const keys = {};
    for (let i = 0; i < localStorage.length; i++) {
      const k = localStorage.key(i);
      keys[k] = localStorage.getItem(k)?.substring(0, 40) + '...';
    }
    return keys;
  });
  console.log('   Pre-submit localStorage:', JSON.stringify(preKeys));

  console.log('3. Click Submit');
  // Listen for navigation
  const navPromise = page.waitForURL('**', { timeout: 15000 }).catch(() => null);
  await page.click('button[type="submit"]');

  // Wait a bit for async login + navigation
  await page.waitForTimeout(5000);
  await navPromise;

  console.log(`4. Final URL: ${page.url()}`);

  // Check localStorage after login
  const postKeys = await page.evaluate(() => {
    const keys = {};
    for (let i = 0; i < localStorage.length; i++) {
      const k = localStorage.key(i);
      keys[k] = localStorage.getItem(k)?.substring(0, 80) + '...';
    }
    return keys;
  });
  console.log('   Post-submit localStorage:', JSON.stringify(postKeys, null, 2));

  const token = await page.evaluate(() => localStorage.getItem('tachyon_token'));
  console.log(`   tachyon_token: ${token ? 'EXISTS (len=' + token.length + ')' : 'MISSING'}`);

  // Check if error message is visible
  const errorVisible = await page.isVisible('#login-error');
  if (errorVisible) {
    const errorText = await page.textContent('#login-error');
    console.log(`   Login error displayed: "${errorText}"`);
  }

  // Check if still on login page with loading spinner
  const loading = await page.evaluate(() => {
    const btn = document.querySelector('button[type="submit"]');
    return btn?.disabled;
  });
  console.log(`   Submit button disabled (loading): ${loading}`);

  // Verdict
  console.log('\n=== VERDICT ===');
  const finalUrl = page.url();
  if (finalUrl.includes('/dashboard') && token) {
    console.log('✅ PASS: Login succeeded, on dashboard with token');
  } else if (finalUrl.includes('/login?return=')) {
    console.log('❌ FAIL: Redirected back to login — AuthGuard rejected');
    console.log('   Root cause: token NOT in localStorage when AuthGuard ran');
    if (!token) console.log('   Token is MISSING from localStorage after submit');
  } else if (finalUrl.includes('/login') && !token && errorVisible) {
    console.log('❌ FAIL: Login API returned an error');
  } else if (finalUrl.includes('/login') && !token && loading) {
    console.log('⚠️  STILL LOADING: Submit button disabled, no token, no error');
    console.log('   Possible: request hanging or response not parsed');
  } else if (finalUrl.includes('/login') && !token) {
    console.log('❌ FAIL: On login page, no token, no error visible');
    console.log('   Possible: form submission prevented by HTML validation');
  } else {
    console.log(`⚠️  Unexpected state: URL=${finalUrl}, token=${!!token}`);
  }

  await page.screenshot({ path: '/tmp/tachyon_auth_final.png', fullPage: true });
  console.log('\nScreenshot: /tmp/tachyon_auth_final.png');
  await browser.close();
})();
