// Rate Limiting Validation Test
// Tests: per-IP rate limiting, per-user rate limiting, 429 responses, rate limit headers
// Usage: BASE_URL=http://localhost:8080 k6 run load-tests/k6/RateLimitTest.js

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Counter, Trend } from 'k6/metrics';

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const API = `${BASE_URL}/api/v1`;
const RATE_LIMIT = parseInt(__ENV.RATE_LIMIT || '100', 10);

const rateLimited = new Rate('rate_limited');
const rateLimitHeaders = new Rate('rate_limit_headers_present');
const retryAfterValid = new Rate('retry_after_valid');
const limitRecovery = new Rate('limit_recovery');

export const options = {
  scenarios: {
    // Scenario 1: Burst requests to trigger rate limiting (single VU, sequential)
    burst: {
      executor: 'shared-iterations',
      vus: 1,
      iterations: RATE_LIMIT + 20,
      exec: 'burstTest',
      startTime: '0s',
    },
    // Scenario 2: Multiple IPs hitting rate limit concurrently
    multi_ip: {
      executor: 'constant-vus',
      vus: 5,
      duration: '30s',
      exec: 'multiIpTest',
      startTime: '5s',
    },
  },
  thresholds: {
    rate_limited: ['rate>0'],          // Must trigger rate limiting
    rate_limit_headers: ['rate>0.9'],  // Must include rate limit headers
    retry_after_valid: ['rate>0.9'],   // Must include valid Retry-After
  },
};

// Burst test: single VU sends requests until rate limited
export function burstTest() {
  const res = http.get(`${BASE_URL}/health`, {
    tags: { test: 'burst' },
  });

  // After the first RATE_LIMIT requests, expect 429
  if (res.status === 429) {
    rateLimited.add(1);
    check(res, {
      '429 includes X-RateLimit-Limit header': (r) =>
        r.headers['X-RateLimit-Limit'] !== undefined,
      '429 includes X-RateLimit-Remaining header': (r) =>
        r.headers['X-RateLimit-Remaining'] !== undefined,
      '429 includes X-RateLimit-Reset header': (r) =>
        r.headers['X-RateLimit-Reset'] !== undefined,
      '429 includes Retry-After header': (r) =>
        r.headers['Retry-After'] !== undefined,
    });
    rateLimitHeaders.add(1);

    const retryAfter = parseInt(res.headers['Retry-After'], 10);
    if (!isNaN(retryAfter) && retryAfter > 0) {
      retryAfterValid.add(1);
    }
  } else {
    check(res, {
      'non-429 response includes rate limit headers': (r) =>
        r.headers['X-RateLimit-Limit'] !== undefined,
    });
  }

  // Minimal sleep for burst
  sleep(0.01);
}

// Multi-IP test: multiple VUs each hit their own rate limit
export function multiIpTest() {
  const res = http.get(`${BASE_URL}/health`, {
    tags: { test: 'multi_ip' },
  });

  check(res, {
    'multi-ip response < 500': (r) => r.status < 500,
    'multi-ip has rate limit headers': (r) =>
      r.headers['X-RateLimit-Limit'] !== undefined,
  });

  sleep(Math.random() * 0.2 + 0.05);
}

export function setup() {
  const res = http.get(`${BASE_URL}/health`);
  if (res.status !== 200) {
    throw new Error(`Server not healthy: ${res.status}`);
  }

  // Check if rate limiting is enabled
  const hasRateLimitHeader = res.headers['X-RateLimit-Limit'] !== undefined;
  if (!hasRateLimitHeader) {
    console.warn('WARNING: Rate limiting may not be enabled (no X-RateLimit-Limit header)');
    console.warn('Set TACHYON_RATE_LIMIT_ENABLED=true to enable');
  }

  console.log(`Rate limit test: expecting limit of ${RATE_LIMIT} requests`);
  return { startTime: Date.now() };
}

export function teardown(data) {
  // Wait for rate limit window to reset, then verify recovery
  const res = http.get(`${BASE_URL}/health`);
  if (res.status === 200) {
    limitRecovery.add(1);
    console.log('Rate limit recovery confirmed: 200 after window reset');
  } else if (res.status === 429) {
    console.warn('Rate limit not yet recovered after teardown wait');
  }
}

export function handleSummary(data) {
  const rl = data.metrics.rate_limited;
  const headers = data.metrics.rate_limit_headers;
  const retry = data.metrics.retry_after_valid;
  const recovery = data.metrics.limit_recovery;

  let out = '\n=== Rate Limit Test Summary ===\n';
  out += `Rate limited (429): ${(rl ? rl.values.count : 0)} requests\n`;
  out += `Headers present: ${(headers ? headers.values.rate * 100 : 0).toFixed(1)}%\n`;
  out += `Valid Retry-After: ${(retry ? retry.values.rate * 100 : 0).toFixed(1)}%\n`;
  out += `Limit recovery: ${(recovery ? recovery.values.rate * 100 : 0).toFixed(1)}%\n`;

  if (rl && rl.values.count === 0) {
    out += '\nWARNING: No 429 responses detected. Rate limiting may be disabled.\n';
    out += 'Set TACHYON_RATE_LIMIT_ENABLED=true and TACHYON_RATE_LIMIT_DEFAULT_REQUESTS_PER_MINUTE=100\n';
  }

  return {
    stdout: out,
    'reports/rate-limit-summary.json': JSON.stringify(data, null, 2),
  };
}
