import http from 'k6/http';
import { check } from 'k6';

export const options = {
  stages: [
    { duration: '10s', target: 10 },
    { duration: '20s', target: 50 },
    { duration: '10s', target: 0 },
  ],
  thresholds: {
    http_req_failed: ['rate<0.01'],
    http_req_duration: ['p(95)<500'],
  },
};

const BASE = __ENV.BASE_URL || 'http://localhost:8080';

export default function () {
  // Health check
  const health = http.get(`${BASE}/health`);
  check(health, { 'health 200': (r) => r.status === 200 });

  // Document list
  const docs = http.get(`${BASE}/api/v1/documents?page=1&page_size=20`);
  check(docs, { 'docs 200': (r) => r.status === 200 });

  // Render markdown
  const render = http.post(`${BASE}/api/v1/render/markdown`,
    JSON.stringify({ content: '# Test\n\nHello **world**' }),
    { headers: { 'Content-Type': 'application/json' } },
  );
  check(render, { 'render 200': (r) => r.status === 200 });

  // Search
  const search = http.get(`${BASE}/api/v1/documents/search?q=test&page=1&page_size=10`);
  check(search, { 'search 200': (r) => r.status === 200 });
}
