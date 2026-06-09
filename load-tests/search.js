import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';
import { randomItem } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

const searchErrors = new Rate('search_errors');
const searchLatency = new Trend('search_latency', true);

export const options = {
  stages: [
    { duration: '10s', target: 25 },
    { duration: '40s', target: 100 },
    { duration: '10s', target: 0 },
  ],
  thresholds: {
    http_req_failed: ['rate<0.02'],
    http_req_duration: ['p(99)<200', 'p(95)<100'],
    search_errors: ['rate<0.02'],
    search_latency: ['p(99)<200'],
  },
};

const BASE = __ENV.BASE_URL || 'http://localhost:8080';
const AUTH_TOKEN = __ENV.AUTH_TOKEN || '';

const headers = AUTH_TOKEN
  ? { 'Content-Type': 'application/json', 'Authorization': `Bearer ${AUTH_TOKEN}` }
  : { 'Content-Type': 'application/json' };

const SEARCH_QUERIES = [
  'rust',
  'database',
  'authentication',
  'search',
  'document',
  'test',
  'API',
  'docker',
  'frontend',
  'markdown',
  'security',
  'WebSocket',
  'CRDT',
  'editor',
  'plugin',
  'knowledge graph',
  'billing',
  'notification',
  'deployment',
  'performance',
];

const TAG_FILTERS = ['load-test', 'k6', 'performance', 'documentation', 'api'];

export default function () {
  const scenario = Math.random();

  if (scenario < 0.5) {
    const query = randomItem(SEARCH_QUERIES);
    const res = http.get(
      `${BASE}/api/v1/documents/search?q=${encodeURIComponent(query)}&page=1&page_size=20`,
      { headers, tags: { name: 'search_query' } }
    );

    const ok = check(res, {
      'search status 200': (r) => r.status === 200,
      'search has results': (r) => {
        try {
          const body = JSON.parse(r.body);
          return Array.isArray(body.results || body.items || body.data || body);
        } catch {
          return false;
        }
      },
    });

    searchLatency.add(res.timings.duration);
    searchErrors.add(!ok);
  } else if (scenario < 0.75) {
    const tag = randomItem(TAG_FILTERS);
    const res = http.get(
      `${BASE}/api/v1/documents?tags=${encodeURIComponent(tag)}&page=1&page_size=20`,
      { headers, tags: { name: 'search_tags' } }
    );

    const ok = check(res, {
      'tag filter status 200': (r) => r.status === 200,
    });

    searchLatency.add(res.timings.duration);
    searchErrors.add(!ok);
  } else {
    const query = randomItem(SEARCH_QUERIES);
    const tag = randomItem(TAG_FILTERS);
    const res = http.get(
      `${BASE}/api/v1/documents/search?q=${encodeURIComponent(query)}&tags=${encodeURIComponent(tag)}&page=1&page_size=10`,
      { headers, tags: { name: 'search_combined' } }
    );

    const ok = check(res, {
      'combined search status 200': (r) => r.status === 200,
    });

    searchLatency.add(res.timings.duration);
    searchErrors.add(!ok);
  }

  sleep(0.05);
}

export function handleSummary(data) {
  const p99 = data.metrics.http_req_duration?.values?.['p(99)'] || 0;
  const failRate = data.metrics.http_req_failed?.values?.rate || 0;
  return {
    stdout: JSON.stringify({
      test: 'search',
      virtual_users: data.metrics.vus_max?.value || 0,
      total_requests: data.metrics.http_reqs?.value || 0,
      p99_latency_ms: Math.round(p99 * 100) / 100,
      failure_rate: Math.round(failRate * 10000) / 100 + '%',
      threshold_p99_pass: p99 < 200,
    }, null, 2),
  };
}
