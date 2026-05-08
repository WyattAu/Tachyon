# Tachyon Load Tests

k6-based load testing suite for the Tachyon API server.

## Prerequisites

- [k6](https://k6.io/docs/getting-started/installation/) installed
- Tachyon server running (or the script will build and start it automatically)

### Install k6

```bash
# macOS
brew install k6

# Ubuntu/Debian
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg \
  --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" \
  | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt update && sudo apt install k6

# Docker
docker run --rm -i grafana/k6 run --help

# Binary download
# https://github.com/grafana/k6/releases
```

## Quick Start

```bash
# Run all tests (smoke + load)
./run.sh all

# Run a specific test
./run.sh smoke
./run.sh load
./run.sh stress
./run.sh soak
./run.sh spike
```

The `run.sh` script will:
1. Check that k6 is installed
2. Start a local Tachyon server if one is not already running
3. Run the requested test(s)
4. Save reports to `load-tests/reports/`

## Test Types

### Smoke Test (`smoke.js`)

Quick sanity check that all endpoints work.

- **VUs:** 1
- **Iterations:** 1
- **Duration:** ~5 seconds
- **Thresholds:** All requests < 500ms

Tests the full user flow: register, login, create document, get document, update document, list documents, search, auth status, logout.

### Load Test (`LoadTest.js`)

Standard load test with a realistic mix of operations.

- **Ramp-up:** 0 → 50 VUs over 30s
- **Sustained:** 50 VUs for 2 minutes
- **Ramp-down:** 50 → 0 over 30s
- **Total duration:** 3 minutes
- **Thresholds:** p95 < 500ms, p99 < 2000ms, error rate < 1%

Operation mix:
| Operation   | Weight | Endpoints                           |
|-------------|--------|-------------------------------------|
| Read        | 40%    | `GET /documents`, `GET /search`     |
| Write       | 30%    | `POST /documents`, `PUT /documents` |
| Auth        | 20%    | `POST /auth/login`                  |
| Navigation  | 10%    | `GET /health`, `GET /auth/status`   |

### Stress Test (`StressTest.js`)

Pushes the system to find breaking points.

- **Ramp-up:** 0 → 200 VUs over 1 minute
- **Sustained:** 200 VUs for 5 minutes
- **Ramp-down:** 50 → 0 over 30s
- **Total duration:** 6.5 minutes
- **Thresholds:** p95 < 1000ms, error rate < 5%

Monitors:
- Connection pool exhaustion (via `/health` endpoint pool info)
- 503 Service Unavailable responses
- Connection errors
- Response size trends

### Soak Test (`SoakTest.js`)

Long-duration test to detect memory leaks and gradual degradation.

- **VUs:** 10
- **Duration:** 30 minutes
- **Thresholds:** p95 < 500ms, error rate < 1%, degradation windows < 5%

Monitors:
- Latency trends over time
- Connection pool idle connections (leaks show as declining idle count)
- Error rate trends

### Spike Test (`SpikeTest.js`)

Tests system recovery from sudden traffic bursts.

- **Baseline:** 10 VUs for 2 minutes
- **Spike:** 200 VUs for 1 minute
- **Recovery:** 10 VUs for 3 minutes
- **Total duration:** 6 minutes
- **Thresholds:** p95 < 1000ms, spike error rate < 10%

Analyzes:
- Baseline vs spike latency ratio
- Recovery latency vs baseline ratio
- Whether the system returns to baseline performance after the spike

## Customization

### Environment Variables

| Variable        | Default                    | Description                      |
|-----------------|----------------------------|----------------------------------|
| `BASE_URL`      | `http://localhost:8080`    | Target server URL                |
| `K6_BINARY`     | `k6`                       | Path to k6 binary                |
| `TEST_USERNAME` | `loadtest`                 | Username for authenticated tests |
| `TEST_PASSWORD` | `LoadTest123!`             | Password for authenticated tests |

```bash
# Run against staging
BASE_URL=https://staging.tachyon.app ./run.sh load

# Run against production (carefully!)
BASE_URL=https://tachyon.app TEST_USERNAME=prod_user TEST_PASSWORD=secret ./run.sh smoke
```

### Changing VUs and Duration

Edit the `options` export in each test file:

```javascript
// LoadTest.js
export const options = {
  stages: [
    { duration: '1m', target: 100 },   // Ramp to 100 VUs over 1 min
    { duration: '5m', target: 100 },   // Hold at 100 VUs for 5 min
    { duration: '1m', target: 0 },     // Ramp down over 1 min
  ],
};
```

### Changing Thresholds

```javascript
export const options = {
  thresholds: {
    http_req_duration: ['p(95)<1000', 'p(99)<3000'],  // Loosen thresholds
    http_req_failed: ['rate<0.05'],                    // Allow 5% errors
  },
};
```

### Adding Custom Metrics

Each test file can define custom metrics:

```javascript
import { Rate, Trend, Counter } from 'k6/metrics';

const myRate = new Rate('my_custom_rate');
const myTrend = new Trend('my_custom_trend');

export default function () {
  const res = http.get('...');
  myRate.add(res.status === 200);
  myTrend.add(res.timings.duration);
}
```

## Interpreting Results

### Console Output

k6 prints real-time progress and a summary. Key fields:

- **Checks:** Pass/fail rate for assertions
- **Data received/sent:** Total bandwidth
- **http_req_duration:** Request latency (p50, p95, p99, max, min, avg)

### Threshold Pass/Fail

k6 exits with code 0 if all thresholds pass, non-zero otherwise. If thresholds fail, review:
- **High p95/p99:** Database queries may be slow; check query plans and connection pool size
- **High error rate:** Look for 500 errors in the log; check server logs for stack traces
- **Pool exhaustion:** Increase database pool size or add connection pooling middleware

### Reports

Reports are saved to `load-tests/reports/`:

| File                    | Description                    |
|-------------------------|--------------------------------|
| `<test>.log`            | Console output                 |
| `<test>.json`           | Raw k6 metrics (JSON)          |
| `<test>-summary.json`   | Parsed summary (stress/soak)   |
| `server.log`            | Server output (if auto-started)|

Import JSON reports into [Grafana](https://k6.io/docs/results-visualization/grafana/) for visualization:

```bash
# With Grafana Cloud or local Grafana + InfluxDB
k6 run --out influxdb=http://localhost:8086/k6 k6/LoadTest.js
```

### Common Patterns

| Symptom                      | Likely Cause                          | Fix                            |
|------------------------------|---------------------------------------|--------------------------------|
| Latency climbs steadily      | Memory leak or connection leak        | Profile server, check pool size |
| 503 during spike             | Connection pool exhausted             | Increase max connections       |
| Errors only in spike phase   | Request queue overflow                | Add buffering, increase workers |
| Recovery latency stays high  | Resources not released after spike    | Check for goroutine/thread leaks |
| P99 >> P95                   | Outlier requests (GC pauses, cold starts) | Check server GC tuning   |

## Running Against Staging/Production

```bash
# Staging
BASE_URL=https://staging-api.tachyon.app \
  TEST_USERNAME=staging_tester \
  TEST_PASSWORD=staging_password \
  ./run.sh smoke

# Production (start with smoke only!)
BASE_URL=https://api.tachyon.app \
  TEST_USERNAME=prod_monitor \
  TEST_PASSWORD=prod_monitor_secret \
  ./run.sh smoke

# Production load test (use lower VUs)
BASE_URL=https://api.tachyon.app k6 run \
  --env BASE_URL=https://api.tachyon.app \
  k6/LoadTest.js
```

**Warning:** Always start with a smoke test against production. Only run load/stress tests with careful monitoring and team approval.
