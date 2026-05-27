import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '30s', target: 100 },
    { duration: '60s', target: 500 },
    { duration: '60s', target: 1000 },
    { duration: '30s', target: 0 },
  ],
  thresholds: {
    http_req_failed: ['rate<0.05'],
    http_req_duration: ['p(95)<200', 'p(99)<500'],
  },
};

const BASE = __ENV.BASE_URL || 'http://localhost:8080';

const READ_ENDPOINTS = [
  () => http.get(`${BASE}/health`),
  () => http.get(`${BASE}/api/v1/documents?page=1&page_size=20`),
  () => http.get(`${BASE}/api/v1/documents/search?q=test`),
];

const WRITE_ENDPOINTS = [
  () => http.post(`${BASE}/api/v1/render/markdown`,
];

export default function () {
  // 80% reads, 20% writes
  const endpoint = Math.random() < 0.8
    ? READ_ENDPOINTS[Math.floor(Math.random() * READ_ENDPOINTS.length)]
    : WRITE_ENDPOINTS[0];

  if (endpoint === WRITE_ENDPOINTS[0]) {
    const res = endpoint(JSON.stringify({
      content: `# Load Test ${Date.now()}\n\nParagraph with some **bold** and *italic* text.\n\n- item 1\n- item 2\n- item 3\n\n\`\`\`rust\nfn main() { println!("hello"); }\n\`\`\``,
    }), { headers: { 'Content-Type': 'application/json' } });
    check(res, { 'write ok': (r) => r.status < 500 });
  } else {
    const res = endpoint();
    check(res, { 'read ok': (r) => r.status === 200 });
  }
  sleep(0.1);
}
