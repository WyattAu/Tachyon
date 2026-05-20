import { Page, expect } from '@playwright/test';

export class AppPage {
  constructor(private page: Page) {}

  async goto(path = '/') {
    await this.page.goto(path);
    await this.page.waitForLoadState('domcontentloaded');
  }

  async register(username: string, email: string, password: string) {
    await this.goto('/register');
    await this.page.fill('[name="username"]', username);
    await this.page.fill('[name="email"]', email);
    await this.page.fill('[name="password"]', password);
    await this.page.click('button[type="submit"]');
    await this.page.waitForURL('**/login');
  }

  async login(email: string, password: string) {
    await this.goto('/login');
    await this.page.fill('[name="email"]', email);
    await this.page.fill('[name="password"]', password);
    await this.page.click('button[type="submit"]');
    await this.page.waitForURL('**/');
  }

  async logout() {
    const logoutBtn = this.page.locator('[data-testid="logout"], button:has-text("Logout"), a:has-text("Logout")');
    if (await logoutBtn.isVisible()) {
      await logoutBtn.first().click();
    }
  }

  async createDocument(title: string, content: string = '') {
    await this.goto('/documents');
    const createBtn = this.page.locator('[data-testid="create-document"], button:has-text("New"), button:has-text("Create"), a:has-text("New")');
    await createBtn.first().click();

    const titleInput = this.page.locator('[name="title"], input[placeholder*="title" i]');
    if (await titleInput.isVisible()) {
      await titleInput.fill(title);
    }

    if (content) {
      const contentArea = this.page.locator('[name="content"], textarea, [contenteditable="true"]');
      if (await contentArea.isVisible()) {
        await contentArea.fill(content);
      }
    }

    const submitBtn = this.page.locator('button[type="submit"]:visible, button:has-text("Save"), button:has-text("Create")');
    if (await submitBtn.isVisible()) {
      await submitBtn.first().click();
    }
  }

  async search(query: string) {
    const searchInput = this.page.locator('[name="search"], input[placeholder*="search" i], [data-testid="search-input"]');
    await searchInput.first().fill(query);
    await searchInput.first().press('Enter');
  }

  async waitForToast(message?: string) {
    const toast = this.page.locator('[role="alert"], [data-testid="toast"], .toast, .notification');
    if (message) {
      await expect(toast).toContainText(message);
    } else {
      await toast.waitFor({ state: 'visible' });
    }
  }
}
