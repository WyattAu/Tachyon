import http from 'k6/http';
import { check, sleep, randomItem } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const API = `${BASE_URL}/api/v1`;

const connectionErrors = new Rate('connection_errors');
const poolExhaustion = new Rate('pool_exhaustion');
const responseSize = new Trend('response_size_bytes');

let cachedToken = '';
let tokenExpiry = 0;

function getAuthToken() {
  const now = Date.now();
  if (cachedToken && now < tokenExpiry) {
    return cachedToken;
  }

  const username = `stress_${__VU}_${Date.now()}`;
  const res = http.post(`${API}/auth/register`, JSON.stringify({
    username: username,
    display_name: `Stress VU ${__VU}`,
    password: 'StressTest123!',
  }), {
    headers: { 'Content-Type': 'application/json' },
  });

  if ((res.status === 200 || res.status === 201) && res.json('access_token')) {
    cachedToken = res.json('access_token');
    tokenExpiry = now + 3500 * 1000;
  }

  return cachedToken;
}

function authHeaders() {
  return {
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${getAuthToken()}`,
    },
  };
}

function stressOperation() {
  const ops = [
    () => {
      const res = http.get(`${API}/documents?page=1&page_size=50`, authHeaders());
      responseSize.add(res.body ? res.body.length : 0);
      if (res.status === 503) poolExhaustion.add(1);
      check(res, { 'list docs under stress': (r) => r.status < 500 });
    },
    () => {
      const queries = ['a', 'the', 'document', 'note', 'test', 'hello', 'search'];
      const q = randomItem(queries);
      const res = http.get(
        `${API}/search?q=${q}&page=1&page_size=20`,
        authHeaders()
      );
      responseSize.add(res.body ? res.body.length : 0);
      if (res.status === 503) poolExhaustion.add(1);
      check(res, { 'search under stress': (r) => r.status < 500 });
    },
    () => {
      const uniqueId = `stress_${__VU}_${Date.now()}_${Math.random().toString(36).slice(2)}`;
      const res = http.post(`${API}/documents`, JSON.stringify({
        title: `Stress document ${uniqueId}`,
        content: `# Stress Test\n\n${'Lorem ipsum dolor sit amet. '.repeat(20)}`,
        tags: ['stress', `batch${__VU}`],
      }), authHeaders());
      check(res, {
        'create doc under stress': (r) =>
          r.status === 200 || r.status === 201 || r.status === 503,
      });
    },
    () => {
      const res = http.post(`${API}/auth/login`, JSON.stringify({
        username: 'stress_test_user',
        password: 'StressTest123!',
      }), {
        headers: { 'Content-Type': 'application/json' },
      });
      check(res, { 'login under stress': (r) => r.status < 500 });
    },
    () => {
      const res = http.get(`${BASE_URL}/health`);
      responseSize.add(res.body ? res.body.length : 0);
      check(res, {
        'health under stress': (r) => r.status === 200 || r.status === 503,
      });
      if (res.status === 200) {
        const poolInfo = res.json('pool');
        if (poolInfo) {
          const utilization =
            (poolInfo.active_connections || 0) / (poolInfo.max_connections || 1);
          if (utilization > 0.9) {
            poolExhaustion.add(1);
          }
        }
      }
    },
    () => {
      const res = http.get(`${API}/documents?page=1&page_size=1`, authHeaders());
      if (res.status === 200) {
        const docs = res.json('results');
        if (docs && docs.length > 0) {
          const docId = docs[0].id;
          http.put(`${API}/documents/${docId}`, JSON.stringify({
            title: `Updated by stress VU ${__VU} at ${Date.now()}`,
          }), authHeaders());
        }
      }
    },
    () => {
      const res = http.get(`${BASE_URL}/ready`);
      check(res, { 'ready under stress': (r) => r.status < 500 });
    },
  ];
  randomItem(ops)();
}

export const options = {
  stages: [
    { duration: '1m', target: 200 },
    { duration: '5m', target: 200 },
    { duration: '30s', target: 0 },
  ],
  thresholds: {
    http_req_duration: ['p(95)<1000'],
    http_req_failed: ['rate<0.05'],
    connection_errors: ['rate<0.01'],
    pool_exhaustion: ['rate<0.1'],
  },
};

export function setup() {
  const res = http.get(`${BASE_URL}/health`);
  if (res.status !== 200) {
    console.warn(`Server may not be ready: health returned ${res.status}`);
  }
  return {};
}

export default function () {
  try {
    stressOperation();
  } catch (e) {
    connectionErrors.add(1);
    console.error(`VU ${__VU} error: ${e.message}`);
  }
  sleep(Math.random() * 0.3 + 0.05);
}

export function handleSummary(data) {
  const poolExh = data.metrics.pool_exhaustion;
  if (poolExh && poolExh.values.rate > 0.05) {
    data.extra = data.extra || {};
    data.extra.warnings = ['High connection pool exhaustion detected'];
  }

  return {
    stdout: summarizeOutput(data),
    'reports/stress-test-summary.json': JSON.stringify(data, null, 2),
  };
}

function summarizeOutput(data) {
  const dur = data.metrics.http_req_duration;
  const errors = data.metrics.http_req_failed;
  let out = '\n=== Stress Test Summary ===\n';
  out += `Total requests: ${data.metrics.http_reqs.values.count}\n`;
  out += `Duration: ${data.metrics.iteration_duration.values.count} iterations\n`;
  out += `p50: ${dur.values.med}ms, p95: ${dur.values['p(95)']}ms, p99: ${dur.values['p(99)']}ms\n`;
  out += `Error rate: ${(errors.values.rate * 100).toFixed(2)}%\n`;
  if (data.metrics.pool_exhaustion) {
    out += `Pool exhaustion rate: ${(data.metrics.pool_exhaustion.values.rate * 100).toFixed(2)}%\n`;
  }
  return out;
}
