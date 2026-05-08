#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
K6_DIR="${SCRIPT_DIR}/k6"
REPORTS_DIR="${SCRIPT_DIR}/reports"
BASE_URL="${BASE_URL:-http://localhost:8080}"
SERVER_PID=""
K6_BINARY="${K6_BINARY:-k6}"

mkdir -p "${REPORTS_DIR}"

cleanup() {
  if [[ -n "${SERVER_PID}" ]] && kill -0 "${SERVER_PID}" 2>/dev/null; then
    echo "Stopping Tachyon server (PID: ${SERVER_PID})..."
    kill "${SERVER_PID}" 2>/dev/null || true
    wait "${SERVER_PID}" 2>/dev/null || true
  fi
  echo "Cleanup complete."
}
trap cleanup EXIT

check_k6() {
  if ! command -v "${K6_BINARY}" &>/dev/null; then
    echo "ERROR: k6 is not installed."
    echo ""
    echo "Install k6:"
    echo "  macOS:  brew install k6"
    echo "  Linux:  sudo apt install k6  # or: sudo gpg -k && sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69"
    echo "          echo \"deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main\" | sudo tee /etc/apt/sources.list.d/k6.list"
    echo "          sudo apt update && sudo apt install k6"
    echo "  Docker: docker run --rm -i grafana/k6 run --help"
    echo "  Binary: https://github.com/grafana/k6/releases"
    echo ""
    echo "Or set K6_BINARY to point to your k6 binary."
    exit 1
  fi
}

ensure_server() {
  if curl -sf "${BASE_URL}/health" > /dev/null 2>&1; then
    echo "Server already running at ${BASE_URL}"
    return
  fi

  echo "Starting Tachyon server..."

  local server_bin="${SCRIPT_DIR}/../target/debug/tachyon-server"
  if [[ ! -f "${server_bin}" ]]; then
    server_bin="${SCRIPT_DIR}/../target/release/tachyon-server"
  fi

  if [[ ! -f "${server_bin}" ]]; then
    echo "Building Tachyon server..."
    cargo build --release --package tachyon-server 2>/dev/null || \
      cargo build --package tachyon-server 2>/dev/null || {
        echo "ERROR: Failed to build tachyon-server."
        echo "Please build it manually: cargo build --release -p tachyon-server"
        exit 1
      }
    server_bin="${SCRIPT_DIR}/../target/release/tachyon-server"
  fi

  RUST_LOG=info "${server_bin}" > "${REPORTS_DIR}/server.log" 2>&1 &
  SERVER_PID=$!
  echo "Server starting (PID: ${SERVER_PID})..."

  local max_wait=30
  local waited=0
  while ! curl -sf "${BASE_URL}/health" > /dev/null 2>&1; do
    if [[ ${waited} -ge ${max_wait} ]]; then
      echo "ERROR: Server did not start within ${max_wait}s."
      echo "Check ${REPORTS_DIR}/server.log for details."
      cat "${REPORTS_DIR}/server.log"
      exit 1
    fi
    sleep 1
    waited=$((waited + 1))
  done

  echo "Server is healthy (took ${waited}s to start)"
}

run_test() {
  local name="$1"
  local script="$2"
  local extra_args="${3:-}"

  if [[ ! -f "${script}" ]]; then
    echo "SKIP: ${name} (${script} not found)"
    return 0
  fi

  echo ""
  echo "========================================"
  echo "  Running: ${name}"
  echo "========================================"

  local report_json="${REPORTS_DIR}/${name}.json"
  local report_html="${REPORTS_DIR}/${name}.html"

  local cmd=("${K6_BINARY}" run --out json="${report_json}" --out influxdb=http://localhost:8086/k6 2>/dev/null || true)

  K6_OPTIONS="${extra_args}" BASE_URL="${BASE_URL}" \
    "${K6_BINARY}" run \
    --out json="${report_json}" \
    ${extra_args} \
    "${script}" \
    2>&1 | tee "${REPORTS_DIR}/${name}.log" || true

  echo ""
  echo "${name} complete. Results in ${REPORTS_DIR}/${name}.log"
}

print_usage() {
  echo "Usage: $0 [TEST_TYPE]"
  echo ""
  echo "Test types:"
  echo "  smoke    - Quick smoke test (1 VU, 1 iteration)"
  echo "  load     - Standard load test (50 VUs, 3 min)"
  echo "  stress   - Stress test (200 VUs, 6.5 min)"
  echo "  soak     - Soak test (10 VUs, 30 min)"
  echo "  spike    - Spike test (10-200-10 VUs, 6 min)"
  echo "  all      - Run smoke + load"
  echo "  help     - Show this message"
  echo ""
  echo "Environment variables:"
  echo "  BASE_URL          Server URL (default: http://localhost:8080)"
  echo "  K6_BINARY         Path to k6 binary (default: k6)"
  echo "  TEST_USERNAME     Username for load/stress tests (default: loadtest)"
  echo "  TEST_PASSWORD     Password for load/stress tests"
  echo ""
  echo "Reports are saved to: ${REPORTS_DIR}/"
}

main() {
  local test_type="${1:-all}"

  case "${test_type}" in
    help|--help|-h)
      print_usage
      exit 0
      ;;
  esac

  check_k6
  ensure_server

  case "${test_type}" in
    smoke)
      run_test "smoke" "${K6_DIR}/smoke.js"
      ;;
    load)
      run_test "load" "${K6_DIR}/LoadTest.js"
      ;;
    stress)
      run_test "stress" "${K6_DIR}/StressTest.js"
      ;;
    soak)
      run_test "soak" "${K6_DIR}/SoakTest.js"
      ;;
    spike)
      run_test "spike" "${K6_DIR}/SpikeTest.js"
      ;;
    all)
      echo "=== Running all tests ==="
      run_test "smoke" "${K6_DIR}/smoke.js" || {
        echo "WARNING: Smoke test failed. Continuing..."
      }
      run_test "load" "${K6_DIR}/LoadTest.js"
      ;;
    *)
      echo "ERROR: Unknown test type '${test_type}'"
      print_usage
      exit 1
      ;;
  esac

  echo ""
  echo "========================================"
  echo "  All tests complete!"
  echo "  Reports: ${REPORTS_DIR}/"
  echo "========================================"
}

main "$@"
