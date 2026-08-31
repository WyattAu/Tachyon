const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({ viewport: { width: 1440, height: 900 } });
  const page = await context.newPage();

  // Enable console logging
  page.on('console', msg => console.log('BROWSER:', msg.text()));

  console.log('1. Navigate to login page');
  await page.goto('http://192.168.1.191:8080/login', { waitUntil: 'load', timeout: 30000 });
  await page.waitForTimeout(3000);

  console.log('2. Check current URL:', page.url());

  // Fill server URL
  const serverUrl = await page.$('#server-url');
  if (serverUrl) {
    await serverUrl.fill('http://192.168.1.191:8080');
    console.log('3. Filled server URL');
  }

  // Fill username
  const username = await page.$('#username');
  if (username) {
    await username.fill('admin');
    console.log('4. Filled username');
  }

  // Fill password
  const password = await page.$('#password');
  if (password) {
    await password.fill('admin123');
    console.log('5. Filled password');
  }

  // Click sign in
  const signInBtn = await page.$('button[type="submit"], button:has-text("Sign")');
  if (signInBtn) {
    await signInBtn.click();
    console.log('6. Clicked sign in');
  }

  // Wait for navigation
  await page.waitForTimeout(5000);

  console.log('7. Final URL:', page.url());

  // Check localStorage
  const token = await page.evaluate(() => localStorage.getItem('tachyon_token'));
  console.log('8. Token in localStorage:', token ? 'YES (length=' + token.length + ')' : 'NO');

  // Take screenshot
  await page.screenshot({ path: '/home/wyatt/dev/src/github.com/WyattAu/Tachyon/test-login-result.png' });
  console.log('9. Screenshot saved');

  await browser.close();
})();
