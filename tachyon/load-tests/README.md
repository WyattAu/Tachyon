# Tachyon Load Tests

## Prerequisites

- k6 installed: `go install go.k6.io/k6@latest`
- Tachyon server running: `cargo run -p tachyon-server`

## Running

Basic load test:
```bash
k6 run load-tests/k6-load.js
```

With custom configuration:
```bash
BASE_URL=http://localhost:8080 k6 run load-tests/k6-load.js
```

High concurrency test:
```bash
k6 run -e RAMP_DURATION=10s -e TEST_DURATION=30s load-tests/k6-load.js
```
