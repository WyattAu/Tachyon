# Debug Control Plane

Agent-driven testing for the Tachyon desktop app. Compiled into debug builds
only; at runtime requires `TACHYON_DEBUG >= 1`. Binds `127.0.0.1:17890`
(configurable via `TACHYON_CONTROL_PORT`), token-gated via
`/tmp/tachyon-control.token` (0600, generated per run or pinned via
`TACHYON_CONTROL_TOKEN`).

## Quick start

```bash
# 1. Launch the app in debug mode
TACHYON_DEBUG=1 \
TACHYON_API_URL=http://192.168.1.191:18080 \
TACHYON_TEST_USER=admin TACHYON_TEST_PASSWORD=admin123 \
cargo run -p tachyon-desktop-app

# 2. Authenticate (token is printed to logs, stored in the token file)
TOKEN=$(cat /tmp/tachyon-control.token)
curl -s -H "Authorization: Bearer $TOKEN" http://127.0.0.1:17890/control/state

# 3. Drive the app
H="Authorization: Bearer $TOKEN"
curl -s -H "$H" -X POST http://127.0.0.1:17890/control/navigate -d '{"route":"/documents"}'
curl -s -H "$H" -X POST http://127.0.0.1:17890/control/type \
  -d '{"selector":"input[type=\"password\"]","text":"secret"}'
curl -s -H "$H" -X POST http://127.0.0.1:17890/control/click -d '{"selector":"button[type=submit]"}'
curl -s -H "$H" -X POST http://127.0.0.1:17890/control/screenshot -d '{}'
curl -s -H "$H" "http://127.0.0.1:17890/control/events?since=0"
```

## Endpoints

| Method | Path | Body | Returns |
|---|---|---|---|
| GET | `/control/state` | — | run_id, route, config |
| POST | `/control/navigate` | `{route}` | `eval_id` |
| POST | `/control/eval` | `{js}` | `eval_id` (result via events) |
| POST | `/control/click` | `{selector}` | `eval_id` |
| POST | `/control/type` | `{selector, text}` | `eval_id` |
| POST | `/control/key` | `{key}` | `eval_id` |
| POST | `/control/screenshot` | `{}` | `{path}` |
| GET | `/control/events?since=N` | — | `{events:[{seq,kind,...}]}` |
| GET | `/control/report` | — | `{report: markdown}` |

Eval results are asynchronous (Tauri `eval()` is fire-and-forget): each action
returns an `eval_id`, and the outcome arrives as an event
`{kind:"eval_result", eval_id, ok, result}` on `/control/events`. Poll with
`since=<last seq>`.

## Correlation

- Every `api_proxy` HTTP request carries `x-request-id: <run_id>.<action_id>`;
  the server echoes it into its request logs.
- Every `debug_report` line carries `run_id`.
- Pin a scenario with `TACHYON_RUN_ID=<id>` to join desktop events, server
  request logs (`/tmp/tachyon.log` on staging), and audit trails by one id.

## Test fixture env vars

| Var | Default | Purpose |
|---|---|---|
| `TACHYON_TEST_SERVER` | `http://127.0.0.1:8080` | API base for auto-login |
| `TACHYON_TEST_USER` | `admin` | traversal login user |
| `TACHYON_TEST_PASSWORD` | `admin` | traversal login password |
| `TACHYON_TEST_DOC_ID` | (dev doc uuid) | doc used by `EDITOR_TEST` |
| `TACHYON_TEST_ROUTES` | core routes | comma-separated traverse list |
| `TACHYON_RUN_ID` | generated | correlation id |
| `TACHYON_DEBUG` | `0` | `1` hooks + control plane, `2` + auto-traverse |
| `TACHYON_CONTROL_PORT` | `17890` | control plane port |
| `TACHYON_CONTROL_TOKEN` | generated | pin the control token (CI) |

## Server-side telemetry (this cycle)

- Query strings are redacted in request logs (`token`, `api_key`, `session`,
  `password`, `secret`, `auth`, `otp`, … → `[REDACTED]`).
- Prometheus metrics now carry `method`/`route`/`status` labels; dynamic path
  segments are normalized (`/documents/<uuid>` → `/documents/:id`) to keep
  cardinality bounded.
