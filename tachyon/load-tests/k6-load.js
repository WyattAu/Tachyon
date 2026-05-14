import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Counter, Trend } from 'k6/metrics';

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const RAMP_DURATION = __ENV.RAMP_DURATION || '30s';
const TEST_DURATION = __ENV.TEST_DURATION || '60s';

const errorRate = new Rate('errors');
const requestLatency = new Trend('request_latency');
const requestCount = new Counter('requests');

export const options = {
    stages: [
        { duration: RAMP_DURATION, target: 100 },
        { duration: TEST_DURATION, target: 1000 },
    ],
    thresholds: {
        http_req_duration: ['p(95)<200', 'p(99)<500'],
        errors: ['rate<0.01'],
    },
};

export default function () {
    const healthRes = http.get(`${BASE_URL}/health`);
    check(healthRes, { 'Health check is 200': (r) => r.status === 200 });

    const docRes = http.batch([
        { method: 'GET', url: `${BASE_URL}/api/v1/documents?page=1&page_size=20` },
        { method: 'GET', url: `${BASE_URL}/api/v1/documents?page=2&page_size=20` },
        { method: 'GET', url: `${BASE_URL}/health` },
        { method: 'GET', url: `${BASE_URL}/api/v1/spaces` },
    ]);

    for (const res of docRes) {
        check(res, { 'Response is 200': (r) => r.status === 200 });
        requestLatency.add(res.timings.duration);
        requestCount.add(1);
        if (res.status !== 200) errorRate.add(1);
    }

    sleep(0.1);
}
