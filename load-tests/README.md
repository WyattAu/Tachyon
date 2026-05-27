# Load Tests

Requires k6: https://k6.io/docs/getting-started/installation/

## Smoke Test (quick validation)

    k6 run load-tests/api-smoke.js

## Stress Test (1,000 concurrent)

    k6 run load-tests/api-stress.js

## Custom Target

    BASE_URL=http://localhost:8080 k6 run load-tests/api-smoke.js
