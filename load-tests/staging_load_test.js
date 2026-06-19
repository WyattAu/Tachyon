// k6 Load Test for Tachyon Server
// Usage: k6 run load-tests/staging_load_test.js

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const apiLatency = new Trend('api_latency');

// Configuration
const BASE_URL = __ENV.BASE_URL || 'http://192.168.1.191:18080';
const USERNAME = __ENV.USERNAME || 'admin';
const PASSWORD = __ENV.PASSWORD || 'admin123';

// Test configuration
export const options = {
  stages: [
    { duration: '30s', target: 10 },   // Ramp up to 10 users
    { duration: '1m', target: 10 },    // Stay at 10 users
    { duration: '30s', target: 25 },   // Ramp up to 25 users
    { duration: '1m', target: 25 },    // Stay at 25 users
    { duration: '30s', target: 50 },   // Ramp up to 50 users
    { duration: '1m', target: 50 },    // Stay at 50 users
    { duration: '30s', target: 100 },  // Ramp up to 100 users
    { duration: '2m', target: 100 },   // Stay at 100 users
    { duration: '30s', target: 0 },    // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<500', 'p(99)<1000'],
    http_req_failed: ['rate<0.01'],
    errors: ['rate<0.01'],
  },
};

// Setup function - runs once per VU
export function setup() {
  // Login and get token
  const loginRes = http.post(
    `${BASE_URL}/api/v1/auth/login`,
    JSON.stringify({
      username: USERNAME,
      password: PASSWORD,
    }),
    {
      headers: { 'Content-Type': 'application/json' },
    }
  );

  check(loginRes, {
    'login successful': (r) => r.status === 200,
  });

  const token = loginRes.json('access_token');
  return { token };
}

// Main test function - runs for each iteration per VU
export default function (data) {
  const params = {
    headers: {
      Authorization: `Bearer ${data.token}`,
      'Content-Type': 'application/json',
    },
  };

  // Test 1: Health check (no auth)
  const healthRes = http.get(`${BASE_URL}/health`);
  check(healthRes, {
    'health check status 200': (r) => r.status === 200,
  });
  apiLatency.add(healthRes.timings.duration);
  errorRate.add(healthRes.status !== 200);
  sleep(1);

  // Test 2: List documents
  const listRes = http.get(`${BASE_URL}/api/v1/documents?page=1&page_size=10`, params);
  check(listRes, {
    'list documents status 200': (r) => r.status === 200,
  });
  apiLatency.add(listRes.timings.duration);
  errorRate.add(listRes.status !== 200);
  sleep(1);

  // Test 3: Create document
  const docTitle = `Load Test Doc ${Date.now()}-${__VU}`;
  const createRes = http.post(
    `${BASE_URL}/api/v1/documents`,
    JSON.stringify({
      title: docTitle,
      content: `# ${docTitle}\n\nThis is a load test document with [[wiki-links]].`,
      tags: ['load-test'],
    }),
    params
  );
  check(createRes, {
    'create document status 200': (r) => r.status === 200,
  });
  apiLatency.add(createRes.timings.duration);
  errorRate.add(createRes.status !== 200);

  const docId = createRes.json('id');
  sleep(1);

  // Test 4: Get document
  if (docId) {
    const getRes = http.get(`${BASE_URL}/api/v1/documents/${docId}`, params);
    check(getRes, {
      'get document status 200': (r) => r.status === 200,
    });
    apiLatency.add(getRes.timings.duration);
    errorRate.add(getRes.status !== 200);
    sleep(1);

    // Test 5: Update document
    const updateRes = http.put(
      `${BASE_URL}/api/v1/documents/${docId}`,
      JSON.stringify({
        title: `${docTitle} (updated)`,
        content: `# ${docTitle} (updated)\n\nThis document has been updated.`,
      }),
      params
    );
    check(updateRes, {
      'update document status 200': (r) => r.status === 200,
    });
    apiLatency.add(updateRes.timings.duration);
    errorRate.add(updateRes.status !== 200);
    sleep(1);

    // Test 6: Search
    const searchRes = http.get(`${BASE_URL}/api/v1/search?q=load+test`, params);
    check(searchRes, {
      'search status 200': (r) => r.status === 200,
    });
    apiLatency.add(searchRes.timings.duration);
    errorRate.add(searchRes.status !== 200);
    sleep(1);

    // Test 7: Graph nodes
    const graphRes = http.get(`${BASE_URL}/api/v1/graph/nodes`, params);
    check(graphRes, {
      'graph nodes status 200': (r) => r.status === 200,
    });
    apiLatency.add(graphRes.timings.duration);
    errorRate.add(graphRes.status !== 200);
    sleep(1);

    // Test 8: Delete document
    const deleteRes = http.del(`${BASE_URL}/api/v1/documents/${docId}`, null, params);
    check(deleteRes, {
      'delete document status 204': (r) => r.status === 204,
    });
    apiLatency.add(deleteRes.timings.duration);
    errorRate.add(deleteRes.status !== 204);
  }

  sleep(2);
}

// Teardown function - runs once after all VUs finish
export function teardown(data) {
  console.log('Load test complete');
}
