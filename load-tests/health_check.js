import http from 'k6/http';
import { check } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const errorRate = new Rate('errors');
const latencyP99 = new Trend('latency_p99', true);

export const options = {
  stages: [
    { duration: '5s', target: 20 },
    { duration: '20s', target: 100 },
    { duration: '5s', target: 0 },
  ],
  thresholds: {
    http_req_failed: ['rate<0.01'],
    http_req_duration: ['p(99)<200', 'p(95)<100', 'p(50)<50'],
    errors: ['rate<0.01'],
  },
};

const BASE = __ENV.BASE_URL || 'http://localhost:8080';

export default function () {
  const res = http.get(`${BASE}/health`, {
    tags: { name: 'health_check' },
  });

  check(res, {
    'health status 200': (r) => r.status === 200,
    'health response time < 200ms': (r) => r.timings.duration < 200,
    'health body contains status': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.status === 'ok' || body.status === 'healthy';
      } catch {
        return r.body && r.body.length > 0;
      }
    },
  });

  errorRate.add(res.status !== 200);
  latencyP99.add(res.timings.duration);
}

export function handleSummary(data) {
  const p99 = data.metrics.http_req_duration?.values?.['p(99)'] || 0;
  const failRate = data.metrics.http_req_failed?.values?.rate || 0;
  return {
    stdout: JSON.stringify({
      test: 'health_check',
      virtual_users: data.metrics.vus_max?.value || 0,
      total_requests: data.metrics.http_reqs?.value || 0,
      p99_latency_ms: Math.round(p99 * 100) / 100,
      failure_rate: Math.round(failRate * 10000) / 100 + '%',
      threshold_p99_pass: p99 < 200,
    }, null, 2),
  };
}
