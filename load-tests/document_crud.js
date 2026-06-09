import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Counter, Rate, Trend } from 'k6/metrics';
import { randomString } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

const createdDocs = new Counter('docs_created');
const docsFailed = new Rate('docs_error_rate');
const createLatency = new Trend('doc_create_latency', true);
const readLatency = new Trend('doc_read_latency', true);
const updateLatency = new Trend('doc_update_latency', true);

export const options = {
  stages: [
    { duration: '10s', target: 15 },
    { duration: '40s', target: 50 },
    { duration: '10s', target: 0 },
  ],
  thresholds: {
    http_req_failed: ['rate<0.02'],
    http_req_duration: ['p(99)<200', 'p(95)<100'],
    docs_error_rate: ['rate<0.02'],
    doc_create_latency: ['p(99)<200'],
    doc_read_latency: ['p(99)<200'],
    doc_update_latency: ['p(99)<200'],
  },
};

const BASE = __ENV.BASE_URL || 'http://localhost:8080';
const AUTH_TOKEN = __ENV.AUTH_TOKEN || '';

const headers = AUTH_TOKEN
  ? { 'Content-Type': 'application/json', 'Authorization': `Bearer ${AUTH_TOKEN}` }
  : { 'Content-Type': 'application/json' };

function randomMarkdown() {
  const title = randomString(8);
  const body = randomString(32);
  return JSON.stringify({
    title: `Load Test Doc ${title}`,
    content: `# Load Test ${title}\n\nThis is a test document created during load testing.\n\n${body}\n\n## Section\n\n- Item 1\n- Item 2\n- Item 3\n\nSome **bold** and *italic* text.`,
    tags: ['load-test', 'k6', 'performance'],
  });
}

export default function () {
  group('Document CRUD Cycle', () => {
    let docId;

    group('CREATE document', () => {
      const payload = randomMarkdown();
      const res = http.post(`${BASE}/api/v1/documents`, payload, {
        headers,
        tags: { name: 'doc_create' },
      });

      const ok = check(res, {
        'create status 201 or 200': (r) => r.status === 201 || r.status === 200,
        'create has id': (r) => {
          try {
            const body = JSON.parse(r.body);
            docId = body.id || body.document_id;
            return !!docId;
          } catch {
            return false;
          }
        },
      });

      createLatency.add(res.timings.duration);
      docsFailed.add(!ok);
      if (ok) createdDocs.add(1);
    });

    if (!docId) return;

    sleep(0.1);

    group('READ document', () => {
      const res = http.get(`${BASE}/api/v1/documents/${docId}`, {
        headers,
        tags: { name: 'doc_read' },
      });

      const ok = check(res, {
        'read status 200': (r) => r.status === 200,
        'read body matches': (r) => {
          try {
            const body = JSON.parse(r.body);
            return body.title && body.title.includes('Load Test');
          } catch {
            return false;
          }
        },
      });

      readLatency.add(res.timings.duration);
      docsFailed.add(!ok);
    });

    sleep(0.1);

    group('UPDATE document', () => {
      const payload = JSON.stringify({
        content: `# Updated Doc\n\nUpdated at ${Date.now()}.\n\nThis content was modified during load testing.`,
      });

      const res = http.patch(`${BASE}/api/v1/documents/${docId}`, payload, {
        headers,
        tags: { name: 'doc_update' },
      });

      const ok = check(res, {
        'update status 200': (r) => r.status === 200 || r.status === 204,
      });

      updateLatency.add(res.timings.duration);
      docsFailed.add(!ok);
    });

    sleep(0.1);

    group('DELETE document', () => {
      const res = http.del(`${BASE}/api/v1/documents/${docId}`, null, {
        headers,
        tags: { name: 'doc_delete' },
      });

      check(res, {
        'delete status 200 or 204': (r) => r.status === 200 || r.status === 204 || r.status === 404,
      });
    });
  });
}

export function handleSummary(data) {
  const p99 = data.metrics.http_req_duration?.values?.['p(99)'] || 0;
  const failRate = data.metrics.http_req_failed?.values?.rate || 0;
  return {
    stdout: JSON.stringify({
      test: 'document_crud',
      virtual_users: data.metrics.vus_max?.value || 0,
      total_requests: data.metrics.http_reqs?.value || 0,
      docs_created: data.metrics.docs_created?.value || 0,
      p99_latency_ms: Math.round(p99 * 100) / 100,
      failure_rate: Math.round(failRate * 10000) / 100 + '%',
      threshold_p99_pass: p99 < 200,
    }, null, 2),
  };
}
