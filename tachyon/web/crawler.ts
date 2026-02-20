/**
 * Tachyon Web Crawler
 * 
 * Crawls all pages of the Tachyon web application and reports errors.
 * Uses Playwright for browser automation.
 * 
 * Usage: bun run crawler.ts
 */

import { chromium, Browser, Page, BrowserContext } from 'playwright';

// Configuration
const CONFIG = {
  baseUrl: process.env.BASE_URL || 'http://localhost:3000',
  apiUrl: process.env.API_URL || 'http://127.0.0.1:8080',
  timeout: 30000,
  slowMo: 0, // Slow down operations for debugging (ms)
  headless: true,
};

// Pages to crawl
const PAGES = [
  { path: '/', name: 'Home/Dashboard', requiresAuth: false },
  { path: '/documents', name: 'Documents List', requiresAuth: false },
  { path: '/documents/new', name: 'New Document', requiresAuth: true },
  { path: '/repositories', name: 'Repositories', requiresAuth: true },
  { path: '/settings', name: 'Settings', requiresAuth: true },
  { path: '/login', name: 'Login', requiresAuth: false },
  { path: '/nonexistent', name: '404 Page', requiresAuth: false },
];

// Error tracking
interface CrawledError {
  page: string;
  url: string;
  type: 'console' | 'pageerror' | 'requestfailed' | 'assertion';
  message: string;
  details?: unknown;
  timestamp: string;
  stack?: string;
}

interface CrawlResult {
  page: string;
  url: string;
  status: 'success' | 'error' | 'skipped';
  loadTime: number;
  errors: CrawledError[];
  warnings: string[];
  consoleLogs: string[];
}

const allErrors: CrawledError[] = [];
const results: CrawlResult[] = [];

/**
 * Main crawler function
 */
async function crawl(): Promise<void> {
  console.log('='.repeat(60));
  console.log('Tachyon Web Crawler');
  console.log('='.repeat(60));
  console.log(`Base URL: ${CONFIG.baseUrl}`);
  console.log(`API URL: ${CONFIG.apiUrl}`);
  console.log(`Timestamp: ${new Date().toISOString()}`);
  console.log('='.repeat(60));
  console.log();

  // Check if API is available
  console.log('[Setup] Checking API availability...');
  try {
    const response = await fetch(`${CONFIG.apiUrl}/health`);
    if (response.ok) {
      console.log('[Setup] API is available');
    } else {
      console.warn('[Setup] API returned non-OK status:', response.status);
    }
  } catch (error) {
    console.error('[Setup] API is not available:', error);
    console.warn('[Setup] Continuing anyway - some tests may fail');
  }
  console.log();

  let browser: Browser | null = null;

  try {
    // Launch browser
    console.log('[Setup] Launching browser...');
    browser = await chromium.launch({
      headless: CONFIG.headless,
      slowMo: CONFIG.slowMo,
    });

    const context = await browser.newContext({
      viewport: { width: 1280, height: 720 },
      userAgent: 'TachyonCrawler/1.0',
    });

    // Crawl each page
    for (const pageInfo of PAGES) {
      const result = await crawlPage(context, pageInfo);
      results.push(result);
      allErrors.push(...result.errors);
      
      // Brief pause between pages
      await sleep(500);
    }

    // Print summary
    printSummary();

  } catch (error) {
    console.error('[Crawler] Fatal error:', error);
    allErrors.push({
      page: 'Crawler',
      url: '',
      type: 'assertion',
      message: error instanceof Error ? error.message : String(error),
      timestamp: new Date().toISOString(),
    });
  } finally {
    if (browser) {
      await browser.close();
    }
  }
}

/**
 * Crawl a single page
 */
