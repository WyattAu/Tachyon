import http from 'k6/http';
import { check, sleep } from 'k6';

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const API = `${BASE_URL}/api/v1`;

const uniqueId = `smoke_${Date.now()}`;
const testUser = {
  username: `smoke_user_${uniqueId}`,
  displayName: 'Smoke Test User',
  email: `smoke_${uniqueId}@tachyon.test`,
  password: 'TestPassword123!',
};

export const options = {
  vus: 1,
  iterations: 1,
  thresholds: {
    http_req_duration: ['p(100)<500'],
  },
};

export default function () {
  let res;
  let accessToken = '';
  let documentId = '';

  res = http.get(`${BASE_URL}/health`);
  check(res, {
    'health check returns 200': (r) => r.status === 200,
    'health check returns healthy': (r) =>
      r.json('status') === 'healthy' || r.json('status') === 'degraded',
  });

  res = http.get(`${BASE_URL}/ready`);
  check(res, {
    'readiness check returns 200 or 503': (r) =>
      r.status === 200 || r.status === 503,
  });

  res = http.post(`${API}/auth/register`, JSON.stringify({
    username: testUser.username,
    display_name: testUser.displayName,
    email: testUser.email,
    password: testUser.password,
  }), {
    headers: { 'Content-Type': 'application/json' },
  });
  check(res, {
    'register returns 200 or 201': (r) => r.status === 200 || r.status === 201,
    'register returns access_token': (r) => {
      if (r.status === 200 || r.status === 201) {
        const body = r.json();
        if (body && body.access_token) {
          accessToken = body.access_token;
          return true;
        }
      }
      return false;
    },
  });

  const authHeaders = {
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${accessToken}`,
    },
  };

  res = http.post(`${API}/auth/login`, JSON.stringify({
    username: testUser.username,
    password: testUser.password,
  }), {
    headers: { 'Content-Type': 'application/json' },
  });
  check(res, {
    'login returns 200': (r) => r.status === 200,
    'login returns success': (r) => r.json('success') === true,
  });
  if (res.status === 200) {
    const body = res.json();
    if (body.access_token) {
      accessToken = body.access_token;
      authHeaders.headers.Authorization = `Bearer ${accessToken}`;
    }
  }

  res = http.get(`${API}/auth/me`, authHeaders);
  check(res, {
    'get me returns 200': (r) => r.status === 200,
    'get me returns username': (r) =>
      r.status === 200 && r.json('username') === testUser.username,
  });

  res = http.post(`${API}/documents`, JSON.stringify({
    title: `Smoke test document ${uniqueId}`,
    content: '# Hello World\n\nThis is a smoke test document.',
    tags: ['smoke', 'test'],
    visibility: 'private',
  }), authHeaders);
  check(res, {
    'create document returns 200 or 201': (r) =>
      r.status === 200 || r.status === 201,
    'create document returns id': (r) => {
      if (r.status === 200 || r.status === 201) {
        const body = r.json();
        if (body && body.id) {
          documentId = body.id;
          return true;
        }
      }
      return false;
    },
  });

  if (documentId) {
    res = http.get(`${API}/documents/${documentId}`, authHeaders);
    check(res, {
      'get document returns 200': (r) => r.status === 200,
      'get document has correct title': (r) =>
        r.status === 200 &&
        r.json('title').includes('Smoke test document'),
    });

    res = http.put(`${API}/documents/${documentId}`, JSON.stringify({
      content: '# Updated\n\nUpdated content.',
    }), authHeaders);
    check(res, {
      'update document returns 200': (r) => r.status === 200,
    });
  }

  res = http.get(`${API}/documents?page=1&page_size=10`, authHeaders);
  check(res, {
    'list documents returns 200': (r) => r.status === 200,
    'list documents has results array': (r) =>
      r.status === 200 && Array.isArray(r.json('results')),
  });

  res = http.get(`${API}/search?q=test&page=1&page_size=10`, authHeaders);
  check(res, {
    'search returns 200': (r) => r.status === 200,
    'search has results': (r) =>
      r.status === 200 && r.json('results') !== undefined,
  });

  res = http.get(`${API}/auth/status`, authHeaders);
  check(res, {
    'auth status returns 200': (r) => r.status === 200,
    'auth status shows authenticated': (r) =>
      r.status === 200 && r.json('authenticated') === true,
  });

  res = http.post(`${API}/auth/logout`, JSON.stringify({}), authHeaders);
  check(res, {
    'logout returns 200': (r) => r.status === 200,
  });
}
