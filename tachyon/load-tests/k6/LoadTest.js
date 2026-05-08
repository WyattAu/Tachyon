import http from 'k6/http';
import { check, sleep, randomItem } from 'k6';

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
      const q = randomItem(queries);
      const res = http.get(
        `${API}/search?q=${q}&page=1&page_size=20`,
        authHeaders()
      );
      check(res, { 'search ok': (r) => r.status === 200 });
    },
  ];
  randomItem(ops)();
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
  randomItem(ops)();
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
  randomItem(ops)();
}

export const options = {
  stages: [
    { duration: '30s', target: 50 },
    { duration: '2m', target: 50 },
    { duration: '30s', target: 0 },
  ],
  thresholds: {
    http_req_duration: ['p(95)<500', 'p(99)<2000'],
    http_req_failed: ['rate<0.01'],
  },
};

export default function () {
  const roll = Math.random();

  if (roll < 0.4) {
    readOperation();
  } else if (roll < 0.7) {
    writeOperation();
  } else if (roll < 0.9) {
    authOperation();
  } else {
    navigationOperation();
  }

  sleep(Math.random() * 0.5 + 0.1);
}
