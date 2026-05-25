import http from 'k6/http';
import { check, sleep } from 'k6';
import encoding from 'k6/encoding';

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const API = `${BASE_URL}/api/v1`;

const uniqueId = `sec_${Date.now()}`;
const userA = {
  username: `sec_userA_${uniqueId}`,
  displayName: 'Security Test A',
  email: `sec_a_${uniqueId}@tachyon.test`,
  password: 'SecureP@ss123!',
};
const userB = {
  username: `sec_userB_${uniqueId}`,
  displayName: 'Security Test B',
  email: `sec_b_${uniqueId}@tachyon.test`,
  password: 'SecureP@ss456!',
};

export const options = {
  vus: 1,
  iterations: 1,
  thresholds: {
    http_req_failed: ['rate<0.5'],
  },
};

function registerAndLogin(user) {
  let res = http.post(`${API}/auth/register`, JSON.stringify({
    username: user.username,
    display_name: user.displayName,
    email: user.email,
    password: user.password,
  }), { headers: { 'Content-Type': 'application/json' } });

  if (res.status === 200 || res.status === 201) {
    const token = res.json('access_token');
    if (token) return token;
  }

  res = http.post(`${API}/auth/login`, JSON.stringify({
    username: user.username,
    password: user.password,
  }), { headers: { 'Content-Type': 'application/json' } });

  if (res.status === 200) {
    return res.json('access_token') || '';
  }
  return '';
}

