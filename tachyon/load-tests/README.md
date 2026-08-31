# Tachyon Load Tests

## Prerequisites

- k6 installed: `go install go.k6.io/k6@latest` or `brew install k6`
- Tachyon server running: `cargo run -p tachyon-server`

## Quick Start

Run the orchestrator script:
```bash
./load-tests/run.sh smoke    # Quick smoke test (1 VU)
./load-tests/run.sh load     # Standard load test (50 VUs)
./load-tests/run.sh stress   # Stress test (200 VUs)
./load-tests/run.sh spike    # Spike test (10-200 VUs)
./load-tests/run.sh soak     # Soak test (10 VUs, 30 min)
./load-tests/run.sh ws       # WebSocket reconnection stress
./load-tests/run.sh rate-limit  # Rate limiting validation
./load-tests/run.sh all      # smoke + load + rate-limit
```

## Running Individual Scripts

```bash
BASE_URL=http://localhost:8080 k6 run load-tests/k6/smoke.js
# Shared-user fallback (subject to the configured login/user rate limits)
BASE_URL=http://localhost:8080 k6 run load-tests/k6/LoadTest.js

# Preferred for concurrency: one pre-provisioned token per VU (do not commit or print tokens).
# Supply at least as many tokens as VUs for strict one-identity-per-VU isolation.
BASE_URL=http://localhost:8080 \\
  TEST_TOKENS_JSON='["token-vu-1","token-vu-2"]' \\
  TEST_SKIP_AUTHOPS=true \\
  k6 run load-tests/k6/LoadTest.js
BASE_URL=http://localhost:8080 k6 run load-tests/k6/StressTest.js
BASE_URL=http://localhost:8080 k6 run load-tests/k6/SoakTest.js
BASE_URL=http://localhost:8080 k6 run load-tests/k6/SpikeTest.js
BASE_URL=ws://localhost:8080 k6 run load-tests/k6/WebSocketStress.js
BASE_URL=http://localhost:8080 k6 run load-tests/k6/RateLimitTest.js
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `BASE_URL` | `http://localhost:8080` | Server URL (use `ws://` for WebSocket tests) |
| `K6_BINARY` | `k6` | Path to k6 binary |
| `TEST_USERNAME` | `loadtest` | Username for authenticated load tests |
| `TEST_PASSWORD` | `LoadTest123!` | Password for the shared-user fallback |
| `TEST_TOKENS` | unset | Comma-separated pre-provisioned bearer tokens; VU N selects token `(N-1) % token_count` |
| `TEST_TOKENS_JSON` | unset | JSON string array of pre-provisioned bearer tokens; takes precedence over `TEST_TOKENS` |
| `RATE_LIMIT` | `100` | Expected rate limit threshold |
| `RECONNECT_CYCLES` | `10` | WebSocket reconnect cycles |
| `HEARTBEAT_INTERVAL_MS` | `25000` | WebSocket heartbeat interval |

## Test Descriptions

| Test | VUs | Duration | Purpose |
|------|-----|----------|---------|
| smoke | 1 | 1 iter | Full API lifecycle (register, login, CRUD, search) |
| load | 50 | 3 min | Read/write/auth mix at moderate concurrency |
| stress | 200 | 6.5 min | Connection pool exhaustion, pool monitoring |
| spike | 10-200-10 | 6 min | Sudden traffic burst and recovery |
| soak | 10 | 30 min | Memory leaks, gradual degradation detection |
| ws | 20 | 2 min | WebSocket connect/disconnect/reconnect cycles |
| rate-limit | 1+5 | 30s | Burst 429 detection, rate limit headers, recovery |

## Reports

Results are saved to `load-tests/reports/`:
- `<test>.log` - Console output
- `<test>.json` - Raw k6 metrics (JSON)
- `server.log` - Server output (if auto-started)

## Performance Thresholds

| Metric | Target |
|--------|--------|
| p95 latency (load) | < 500ms |
| p99 latency (load) | < 2000ms |
| p95 latency (stress) | < 1000ms |
| Error rate (load) | < 1% |
| Error rate (stress) | < 5% |
| Connection pool exhaustion | < 10% |
| Rate limit headers present | > 90% |
