import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

const readErrors = new Rate('read_errors');
const readLatency = new Trend('read_latency', true);
const cacheHits = new Counter('cache_hits');

export const options = {
  stages: [
    { duration: '5s', target: 50 },
    { duration: '20s', target: 200 },
    { duration: '5s', target: 0 },
  ],
  thresholds: {
    http_req_failed: ['rate<0.01'],
    http_req_duration: ['p(99)<200', 'p(95)<100', 'p(50)<50'],
    read_errors: ['rate<0.01'],
    read_latency: ['p(99)<200'],
  },
};

const BASE = __ENV.BASE_URL || 'http://localhost:8080';
const AUTH_TOKEN = __ENV.AUTH_TOKEN || '';

const headers = AUTH_TOKEN
  ? { 'Authorization': `Bearer ${AUTH_TOKEN}` }
  : {};

const READ_ENDPOINTS = [
  { path: '/health', name: 'health' },
  { path: '/api/v1/documents?page=1&page_size=20', name: 'doc_list' },
  { path: '/api/v1/documents?page=2&page_size=20', name: 'doc_list_p2' },
  { path: '/api/v1/documents/search?q=test&page=1&page_size=10', name: 'search' },
];

export default function () {
  const endpoint = READ_ENDPOINTS[Math.floor(Math.random() * READ_ENDPOINTS.length)];
  const url = `${BASE}${endpoint.path}`;

  const res = http.get(url, {
    headers,
    tags: { name: endpoint.name },
  });

  const ok = check(res, {
    'read status 200': (r) => r.status === 200,
    'read latency < 200ms': (r) => r.timings.duration < 200,
  });

  readLatency.add(res.timings.duration);
  readErrors.add(!ok);

  if (res.status === 200) {
    const cacheHeader = res.headers['X-Cache'] || res.headers['x-cache'] || '';
    if (cacheHeader.toLowerCase().includes('hit')) {
      cacheHits.add(1);
    }
  }

  sleep(0.02);
}

export function handleSummary(data) {
  const p99 = data.metrics.http_req_duration?.values?.['p(99)'] || 0;
  const p95 = data.metrics.http_req_duration?.values?.['p(95)'] || 0;
  const p50 = data.metrics.http_req_duration?.values?.['p(50)'] || 0;
  const failRate = data.metrics.http_req_failed?.values?.rate || 0;
  const totalReqs = data.metrics.http_reqs?.value || 0;
  const duration = data.state?.testRunDuration || 30000;
  const rps = Math.round((totalReqs / (duration / 1000)) * 100) / 100;

  return {
    stdout: JSON.stringify({
      test: 'concurrent_reads',
      virtual_users: data.metrics.vus_max?.value || 0,
      total_requests: totalReqs,
      requests_per_second: rps,
      latency_p50_ms: Math.round(p50 * 100) / 100,
      latency_p95_ms: Math.round(p95 * 100) / 100,
      latency_p99_ms: Math.round(p99 * 100) / 100,
      failure_rate: Math.round(failRate * 10000) / 100 + '%',
      cache_hits: data.metrics.cache_hits?.value || 0,
      threshold_p99_pass: p99 < 200,
    }, null, 2),
  };
}