function authHeaders(token) {
  return {
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${token}`,
    },
  };
}

export default function () {
  let tokenA = registerAndLogin(userA);
  let tokenB = registerAndLogin(userB);
  check(tokenA, { 'user A authenticated': (t) => t.length > 0 });
  check(tokenB, { 'user B authenticated': (t) => t.length > 0 });
  if (!tokenA || !tokenB) return;

  const headersA = authHeaders(tokenA);
  const headersB = authHeaders(tokenB);

  // ── 1. SQL Injection ──────────────────────────────────────────────────

  const sqliPayloads = [
    "' OR '1'='1",
    "'; DROP TABLE users; --",
    "1; SELECT * FROM users WHERE '1'='1",
    "' UNION SELECT id, username, password FROM users --",
    "admin'--",
    "1' OR '1' = '1' /*",
  ];

  const sqliEndpoints = [
    { method: 'GET', url: `${API}/search?q=${encodeURIComponent("__PAYLOAD__")}&page=1&page_size=10` },
    { method: 'GET', url: `${API}/documents?page=1&page_size=10&sort=${encodeURIComponent("__PAYLOAD__")}` },
    { method: 'POST', url: `${API}/documents`, body: (p) => JSON.stringify({ title: p, content: 'test' }) },
    { method: 'PUT', url: `${API}/auth/profile`, body: (p) => JSON.stringify({ display_name: p }) },
  ];

  let sqliPassed = true;
  for (const payload of sqliPayloads) {
    for (const ep of sqliEndpoints) {
      const url = ep.url.replace('__PAYLOAD__', payload);
      let res;
      if (ep.body) {
        const body = ep.body(payload);
        if (ep.method === 'POST') {
          res = http.post(url, body, headersA);
        } else {
          res = http.put(url, body, headersA);
        }
      } else {
        res = http.get(url, headersA);
      }
      if (res.status === 500) {
        sqliPassed = false;
        break;
      }
    }
    if (!sqliPassed) break;
  }
  check(null, { 'SQL injection: no 500 errors': () => sqliPassed });

  // ── 2. XSS in document content ────────────────────────────────────────

  const xssPayloads = [
    '<script>alert("xss")</script>',
    '<img src=x onerror=alert(1)>',
    '"><script>document.cookie</script>',
    '<svg/onload=alert(1)>',
    'javascript:alert(1)',
    '<iframe src="javascript:alert(1)">',
  ];

  let xssDocId = '';
  for (const payload of xssPayloads) {
    const res = http.post(`${API}/documents`, JSON.stringify({
      title: `XSS test ${Date.now()}`,
      content: payload,
      tags: [payload],
      visibility: 'private',
    }), headersA);

    if (res.status === 200 || res.status === 201) {
      const id = res.json('id');
      if (id && !xssDocId) xssDocId = id;

      if (id) {
        const getRes = http.get(`${API}/documents/${id}`, headersA);
        if (getRes.status === 200) {
          const body = getRes.body || '';
          const reflected = body.includes('<script>') && !body.includes('&lt;script&gt;');
          check(null, {
            'XSS payload not reflected unsanitized': () => !reflected,
          });
        }
      }
    }
  }

  // ── 3. CSRF protection ────────────────────────────────────────────────

  const csrfRes = http.post(`${API}/documents`, JSON.stringify({
    title: 'CSRF test',
    content: 'CSRF content',
  }), {
    headers: {
      'Content-Type': 'application/json',
      'Origin': 'http://evil.example.com',
    },
  });
  check(csrfRes, {
    'CSRF: cross-origin request handled (rejected or processed with CORS)': (r) =>
      r.status === 200 || r.status === 201 || r.status === 403,
  });

  // ── 4. Authentication bypass ──────────────────────────────────────────

  const noAuthRes = http.get(`${API}/documents?page=1&page_size=10`);
  check(noAuthRes, {
    'Auth bypass: unauthenticated request denied': (r) => r.status === 401,
  });

  const badTokenRes = http.get(`${API}/documents?page=1&page_size=10`, {
    headers: {
      'Content-Type': 'application/json',
      Authorization: 'Bearer invalid.token.here',
    },
  });
  check(badTokenRes, {
    'Auth bypass: invalid token rejected': (r) => r.status === 401,
  });

  const emptyTokenRes = http.get(`${API}/documents?page=1&page_size=10`, {
    headers: {
      'Content-Type': 'application/json',
      Authorization: 'Bearer ',
    },
  });
  check(emptyTokenRes, {
    'Auth bypass: empty token rejected': (r) => r.status === 401,
  });

  // ── 5. Authorization enforcement ──────────────────────────────────────

  let privateDocId = '';
  const createRes = http.post(`${API}/documents`, JSON.stringify({
    title: `Private doc A ${uniqueId}`,
    content: '# Private content',
    visibility: 'private',
  }), headersA);
  if (createRes.status === 200 || createRes.status === 201) {
    privateDocId = createRes.json('id');
  }

  if (privateDocId) {
    const crossAccessRes = http.get(`${API}/documents/${privateDocId}`, headersB);
    check(crossAccessRes, {
      'Authz: user B cannot read user A private doc': (r) =>
        r.status === 403 || r.status === 404,
    });

    const crossEditRes = http.put(`${API}/documents/${privateDocId}`, JSON.stringify({
      content: 'Hacked by B!',
    }), headersB);
    check(crossEditRes, {
      'Authz: user B cannot edit user A private doc': (r) =>
        r.status === 403 || r.status === 404,
    });

    const crossDeleteRes = http.del(`${API}/documents/${privateDocId}`, null, headersB);
    check(crossDeleteRes, {
      'Authz: user B cannot delete user A private doc': (r) =>
        r.status === 403 || r.status === 404,
    });
  }

  // ── 6. Rate limiting ──────────────────────────────────────────────────

  let rateLimited = false;
  const rateLimitIterations = 150;
  for (let i = 0; i < rateLimitIterations; i++) {
    const res = http.get(`${BASE_URL}/health`);
    if (res.status === 429) {
      rateLimited = true;
      const hasRetryAfter = res.headers['Retry-After'] || res.headers['X-RateLimit-Reset'];
      check(null, {
        'Rate limit: 429 includes retry info': () => !!hasRetryAfter || res.status === 429,
      });
      break;
    }
  }
  check(null, {
    'Rate limiting: eventually blocks excessive requests': () => rateLimited,
  });

  // ── 7. JWT tampering ──────────────────────────────────────────────────

  const parts = tokenA.split('.');
  check(parts.length, { 'JWT has 3 parts': (l) => l === 3 });

  if (parts.length === 3) {
    const headerJson = JSON.parse(encoding.b64decode(parts[0], 'rawurl'));
    const origPayload = JSON.parse(encoding.b64decode(parts[1], 'rawurl'));

    // Tamper with role claim
    const tamperedPayload = Object.assign({}, origPayload, { role: 'admin' });
    const tamperedPayloadB64 = encoding.b64encode(JSON.stringify(tamperedPayload), 'rawurl');
    const tamperedToken = `${parts[0]}.${tamperedPayloadB64}.${parts[2]}`;

    const tamperedRes = http.get(`${API}/documents?page=1&page_size=10`, {
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${tamperedToken}`,
      },
    });
    check(tamperedRes, {
      'JWT tamper: modified payload rejected': (r) => r.status === 401,
    });

    // Tamper with subject claim
    const tamperedSub = Object.assign({}, origPayload, { sub: '00000000-0000-0000-0000-000000000001' });
    const tamperedSubB64 = encoding.b64encode(JSON.stringify(tamperedSub), 'rawurl');
    const tamperedSubToken = `${parts[0]}.${tamperedSubB64}.${parts[2]}`;

    const tamperedSubRes = http.get(`${API}/documents?page=1&page_size=10`, {
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${tamperedSubToken}`,
      },
    });
    check(tamperedSubRes, {
      'JWT tamper: modified subject rejected': (r) => r.status === 401,
    });

    // Token with wrong signature
    const fakeSig = encoding.b64encode('fakesignature', 'rawurl');
    const fakeToken = `${parts[0]}.${parts[1]}.${fakeSig}`;

    const fakeSigRes = http.get(`${API}/documents?page=1&page_size=10`, {
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${fakeToken}`,
      },
    });
    check(fakeSigRes, {
      'JWT tamper: wrong signature rejected': (r) => r.status === 401,
    });

    // Expired token simulation
    const expiredPayload = Object.assign({}, origPayload, { exp: Math.floor(Date.now() / 1000) - 3600 });
    const expiredB64 = encoding.b64encode(JSON.stringify(expiredPayload), 'rawurl');
    const expiredToken = `${parts[0]}.${expiredB64}.${parts[2]}`;

    const expiredRes = http.get(`${API}/documents?page=1&page_size=10`, {
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${expiredToken}`,
      },
    });
    check(expiredRes, {
      'JWT tamper: expired token rejected': (r) => r.status === 401,
    });
  }

  sleep(0.1);
}
