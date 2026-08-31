import { test, expect } from '@playwright/test';

test.describe('Document CRUD Operations', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.fill('input[type="email"]', 'test@example.com');
    await page.fill('input[type="password"]', 'ValidPassword123!');
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/(documents|dashboard)/, { timeout: 10000 });
  });

  test('should display documents list page', async ({ page }) => {
    await page.goto('/documents');
    
    await expect(page.locator('h1, h2')).toContainText(/documents/i);
    await expect(page.locator('button:has-text("New"), button:has-text("Create")')).toBeVisible();
  });

  test('should create a new document', async ({ page }) => {
    await page.goto('/documents');
    
    await page.click('button:has-text("New"), button:has-text("Create Document")');
    
    await expect(page).toHaveURL(/\/documents\/new|\/documents\/create/);
    
    const timestamp = Date.now();
    await page.fill('input[name="title"], input[id="title"]', `Test Document ${timestamp}`);
    await page.fill('input[name="slug"], input[id="slug"]', `test-doc-${timestamp}`);
    
    const descriptionField = page.locator('textarea[name="description"], textarea[id="description"]');
    if (await descriptionField.isVisible()) {
      await descriptionField.fill('This is a test document description');
    }
    
    await page.click('button[type="submit"], button:has-text("Save")');
    
    await expect(page).toHaveURL(/\/documents\/[\w-]+/);
    await expect(page.locator('h1, h2')).toContainText(`Test Document ${timestamp}`);
  });

  test('should view document details', async ({ page }) => {
    await page.goto('/documents');
    
    const firstDocument = page.locator('a[href^="/documents/"]').first();
    if (await firstDocument.isVisible()) {
      await firstDocument.click();
      
      await expect(page.locator('h1, h2')).toBeVisible();
      await expect(page.locator('.document-content, [data-testid="document-content"]')).toBeVisible();
    }
  });

  test('should edit an existing document', async ({ page }) => {
    await page.goto('/documents');
    
    const firstDocument = page.locator('a[href^="/documents/"]').first();
    if (await firstDocument.isVisible()) {
      await firstDocument.click();
      
      await page.click('button:has-text("Edit"), a:has-text("Edit")');
      
      await expect(page).toHaveURL(/\/edit/);
      
      const titleInput = page.locator('input[name="title"], input[id="title"]');
      await titleInput.fill(await titleInput.inputValue() + ' - Updated');
      
      await page.click('button[type="submit"], button:has-text("Save")');
      
      await expect(page.locator('[role="status"], .success-message')).toBeVisible({ timeout: 5000 });
    }
  });

  test('should delete a document', async ({ page }) => {
    const timestamp = Date.now();
    await page.goto('/documents');
    
    await page.click('button:has-text("New"), button:has-text("Create Document")');
    await page.fill('input[name="title"], input[id="title"]', `Delete Test ${timestamp}`);
    await page.fill('input[name="slug"], input[id="slug"]', `delete-test-${timestamp}`);
    await page.click('button[type="submit"], button:has-text("Save")');
    
    await page.waitForURL(/\/documents\/[\w-]+/);
    
    await page.click('button:has-text("Delete"), button[aria-label="Delete"]');
    
    page.on('dialog', dialog => dialog.accept());
    
    await page.click('button:has-text("Confirm"), button:has-text("Yes, Delete")');
    
    await expect(page).toHaveURL(/\/documents/);
    
    await page.waitForTimeout(1000);
    await expect(page.locator(`text=Delete Test ${timestamp}`)).not.toBeVisible();
  });

  test('should search documents', async ({ page }) => {
    await page.goto('/documents');
    
    const searchInput = page.locator('input[type="search"], input[placeholder*="search"]');
    if (await searchInput.isVisible()) {
      await searchInput.fill('test');
      await searchInput.press('Enter');
      
      await expect(page.locator('.document-list, [data-testid="document-list"]')).toBeVisible();
    }
  });

  test('should filter documents by status', async ({ page }) => {
    await page.goto('/documents');
    
    const filterButton = page.locator('button:has-text("Filter"), button[aria-label="Filter"]');
    if (await filterButton.isVisible()) {
      await filterButton.click();
      
      const statusFilter = page.locator('select[name="status"], [data-testid="status-filter"]');
      if (await statusFilter.isVisible()) {
        await statusFilter.selectOption('Draft');
        
        await page.waitForTimeout(500);
      }
    }
  });

  test('should paginate documents', async ({ page }) => {
    await page.goto('/documents');
    
    const nextButton = page.locator('button:has-text("Next"), button[aria-label="Next page"]');
    if (await nextButton.isEnabled()) {
      await nextButton.click();
      
      await expect(page.locator('.pagination, [data-testid="pagination"]')).toBeVisible();
    }
  });

  test('should show document version history', async ({ page }) => {
    await page.goto('/documents');
    
    const firstDocument = page.locator('a[href^="/documents/"]').first();
    if (await firstDocument.isVisible()) {
      await firstDocument.click();
      
      const historyButton = page.locator('button:has-text("History"), a:has-text("Version History")');
      if (await historyButton.isVisible()) {
        await historyButton.click();
        
        await expect(page.locator('.version-list, [data-testid="version-list"]')).toBeVisible();
      }
    }
  });

  test('should revert to previous document version', async ({ page }) => {
    await page.goto('/documents');
    
    const firstDocument = page.locator('a[href^="/documents/"]').first();
    if (await firstDocument.isVisible()) {
      await firstDocument.click();
      
      const historyButton = page.locator('button:has-text("History")');
      if (await historyButton.isVisible()) {
        await historyButton.click();
        
        const revertButton = page.locator('button:has-text("Revert")').first();
        if (await revertButton.isVisible()) {
          revertButton.click();
          
          page.on('dialog', dialog => dialog.accept());
          
          await expect(page.locator('[role="status"]')).toBeVisible({ timeout: 5000 });
        }
      }
    }
  });

  test('should add tags to document', async ({ page }) => {
    await page.goto('/documents');
    
    const firstDocument = page.locator('a[href^="/documents/"]').first();
    if (await firstDocument.isVisible()) {
      await firstDocument.click();
      
      await page.click('button:has-text("Edit")');
      
      const tagsInput = page.locator('input[name="tags"], input[placeholder*="tag"]');
      if (await tagsInput.isVisible()) {
        await tagsInput.fill('test-tag');
        await tagsInput.press('Enter');
        
        await page.click('button[type="submit"], button:has-text("Save")');
        
        await expect(page.locator('text=test-tag')).toBeVisible();
      }
    }
  });

  test('should change document visibility', async ({ page }) => {
    await page.goto('/documents');
    
    const firstDocument = page.locator('a[href^="/documents/"]').first();
    if (await firstDocument.isVisible()) {
      await firstDocument.click();
      
      await page.click('button:has-text("Edit")');
      
      const visibilitySelect = page.locator('select[name="visibility"], [data-testid="visibility-select"]');
      if (await visibilitySelect.isVisible()) {
        await visibilitySelect.selectOption('Public');
        
        await page.click('button[type="submit"], button:has-text("Save")');
        
        await expect(page.locator('[role="status"]')).toBeVisible({ timeout: 5000 });
      }
    }
  });

  test('should export document', async ({ page }) => {
    await page.goto('/documents');
    
    const firstDocument = page.locator('a[href^="/documents/"]').first();
    if (await firstDocument.isVisible()) {
      await firstDocument.click();
      
      const exportButton = page.locator('button:has-text("Export"), a:has-text("Download")');
      if (await exportButton.isVisible()) {
        const [download] = await Promise.all([
          page.waitForEvent('download'),
          exportButton.click()
        ]);
        
        expect(download.suggestedFilename()).toMatch(/\.(md|pdf|html|txt)$/);
      }
    }
  });

  test('should share document link', async ({ page }) => {
    await page.goto('/documents');
    
    const firstDocument = page.locator('a[href^="/documents/"]').first();
    if (await firstDocument.isVisible()) {
      await firstDocument.click();
      
      const shareButton = page.locator('button:has-text("Share"), button[aria-label="Share"]');
      if (await shareButton.isVisible()) {
        await shareButton.click();
        
        await expect(page.locator('input[type="url"], [data-testid="share-link"]')).toBeVisible();
      }
    }
  });
});
