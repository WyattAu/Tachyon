import { test, expect } from '@playwright/test';

test.describe('Document Operations', () => {
  let authToken: string | null = null;

  test.beforeAll(async ({ request }) => {
    const response = await request.post('/api/v1/auth/login', {
      data: {
        username: 'admin',
        password: 'admin123'
      }
    });
    
    if (response.status() === 200) {
      const body = await response.json();
      authToken = body.access_token;
    }
  });

  test.beforeEach(async ({ page }) => {
    if (authToken) {
      await page.goto('/');
      await page.evaluate((token) => {
        localStorage.setItem('auth_token', token);
      }, authToken);
    }
  });

  test('should list documents', async ({ request }) => {
    const response = await request.get('/api/v1/documents', {
      headers: authToken ? { Authorization: `Bearer ${authToken}` } : {}
    });
    
    expect(response.status()).toBe(200);
    
    const body = await response.json();
    expect(body).toHaveProperty('results');
    expect(Array.isArray(body.results)).toBe(true);
  });

  test('should create a new document', async ({ request }) => {
    const response = await request.post('/api/v1/documents', {
      headers: authToken ? { Authorization: `Bearer ${authToken}` } : {},
      data: {
        title: `E2E Test Document ${Date.now()}`,
        content: '# Test Content\n\nThis is a test document.',
        visibility: 'private'
      }
    });
    
    expect([200, 201]).toContain(response.status());
    
    const body = await response.json();
    const doc = body.data || body;
    expect(doc.id || body.id).toBeDefined();
  });

  test('should get a document by ID', async ({ request }) => {
    const createResponse = await request.post('/api/v1/documents', {
      headers: authToken ? { Authorization: `Bearer ${authToken}` } : {},
      data: {
        title: `Get Test Document ${Date.now()}`,
        content: 'Test content for retrieval'
      }
    });
    
    if (createResponse.status() === 200 || createResponse.status() === 201) {
      const { id } = await createResponse.json();
      
      const getResponse = await request.get(`/api/v1/documents/${id}`, {
        headers: authToken ? { Authorization: `Bearer ${authToken}` } : {}
      });
      
      expect(getResponse.status()).toBe(200);
      
      const body = await getResponse.json();
      expect(body.id).toBe(id);
    }
  });

  test('should update a document', async ({ request }) => {
    const createResponse = await request.post('/api/v1/documents', {
      headers: authToken ? { Authorization: `Bearer ${authToken}` } : {},
      data: {
        title: `Update Test Document ${Date.now()}`,
        content: 'Original content'
      }
    });
    
    if (createResponse.status() === 200 || createResponse.status() === 201) {
      const { id } = await createResponse.json();
      
      const updateResponse = await request.put(`/api/v1/documents/${id}`, {
        headers: authToken ? { Authorization: `Bearer ${authToken}` } : {},
        data: {
          title: `Updated Document ${Date.now()}`,
          content: 'Updated content'
        }
      });
      
      expect([200, 204]).toContain(updateResponse.status());
    }
  });

  test('should delete a document', async ({ request }) => {
    const createResponse = await request.post('/api/v1/documents', {
      headers: authToken ? { Authorization: `Bearer ${authToken}` } : {},
      data: {
        title: `Delete Test Document ${Date.now()}`,
        content: 'Content to be deleted'
      }
    });
    
    if (createResponse.status() === 200 || createResponse.status() === 201) {
      const { id } = await createResponse.json();
      
      const deleteResponse = await request.delete(`/api/v1/documents/${id}`, {
        headers: authToken ? { Authorization: `Bearer ${authToken}` } : {}
      });
      
      expect([200, 204]).toContain(deleteResponse.status());
      
      const getResponse = await request.get(`/api/v1/documents/${id}`, {
        headers: authToken ? { Authorization: `Bearer ${authToken}` } : {}
      });
      
      expect(getResponse.status()).toBe(404);
    }
  });

  test('should search documents', async ({ request }) => {
    const response = await request.get('/api/v1/documents/search?search=test', {
      headers: authToken ? { Authorization: `Bearer ${authToken}` } : {}
    });
    
    expect(response.status()).toBe(200);
    
    const body = await response.json();
    expect(body).toHaveProperty('results');
  });

  test('should handle pagination', async ({ request }) => {
    const response = await request.get('/api/v1/documents?page=1&page_size=10', {
      headers: authToken ? { Authorization: `Bearer ${authToken}` } : {}
    });
    
    expect(response.status()).toBe(200);
    
    const body = await response.json();
    expect(body.page).toBe(1);
    expect(body.page_size).toBeLessThanOrEqual(10);
  });

  test('should validate document creation', async ({ request }) => {
    const response = await request.post('/api/v1/documents', {
      headers: authToken ? { Authorization: `Bearer ${authToken}` } : {},
      data: {
        title: '',
        content: ''
      }
    });
    
    expect(response.status()).toBe(400);
  });

  test('should return 404 for non-existent document', async ({ request }) => {
    const fakeId = '00000000-0000-0000-0000-000000000000';
    
    const response = await request.get(`/api/v1/documents/${fakeId}`, {
      headers: authToken ? { Authorization: `Bearer ${authToken}` } : {}
    });
    
    expect(response.status()).toBe(404);
  });
});

