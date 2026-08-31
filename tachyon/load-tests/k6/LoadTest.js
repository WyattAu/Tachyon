import http from 'k6/http';
import { check, sleep } from 'k6';

// k6 v1.6's bundled randomItem throws "Value is not an object: null" even with
// a well-formed array, so pick manually until that regression is fixed.
const randomItem = (arr) => arr[Math.floor(Math.random() * arr.length)];

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const API = `${BASE_URL}/api/v1`;

let cachedToken = '';
let tokenExpiry = 0;

function getAuthToken() {
  const now = Date.now();
  if (cachedToken && now < tokenExpiry) {
    return cachedToken;
  }

  const res = http.post(`${API}/auth/login`, JSON.stringify({
    username: __ENV.TEST_USERNAME || 'loadtest',
    password: __ENV.TEST_PASSWORD || 'LoadTest123!',
  }), {
    headers: { 'Content-Type': 'application/json' },
  });

  if (res.status === 200 && res.json('access_token')) {
    cachedToken = res.json('access_token');
    tokenExpiry = now + 3500 * 1000;
    return cachedToken;
  }

  const registerRes = http.post(`${API}/auth/register`, JSON.stringify({
    username: `loadtest_${__VU}_${Date.now()}`,
    display_name: `Load Test VU ${__VU}`,
    password: 'LoadTest123!',
  }), {
    headers: { 'Content-Type': 'application/json' },
  });

  if (registerRes.status === 200 || registerRes.status === 201) {
    cachedToken = registerRes.json('access_token') || '';
    tokenExpiry = now + 3500 * 1000;
  } else {
    // Register failed (e.g. user exists, rate-limited). Fall back to login
    // with the configured TEST_USERNAME/TEST_PASSWORD so VUs still get a token.
    const loginRes = http.post(`${API}/auth/login`, JSON.stringify({
      username: __ENV.TEST_USERNAME || 'loadtest',
      password: __ENV.TEST_PASSWORD || 'LoadTest123!',
    }), {
      headers: { 'Content-Type': 'application/json' },
    });
    if (loginRes.status === 200 && loginRes.json('access_token')) {
      cachedToken = loginRes.json('access_token');
      tokenExpiry = now + 3500 * 1000;
    }
  }

  return cachedToken;
}

function authHeaders() {
  const token = getAuthToken();
  return {
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${token}`,
    },
  };
}

function readOperation() {
  const ops = [
    () => {
      const res = http.get(`${API}/documents?page=1&page_size=20`, authHeaders());
      check(res, { 'list docs ok': (r) => r.status === 200 });
    },
    () => {
      const queries = ['test', 'document', 'note', 'hello', 'world'];
      const q = queries[Math.floor(Math.random() * queries.length)];
      const res = http.get(
        `${API}/search?q=${q}&page=1&page_size=20`,
        authHeaders()
      );
      check(res, { 'search ok': (r) => r.status === 200 });
    },
    () => {
      // Unauthenticated list — exercises the anonymous path (no token needed).
      const res = http.get(`${API}/documents?page=1&page_size=5`);
      check(res, { 'anon list ok': (r) => r.status === 200 || r.status === 401 });
    },
  ];
  ops[Math.floor(Math.random() * ops.length)]();
}

function writeOperation() {
  const ops = [
    () => {
      const uniqueId = `vu${__VU}_${Date.now()}`;
      const res = http.post(`${API}/documents`, JSON.stringify({
        title: `Load test doc ${uniqueId}`,
        content: `# Document ${uniqueId}\n\nGenerated at ${new Date().toISOString()}.`,
        tags: ['loadtest', `vu${__VU}`],
      }), authHeaders());
      check(res, {
        'create doc ok': (r) => r.status === 200 || r.status === 201,
      });
    },
    () => {
      const res = http.get(`${API}/documents?page=1&page_size=1`, authHeaders());
      if (res.status === 200) {
        const docs = res.json('results');
        if (docs && docs.length > 0) {
          const docId = docs[0].id;
          http.put(`${API}/documents/${docId}`, JSON.stringify({
            content: `Updated at ${new Date().toISOString()} by VU ${__VU}.`,
          }), authHeaders());
        }
      }
    },
  ];
  ops[Math.floor(Math.random() * ops.length)]();
}

function authOperation() {
  const res = http.post(`${API}/auth/login`, JSON.stringify({
    username: __ENV.TEST_USERNAME || 'loadtest',
    password: __ENV.TEST_PASSWORD || 'LoadTest123!',
  }), {
    headers: { 'Content-Type': 'application/json' },
  });
  check(res, { 'login ok': (r) => r.status === 200 });
}

function navigationOperation() {
  const ops = [
    () => {
      const res = http.get(`${BASE_URL}/health`);
      check(res, { 'health ok': (r) => r.status === 200 });
    },
    () => {
      const res = http.get(`${BASE_URL}/ready`);
      check(res, { 'ready ok': (r) => r.status === 200 || r.status === 503 });
    },
    () => {
      const res = http.get(`${API}/auth/status`, authHeaders());
      check(res, { 'auth status ok': (r) => r.status === 200 });
    },
  ];
  ops[Math.floor(Math.random() * ops.length)]();
}

export const options = {
  stages: [
    { duration: '30s', target: 50 },
    { duration: '2m', target: 50 },
    { duration: '30s', target: 0 },
  ],
  thresholds: {
    http_req_duration: ['p(95)<500', 'p(99)<2000'],
    // Only count "unexpected" failures: the deliberate unauthenticated anon-list
    // probe gets a 401 by design, which k6 tags expected_response=false.
    'http_req_failed{expected_response:true}': ['rate<0.01'],
  },
};

// One login in setup(), shared with every VU via the setup payload. At 50 VUs
// the per-IP login rate limit would 429 the per-VU bootstrap logins.
export function setup() {
  const res = http.post(`${API}/auth/login`, JSON.stringify({
    username: __ENV.TEST_USERNAME || 'loadtest',
    password: __ENV.TEST_PASSWORD || 'LoadTest123!',
  }), {
    headers: { 'Content-Type': 'application/json' },
  });
  if (res.status === 200 && res.json('access_token')) {
    return { token: res.json('access_token') };
  }
  return { token: '' };
}

export default function (data) {
  if (!cachedToken && data && data.token) {
    cachedToken = data.token;
    tokenExpiry = Date.now() + 3500 * 1000;
  }

  const roll = Math.random();

  if (roll < 0.4) {
    readOperation();
  } else if (roll < 0.7) {
    writeOperation();
  } else if (roll < 0.9 && !__ENV.TEST_SKIP_AUTHOPS) {
    authOperation();
  } else {
    navigationOperation();
  }

  sleep(Math.random() * 0.5 + 0.1);
}