async function crawlPage(
  context: BrowserContext,
  pageInfo: { path: string; name: string; requiresAuth: boolean }
): Promise<CrawlResult> {
  const url = `${CONFIG.baseUrl}${pageInfo.path}`;
  const result: CrawlResult = {
    page: pageInfo.name,
    url,
    status: 'success',
    loadTime: 0,
    errors: [],
    warnings: [],
    consoleLogs: [],
  };

  console.log(`\n[Crawling] ${pageInfo.name}`);
  console.log(`  URL: ${url}`);

  const pageErrors: CrawledError[] = [];
  const pageWarnings: string[] = [];
  const consoleLogs: string[] = [];

  const page = await context.newPage();

  try {
    // Set up error handlers
    page.on('console', (msg) => {
      const text = msg.text();
      const type = msg.type();
      
      consoleLogs.push(`[${type}] ${text}`);
      
      // Only track errors and warnings
      if (type === 'error') {
        // Filter out known non-critical errors
        if (text.includes('favicon') || text.includes('404')) {
          pageWarnings.push(`Console warning: ${text}`);
          return;
        }
        
        pageErrors.push({
          page: pageInfo.name,
          url,
          type: 'console',
          message: text,
          timestamp: new Date().toISOString(),
        });
      } else if (type === 'warning') {
        pageWarnings.push(text);
      }
    });

    page.on('pageerror', (error) => {
      consoleLogs.push(`[pageerror] ${error.message}`);
      pageErrors.push({
        page: pageInfo.name,
        url,
        type: 'pageerror',
        message: error.message,
        stack: error.stack,
        timestamp: new Date().toISOString(),
      });
    });

    page.on('requestfailed', (request) => {
      const failure = request.failure();
      if (failure) {
        consoleLogs.push(`[requestfailed] ${request.url()} - ${failure.errorText}`);
        
        // Don't count favicon or optional resource failures as errors
        if (request.url().includes('favicon')) {
          pageWarnings.push(`Request failed: ${request.url()} - ${failure.errorText}`);
          return;
        }
        
        pageErrors.push({
          page: pageInfo.name,
          url,
          type: 'requestfailed',
          message: `${request.method()} ${request.url()}: ${failure.errorText}`,
          details: { url: request.url(), method: request.method() },
          timestamp: new Date().toISOString(),
        });
      }
    });

    // Navigate to page
    const startTime = Date.now();
    
    try {
      const response = await page.goto(url, {
        waitUntil: 'networkidle',
        timeout: CONFIG.timeout,
      });

      result.loadTime = Date.now() - startTime;

      if (!response) {
        result.status = 'error';
        pageErrors.push({
          page: pageInfo.name,
          url,
          type: 'assertion',
          message: 'No response received',
          timestamp: new Date().toISOString(),
        });
      } else if (response.status() >= 400) {
        // 404s are expected for some test pages
        if (pageInfo.path === '/nonexistent' && response.status() === 404) {
          result.status = 'success';
          console.log(`  ✓ 404 returned as expected`);
        } else {
          result.status = 'error';
          pageErrors.push({
            page: pageInfo.name,
            url,
            type: 'assertion',
            message: `HTTP ${response.status()}: ${response.statusText()}`,
            timestamp: new Date().toISOString(),
          });
        }
      }
    } catch (navError) {
      result.status = 'error';
      result.loadTime = Date.now() - startTime;
      
      const errorMessage = navError instanceof Error ? navError.message : String(navError);
      pageErrors.push({
        page: pageInfo.name,
        url,
        type: 'assertion',
        message: `Navigation failed: ${errorMessage}`,
        timestamp: new Date().toISOString(),
      });
    }

    // Wait for page to stabilize
    await sleep(1000);

    // Check for Tachyon app state
    const appState = await page.evaluate(() => {
      if (typeof window !== 'undefined' && (window as any).Tachyon) {
        return {
          initialized: true,
          isAuthenticated: (window as any).Tachyon.isAuthenticated,
          errorCount: (window as any).Tachyon.errors?.length || 0,
        };
      }
      return { initialized: false };
    });

    if (appState.initialized) {
      console.log(`  App initialized: ${appState.isAuthenticated ? 'authenticated' : 'not authenticated'}`);
      if (appState.errorCount > 0) {
        pageWarnings.push(`App has ${appState.errorCount} stored errors`);
      }
    }

    // Try interactive elements if page loaded successfully
    if (result.status === 'success') {
      await testInteractiveElements(page, pageInfo, pageErrors);
    }

    result.errors = pageErrors;
    result.warnings = pageWarnings;
    result.consoleLogs = consoleLogs;

    // Print page result
    if (pageErrors.length === 0) {
      console.log(`  ✓ No errors (${result.loadTime}ms)`);
    } else {
      console.log(`  ✗ ${pageErrors.length} error(s) (${result.loadTime}ms)`);
      pageErrors.forEach((err) => {
        console.log(`    - [${err.type}] ${err.message.substring(0, 100)}${err.message.length > 100 ? '...' : ''}`);
      });
    }

    if (pageWarnings.length > 0) {
      console.log(`  ⚠ ${pageWarnings.length} warning(s)`);
    }

  } catch (error) {
    result.status = 'error';
    pageErrors.push({
      page: pageInfo.name,
      url,
      type: 'assertion',
      message: error instanceof Error ? error.message : String(error),
      timestamp: new Date().toISOString(),
    });
    result.errors = pageErrors;
  } finally {
    await page.close();
  }

  return result;
}