test.describe('Document Rendering', () => {
  test('should render markdown to HTML', async ({ request }) => {
    const response = await request.post('/api/v1/render/markdown', {
      data: '# Hello World\n\nThis is **bold** text.'
    });
    
    expect(response.status()).toBe(200);
    
    const body = await response.json();
    expect(body.html).toBeDefined();
    expect(body.html).toContain('<h1>');
    expect(body.html).toContain('<strong>');
  });

  test('should handle code blocks in markdown', async ({ request }) => {
    const response = await request.post('/api/v1/render/markdown', {
      data: '```rust\nfn main() {}\n```'
    });
    
    expect(response.status()).toBe(200);
    
    const body = await response.json();
    expect(body.html).toContain('code');
  });

  test('should return metadata in render response', async ({ request }) => {
    const response = await request.post('/api/v1/render/markdown', {
      data: '# Title\n\nParagraph here.'
    });
    
    expect(response.status()).toBe(200);
    
    const body = await response.json();
    expect(body.word_count).toBeDefined();
    expect(body.character_count).toBeDefined();
  });
});

test.describe('Document Versions', () => {
  let authToken: string | null = null;
  let documentId: string | null = null;

  test.beforeAll(async ({ request }) => {
    const loginResponse = await request.post('/api/v1/auth/login', {
      data: { username: 'admin', password: 'admin123' }
    });
    
    if (loginResponse.status() === 200) {
      authToken = (await loginResponse.json()).access_token;
      
      const createResponse = await request.post('/api/v1/documents', {
        headers: { Authorization: `Bearer ${authToken}` },
        data: {
          title: `Version Test Doc ${Date.now()}`,
          content: 'Initial content'
        }
      });
      
      if (createResponse.status() === 200 || createResponse.status() === 201) {
        documentId = (await createResponse.json()).id;
      }
    }
  });

  test('should list document versions', async ({ request }) => {
    if (!documentId || !authToken) {
      test.skip();
      return;
    }
    
    const response = await request.get(`/api/v1/documents/${documentId}/versions`, {
      headers: { Authorization: `Bearer ${authToken}` }
    });
    
    expect(response.status()).toBe(200);
    
    const body = await response.json();
    expect(Array.isArray(body)).toBe(true);
  });

  test('should create a new version', async ({ request }) => {
    if (!documentId || !authToken) {
      test.skip();
      return;
    }
    
    const response = await request.post(`/api/v1/documents/${documentId}/versions`, {
      headers: { Authorization: `Bearer ${authToken}` },
      data: {
        content: 'Updated content for version',
        commit_message: 'E2E test version'
      }
    });
    
    expect([200, 201]).toContain(response.status());
  });
});
