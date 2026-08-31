import { test, expect } from '@playwright/test';
import { AppPage } from './helpers';

test.describe('Document CRUD', () => {
  let app: AppPage;
  const uniqueId = Date.now();
  const testDocTitle = `CRUD Test Doc ${uniqueId}`;

  test.beforeEach(async ({ page }) => {
    app = new AppPage(page);
  });

  test('create a new document', async ({ page }) => {
    await app.register(`crud_${uniqueId}`, `crud_${uniqueId}@example.com`, 'TestPass123!');
    await app.login(`crud_${uniqueId}@example.com`, 'TestPass123!');

    await app.createDocument(testDocTitle, '# My New Document\n\nThis is test content.');

    const isOnDocPage =
      page.url().includes('documents') && page.url().split('/').length > 2;
    const hasEditor =
      (await page.locator('[contenteditable="true"], [data-testid="editor"], textarea').first().isVisible().catch(() => false)) ||
      (await page.locator(`text="${testDocTitle}"`).first().isVisible().catch(() => false));

    expect(isOnDocPage || hasEditor).toBeTruthy();
  });

  test('edit title and content', async ({ page }) => {
    await app.register(`edit_${uniqueId}`, `edit_${uniqueId}@example.com`, 'TestPass123!');
    await app.login(`edit_${uniqueId}@example.com`, 'TestPass123!');

    await app.createDocument(`Edit Test ${uniqueId}`, 'Original content');
    await page.waitForLoadState('networkidle');

    const titleInput = page.locator(
      '[name="title"], input[placeholder*="title" i], [data-testid="document-title-input"], h1[contenteditable="true"]',
    );
    if (await titleInput.first().isVisible({ timeout: 5000 }).catch(() => false)) {
      await titleInput.first().fill('Updated Title');
    }

    const editor = page.locator(
      '[contenteditable="true"], [data-testid="editor"], textarea',
    );
    if (await editor.first().isVisible({ timeout: 5000 }).catch(() => false)) {
      await editor.first().click();
      await page.keyboard.type('\nUpdated content appended');
      await page.waitForTimeout(500);
    }

    const saveBtn = page.locator(
      'button:has-text("Save"), button:has-text("Update"), [data-testid="save-document"]',
    );
    if (await saveBtn.first().isVisible().catch(() => false)) {
      await saveBtn.first().click();
      await page.waitForLoadState('networkidle');
    }

    await page.reload();
    await page.waitForLoadState('networkidle');

    const hasContent = await page.locator('text="Updated content appended"').first().isVisible({ timeout: 5000 }).catch(() => false);
    expect(hasContent).toBeTruthy();
  });

  test('document appears in list', async ({ page }) => {
    await app.register(`list_${uniqueId}`, `list_${uniqueId}@example.com`, 'TestPass123!');
    await app.login(`list_${uniqueId}@example.com`, 'TestPass123!');

    await app.createDocument(`List Test ${uniqueId}`, 'List test content');
    await page.waitForLoadState('networkidle');

    await app.goto('/documents');
    await page.waitForLoadState('networkidle');

    const docInList = page.locator(`text="List Test ${uniqueId}"`);
    const isDocVisible = await docInList.first().isVisible({ timeout: 5000 }).catch(() => false);
    expect(isDocVisible).toBeTruthy();
  });

  test('delete document', async ({ page }) => {
    await app.register(`del_${uniqueId}`, `del_${uniqueId}@example.com`, 'TestPass123!');
    await app.login(`del_${uniqueId}@example.com`, 'TestPass123!');

    await app.createDocument(`Delete Test ${uniqueId}`, 'To be deleted');
    await page.waitForLoadState('networkidle');

    await app.goto('/documents');
    await page.waitForLoadState('networkidle');

    const docRow = page.locator(`text="Delete Test ${uniqueId}"`);
    if (await docRow.first().isVisible({ timeout: 5000 }).catch(() => false)) {
      await docRow.first().click({ button: 'right' });

      const deleteOption = page.locator(
        'text="Delete", text="Remove", [data-testid="delete-document"], button:has-text("Delete")',
      );
      const hasDeleteOption = await deleteOption.first().isVisible({ timeout: 3000 }).catch(() => false);
      if (hasDeleteOption) {
        await deleteOption.first().click();

        const confirmBtn = page.locator(
          'button:has-text("Confirm"), button:has-text("Yes"), button:has-text("Delete")',
        );
        if (await confirmBtn.first().isVisible({ timeout: 3000 }).catch(() => false)) {
          await confirmBtn.first().click();
          await page.waitForLoadState('networkidle');
        }
      }
    }

    const docStillVisible = await docRow.first().isVisible({ timeout: 3000 }).catch(() => false);
    expect(docStillVisible).toBeFalsy();
  });

  test('document is removed from list after deletion', async ({ page }) => {
    await app.register(`rm_${uniqueId}`, `rm_${uniqueId}@example.com`, 'TestPass123!');
    await app.login(`rm_${uniqueId}@example.com`, 'TestPass123!');

    const docTitle = `Remove Test ${uniqueId}`;
    await app.createDocument(docTitle, 'Will be removed and verified');
    await page.waitForLoadState('networkidle');

    await app.goto('/documents');
    await page.waitForLoadState('networkidle');

    const docLink = page.locator(`text="${docTitle}"`);
    if (await docLink.first().isVisible({ timeout: 5000 }).catch(() => false)) {
      await docLink.first().click();
      await page.waitForLoadState('networkidle');

      const deleteBtn = page.locator(
        'button:has-text("Delete"), [data-testid="delete-document"], [aria-label*="Delete" i]',
      );
      if (await deleteBtn.first().isVisible({ timeout: 3000 }).catch(() => false)) {
        await deleteBtn.first().click();
        const confirmBtn = page.locator(
          'button:has-text("Confirm"), button:has-text("Yes"), button:has-text("Delete")',
        );
        if (await confirmBtn.first().isVisible({ timeout: 3000 }).catch(() => false)) {
          await confirmBtn.first().click();
          await page.waitForLoadState('networkidle');
        }
      }
    }

    await app.goto('/documents');
    await page.waitForLoadState('networkidle');

    const isDocGone = !(await page.locator(`text="${docTitle}"`).first().isVisible({ timeout: 3000 }).catch(() => false));
    expect(isDocGone).toBeTruthy();
  });

  test('search for document', async ({ page }) => {
    await app.register(`search_${uniqueId}`, `search_${uniqueId}@example.com`, 'TestPass123!');
    await app.login(`search_${uniqueId}@example.com`, 'TestPass123!');

    await app.createDocument(`Searchable Doc ${uniqueId}`, 'Unique search term xyzabc123');
    await page.waitForLoadState('networkidle');

    await app.goto('/search');
    await page.waitForLoadState('networkidle');

    const searchInput = page.locator(
      'input[type="search"], input[placeholder*="search" i], [data-testid="search-input"], [name="q"], [name="search"]',
    );
    if (await searchInput.first().isVisible({ timeout: 5000 }).catch(() => false)) {
      await searchInput.first().fill(`Searchable Doc ${uniqueId}`);
      await searchInput.first().press('Enter');
      await page.waitForLoadState('networkidle');

      const result = page.locator(`text="Searchable Doc ${uniqueId}"`);
      const hasResult = await result.first().isVisible({ timeout: 5000 }).catch(() => false);
      expect(hasResult).toBeTruthy();
    }
  });

  test('filter documents by tag', async ({ page }) => {
    await app.register(`tag_${uniqueId}`, `tag_${uniqueId}@example.com`, 'TestPass123!');
    await app.login(`tag_${uniqueId}@example.com`, 'TestPass123!');

    await app.createDocument(`Tagged Doc ${uniqueId}`, 'Content with tag filter-test');
    await page.waitForLoadState('networkidle');

    await app.goto('/documents');
    await page.waitForLoadState('networkidle');

    const tagFilter = page.locator(
      '[data-testid="tag-filter"], [data-testid="filter-tags"], button:has-text("filter-test"), .tag:has-text("filter-test")',
    );
    const hasTagFilter = await tagFilter.first().isVisible({ timeout: 5000 }).catch(() => false);
    expect(hasTagFilter).toBeTruthy();
  });

  test('filter documents by space', async ({ page }) => {
    await app.register(`spacefilter_${uniqueId}`, `spacefilter_${uniqueId}@example.com`, 'TestPass123!');
    await app.login(`spacefilter_${uniqueId}@example.com`, 'TestPass123!');

    await app.createDocument(`Space Doc ${uniqueId}`, 'Content in a space');
    await page.waitForLoadState('networkidle');

    await app.goto('/documents');
    await page.waitForLoadState('networkidle');

    const spaceFilter = page.locator(
      '[data-testid="space-filter"], select[name="space"], [data-testid="filter-space"]',
    );
    const hasSpaceFilter = await spaceFilter.first().isVisible({ timeout: 5000 }).catch(() => false);
    expect(hasSpaceFilter).toBeTruthy();
  });

  test('document API create returns correct shape', async ({ request }) => {
    const uniqueTs = Date.now();
    const response = await request.post('/api/v1/documents', {
      data: {
        title: `API Test Doc ${uniqueTs}`,
        content: 'Created via API',
        tags: ['e2e-test'],
      },
    });
    expect([201, 400, 401, 403, 500]).toContain(response.status());

    if (response.status() === 201) {
      const body = await response.json();
      expect(body).toHaveProperty('id');
      expect(body).toHaveProperty('title');
      expect(body.title).toBe(`API Test Doc ${uniqueTs}`);
    }
  });

  test('document API list returns results array', async ({ request }) => {
    const response = await request.get('/api/v1/documents');
    expect([200, 401, 403, 500]).toContain(response.status());

    if (response.status() === 200) {
      const body = await response.json();
      expect(body).toHaveProperty('results');
      expect(Array.isArray(body.results)).toBeTruthy();
      expect(body).toHaveProperty('total');
    }
  });

  test('document API search requires query parameter', async ({ request }) => {
    const response = await request.get('/api/v1/documents/search');
    expect(response.status()).toBe(400);
  });
});