/**
 * Test interactive elements on a page
 */
async function testInteractiveElements(
  page: Page,
  pageInfo: { path: string; name: string; requiresAuth: boolean },
  errors: CrawledError[]
): Promise<void> {
  try {
    // Test theme toggle if present
    const themeToggle = await page.$('#theme-toggle');
    if (themeToggle) {
      await themeToggle.click();
      await sleep(200);
      await themeToggle.click(); // Toggle back
      console.log(`  ✓ Theme toggle works`);
    }

    // Test search input if present
    const searchInput = await page.$('#search-input');
    if (searchInput) {
      await searchInput.fill('test query');
      await sleep(500); // Wait for debounce
      console.log(`  ✓ Search input works`);
    }

    // Test navigation links
    const navLinks = await page.$$('nav a');
    if (navLinks.length > 0) {
      console.log(`  ✓ Found ${navLinks.length} navigation link(s)`);
    }

  } catch (error) {
    errors.push({
      page: pageInfo.name,
      url: page.url(),
      type: 'assertion',
      message: `Interactive test failed: ${error instanceof Error ? error.message : String(error)}`,
      timestamp: new Date().toISOString(),
    });
  }
}

/**
 * Print final summary
 */
function printSummary(): void {
  console.log('\n' + '='.repeat(60));
  console.log('CRAWL SUMMARY');
  console.log('='.repeat(60));

  const successCount = results.filter((r) => r.status === 'success').length;
  const errorCount = results.filter((r) => r.status === 'error').length;
  const skippedCount = results.filter((r) => r.status === 'skipped').length;

  console.log(`\nPages crawled: ${results.length}`);
  console.log(`  ✓ Success: ${successCount}`);
  console.log(`  ✗ Errors:  ${errorCount}`);
  console.log(`  ⊘ Skipped: ${skippedCount}`);

  const totalErrors = allErrors.length;
  const totalWarnings = results.reduce((sum, r) => sum + r.warnings.length, 0);

  console.log(`\nTotal errors:   ${totalErrors}`);
  console.log(`Total warnings: ${totalWarnings}`);

  if (totalErrors > 0) {
    console.log('\n' + '-'.repeat(60));
    console.log('ERROR DETAILS');
    console.log('-'.repeat(60));
    
    // Group errors by type
    const errorsByType = new Map<string, CrawledError[]>();
    allErrors.forEach((err) => {
      const existing = errorsByType.get(err.type) || [];
      existing.push(err);
      errorsByType.set(err.type, existing);
    });

    errorsByType.forEach((errors, type) => {
      console.log(`\n[${type.toUpperCase()}] (${errors.length} errors)`);
      errors.forEach((err) => {
        console.log(`  Page: ${err.page}`);
        console.log(`  URL: ${err.url}`);
        console.log(`  Message: ${err.message}`);
        if (err.stack) {
          console.log(`  Stack: ${err.stack.split('\n').slice(0, 3).join('\n         ')}`);
        }
        console.log(`  Time: ${err.timestamp}`);
        console.log();
      });
    });
  }

  // Print all console output for debugging
  const allConsoleLogs = results.flatMap((r) => 
    r.consoleLogs.map((log) => `[${r.page}] ${log}`)
  );
  
  if (allConsoleLogs.length > 0) {
    console.log('\n' + '-'.repeat(60));
    console.log('CONSOLE OUTPUT (first 50 lines)');
    console.log('-'.repeat(60));
    allConsoleLogs.slice(0, 50).forEach((log) => console.log(log));
    if (allConsoleLogs.length > 50) {
      console.log(`... and ${allConsoleLogs.length - 50} more lines`);
    }
  }

  console.log('\n' + '='.repeat(60));
  if (totalErrors === 0) {
    console.log('✓ ALL TESTS PASSED');
  } else {
    console.log(`✗ ${totalErrors} ERROR(S) FOUND`);
    process.exit(1);
  }
  console.log('='.repeat(60));
}

/**
 * Utility: Sleep
 */
function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// Run crawler
crawl().catch((error) => {
  console.error('Crawler failed:', error);
  process.exit(1);
});
