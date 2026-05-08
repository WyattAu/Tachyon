import http from 'k6/http';
import { check, sleep, randomItem } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const API = `${BASE_URL}/api/v1`;

const latencyTrend = new Trend('soak_latency');
const errorRate = new Rate('soak_errors');
const degradationTracker = new Rate('degradation_window');

let cachedToken = '';
let tokenExpiry = 0;
let lastErrorWindowStart = 0;
let windowErrorCount = 0;

function getAuthToken() {
  const now = Date.now();
  if (cachedToken && now < tokenExpiry) {
    return cachedToken;
  }

  const username = `soak_${__VU}_${Date.now()}`;
  const res = http.post(`${API}/auth/register`, JSON.stringify({
    username: username,
    display_name: `Soak VU ${__VU}`,
    password: 'SoakTest123!',
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

function soakOperation() {
  const ops = [
    () => {
      const res = http.get(`${API}/documents?page=1&page_size=20`, authHeaders());
      latencyTrend.add(res.timings.duration);
      if (res.status >= 500) errorRate.add(1);
      check(res, { 'soak list docs': (r) => r.status < 500 });
    },
    () => {
      const queries = ['test', 'document', 'note', 'search', 'content'];
      const q = randomItem(queries);
      const res = http.get(
        `${API}/search?q=${q}&page=1&page_size=10`,
        authHeaders()
      );
      latencyTrend.add(res.timings.duration);
      if (res.status >= 500) errorRate.add(1);
      check(res, { 'soak search': (r) => r.status < 500 });
    },
    () => {
      const uniqueId = `soak_${__VU}_${Date.now()}`;
      const res = http.post(`${API}/documents`, JSON.stringify({
        title: `Soak document ${uniqueId}`,
        content: `# Soak Test\n\nContent at ${new Date().toISOString()}.`,
        tags: ['soak'],
      }), authHeaders());
      latencyTrend.add(res.timings.duration);
      if (res.status >= 500) errorRate.add(1);
      check(res, {
        'soak create doc': (r) => r.status === 200 || r.status === 201,
      });
    },
    () => {
      const res = http.get(`${BASE_URL}/health`);
      latencyTrend.add(res.timings.duration);
      check(res, { 'soak health': (r) => r.status === 200 });

      if (res.status === 200) {
        const poolInfo = res.json('pool');
        if (poolInfo) {
          const active = poolInfo.active_connections || 0;
          const idle = poolInfo.idle_connections || 0;
          if (idle === 0 && active > 0) {
            degradationTracker.add(1);
          }
        }
      }
    },
    () => {
      const res = http.get(`${API}/auth/status`, authHeaders());
      latencyTrend.add(res.timings.duration);
      check(res, { 'soak auth status': (r) => r.status === 200 });
    },
    () => {
      const res = http.post(`${API}/auth/login`, JSON.stringify({
        username: 'soak_test_user',
        password: 'SoakTest123!',
      }), {
        headers: { 'Content-Type': 'application/json' },
      });
      latencyTrend.add(res.timings.duration);
      check(res, { 'soak login': (r) => r.status < 500 });
    },
  ];
  randomItem(ops)();
}

export const options = {
  stages: [
    { duration: '30m', target: 10 },
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'],
    soak_errors: ['rate<0.01'],
    degradation_window: ['rate<0.05'],
  },
};

export function setup() {
  const res = http.get(`${BASE_URL}/health`);
  if (res.status !== 200) {
    throw new Error(`Server not healthy: health returned ${res.status}`);
  }
  console.log('Soak test started. Server is healthy.');
  return { startTime: Date.now() };
}

export default function () {
  soakOperation();
  sleep(Math.random() * 2 + 1);
}

export function handleSummary(data) {
  const dur = data.metrics.http_req_duration;
  const errors = data.metrics.soak_errors;
  const degradation = data.metrics.degradation_window;
  const totalReqs = data.metrics.http_reqs.values.count;
  const testDuration = data.state.testRunDurationMs / 1000;
  const reqPerSec = totalReqs / testDuration;

  let out = '\n=== Soak Test Summary ===\n';
  out += `Duration: ${Math.round(testDuration)}s\n`;
  out += `Total requests: ${totalReqs}\n`;
  out += `Throughput: ${reqPerSec.toFixed(1)} req/s\n`;
  out += `p50: ${dur.values.med}ms, p95: ${dur.values['p(95)']}ms, p99: ${dur.values['p(99)']}ms, max: ${dur.values.max}ms\n`;
  out += `Error rate: ${(errors.values.rate * 100).toFixed(3)}%\n`;
  out += `Degradation windows: ${(degradation.values.rate * 100).toFixed(2)}%\n`;

  const medLatency = dur.values.med;
  const p95Latency = dur.values['p(95)'];
  out += '\nLatency Trend Analysis:\n';
  out += `  Median latency: ${medLatency}ms\n`;
  out += `  P95 latency: ${p95Latency}ms\n`;

  if (p95Latency > 400) {
    out += '\n  WARNING: P95 latency approaching threshold (500ms).\n';
    out += '  This may indicate gradual degradation or memory growth.\n';
  }

  if (errors.values.rate > 0.005) {
    out += '\n  WARNING: Error rate elevated. Check for connection leaks or resource exhaustion.\n';
  }

  return {
    stdout: out,
    'reports/soak-test-summary.json': JSON.stringify(data, null, 2),
  };
}
