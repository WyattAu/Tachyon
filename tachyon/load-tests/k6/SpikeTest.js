import http from 'k6/http';
import { check, sleep, randomItem } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const API = `${BASE_URL}/api/v1`;

const baselineLatency = new Trend('baseline_latency');
const spikeLatency = new Trend('spike_latency');
const recoveryLatency = new Trend('recovery_latency');
const spikeErrors = new Rate('spike_errors');

let cachedToken = '';
let tokenExpiry = 0;

function getAuthToken() {
  const now = Date.now();
  if (cachedToken && now < tokenExpiry) {
    return cachedToken;
  }

  const username = `spike_${__VU}_${Date.now()}`;
  const res = http.post(`${API}/auth/register`, JSON.stringify({
    username: username,
    display_name: `Spike VU ${__VU}`,
    password: 'SpikeTest123!',
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

function spikeOperation() {
  const ops = [
    () => {
      const res = http.get(`${API}/documents?page=1&page_size=20`, authHeaders());
      return res;
    },
    () => {
      const queries = ['test', 'document', 'note', 'search'];
      const q = randomItem(queries);
      return http.get(`${API}/search?q=${q}&page=1&page_size=20`, authHeaders());
    },
    () => {
      const uniqueId = `spike_${__VU}_${Date.now()}_${Math.random().toString(36).slice(2)}`;
      return http.post(`${API}/documents`, JSON.stringify({
        title: `Spike document ${uniqueId}`,
        content: `# Spike\n\nGenerated at ${new Date().toISOString()}.`,
        tags: ['spike'],
      }), authHeaders());
    },
    () => {
      return http.get(`${BASE_URL}/health`);
    },
    () => {
      return http.post(`${API}/auth/login`, JSON.stringify({
        username: 'spike_test_user',
        password: 'SpikeTest123!',
      }), {
        headers: { 'Content-Type': 'application/json' },
      });
    },
  ];

  const res = randomItem(ops)();
  const elapsed = Date.now();
  const testStart = Date.now() - (__ITER * 100);

  const elapsedMs = elapsed / 1000;

  if (res.status >= 500) {
    spikeErrors.add(1);
  }

  if (__VUs < 20) {
    baselineLatency.add(res.timings.duration);
  } else if (__VUs >= 100) {
    spikeLatency.add(res.timings.duration);
  } else {
    recoveryLatency.add(res.timings.duration);
  }

  check(res, {
    'spike request ok': (r) => r.status < 500,
  });
}

export const options = {
  stages: [
    { duration: '2m', target: 10 },
    { duration: '1m', target: 200 },
    { duration: '3m', target: 10 },
  ],
  thresholds: {
    http_req_duration: ['p(95)<1000'],
    spike_errors: ['rate<0.1'],
  },
};

export function setup() {
  const res = http.get(`${BASE_URL}/health`);
  if (res.status !== 200) {
    throw new Error(`Server not healthy: health returned ${res.status}`);
  }

  const baselineRes = http.get(`${API}/documents?page=1&page_size=1`);
  console.log(`Spike test started. Baseline latency: ${baselineRes.timings.duration}ms`);
  return { baselineLatencyMs: baselineRes.timings.duration };
}

export default function () {
  spikeOperation();
  sleep(Math.random() * 0.5 + 0.1);
}

export function handleSummary(data) {
  const baseline = data.metrics.baseline_latency;
  const spike = data.metrics.spike_latency;
  const recovery = data.metrics.recovery_latency;
  const errors = data.metrics.spike_errors;

  let out = '\n=== Spike Test Summary ===\n';
  out += '\n--- Baseline Phase (10 VUs, 2 min) ---\n';
  out += `  p50: ${baseline.values.med}ms, p95: ${baseline.values['p(95)']}ms\n`;

  out += '\n--- Spike Phase (200 VUs, 1 min) ---\n';
  out += `  p50: ${spike.values.med}ms, p95: ${spike.values['p(95)']}ms\n`;
  out += `  Error rate: ${(errors.values.rate * 100).toFixed(2)}%\n`;

  out += '\n--- Recovery Phase (10 VUs, 3 min) ---\n';
  out += `  p50: ${recovery.values.med}ms, p95: ${recovery.values['p(95)']}ms\n`;

  const baselineP95 = baseline.values['p(95)'];
  const spikeP95 = spike.values['p(95)'];
  const recoveryP95 = recovery.values['p(95)'];

  if (baselineP95 && recoveryP95) {
    const degradationRatio = spikeP95 / baselineP95;
    const recoveryRatio = recoveryP95 / baselineP95;

    out += '\n--- Analysis ---\n';
    out += `  Spike degradation: ${degradationRatio.toFixed(1)}x baseline\n`;
    out += `  Recovery ratio: ${recoveryRatio.toFixed(2)}x baseline\n`;

    if (recoveryRatio < 1.5) {
      out += '  PASS: System recovered well after spike.\n';
    } else if (recoveryRatio < 3.0) {
      out += '  WARNING: Recovery latency elevated. Monitor for connection pool issues.\n';
    } else {
      out += '  FAIL: System did not recover properly. Investigate resource leaks.\n';
    }
  }

  return {
    stdout: out,
    'reports/spike-test-summary.json': JSON.stringify(data, null, 2),
  };
}
